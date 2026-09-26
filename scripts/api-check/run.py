#!/usr/bin/env python3
"""Weekly live API check for ethcli's upstream providers.

Runs each read-only command from checks.txt against the real APIs and
classifies the outcome:

  PASS  command succeeded
  FAIL  the API changed or disappeared: HTTP 400/404/405/410, DNS failure,
        a response ethcli can no longer parse, or an auth error from an API
        that is supposed to be keyless
  WARN  probably not a code problem: rate limit, 5xx, timeout, geo-block,
        or an auth/plan error on a keyed API
  SKIP  the required API key is not configured

Every non-PASS result is retried once. Exit status is 1 if any check FAILs.

Usage: run.py --ethcli PATH [--checks FILE] [--json OUT] [--summary OUT]
"""
import argparse
import json
import os
import re
import shlex
import socket
import subprocess
import sys
import time
from pathlib import Path

TIMEOUT_S = 90
RETRY_DELAY_S = 10

# Order matters: the first matching rule wins.
FAIL_PATTERNS = [
    (r"error decoding response body|missing field|invalid type|unknown variant|"
     r"expected value|failed to parse|deserializ|invalid response", "response no longer parses"),
    (r"\b(404|410)\b|not found", "endpoint missing (404/410)"),
    (r"\b405\b|method not allowed", "method not allowed (405)"),
    (r"\b400\b|bad request|\b422\b|unprocessable", "request rejected (400/422): request shape may have drifted"),
    (r"dns error|failed to lookup address|name or service not known|nodename nor servname|no such host",
     "DNS failure: host gone"),
]
AUTH_PATTERN = r"\b(401|403)\b|unauthori[sz]ed|forbidden|invalid api key|api key.*(invalid|expired)"
WARN_PATTERNS = [
    (r"\b429\b|too many requests|rate.?limit", "rate limited"),
    (r"\b(500|502|503|504|52\d|530)\b|internal server error|bad gateway|service unavailable|gateway time",
     "server error"),
    (r"timed? ?out|timeout|deadline", "timeout"),
    (r"\b451\b|legal reasons|restricted location", "geo-blocked from runner"),
    (r"paused|quota|credits|exceeded|capacity limit|upgrade your plan", "plan/quota limit"),
    (r"connection (reset|refused|closed)|error sending request", "connection error"),
]
MISSING_KEY = r"(api[_ ]?key|access key|credentials?).{0,40}(not (set|configured|found)|required|missing)"

SECRET_RE = re.compile(r"(?i)(key|token|secret|apikey|api-key)=[^&\s\"']+")
URL_RE = re.compile(r"(https?|wss?)://([^/\s\"'@]*@)?([^/\s\"':]+)[^\s\"']*")


def redact(text):
    text = URL_RE.sub(lambda m: f"{m.group(1)}://{m.group(3)}/…", text)
    text = SECRET_RE.sub(lambda m: m.group(1) + "=***", text)
    for name, value in os.environ.items():
        if value and len(value) >= 8 and re.search(r"KEY|TOKEN|SECRET", name):
            text = text.replace(value, "***")
    return text


def parse_checks(path):
    checks = []
    for n, raw in enumerate(Path(path).read_text().splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = [p.strip() for p in line.split("|")]
        # "A|B" in the env column is split too; the args are always the last part.
        if len(parts) < 3:
            sys.exit(f"{path}:{n}: expected 'provider | env | args'")
        provider, env_spec, args = parts[0], "|".join(parts[1:-1]), parts[-1]
        checks.append({"line": n, "provider": provider, "env": env_spec, "args": args})
    return checks


def env_satisfied(spec):
    if spec in ("", "-"):
        return True, []
    missing = []
    for group in spec.split(","):
        options = [o.strip() for o in group.split("|") if o.strip()]
        if not any(os.environ.get(o) for o in options):
            missing.append(" or ".join(options))
    return not missing, missing


def host_resolves(host):
    try:
        socket.getaddrinfo(host, 443)
        return True
    except OSError:
        return False


def classify(exit_code, out, err, keyless, resolve=host_resolves):
    text = f"{out}\n{err}"
    lower = text.lower()
    # "Error:" lines only; aggregate commands list per-source failures under an
    # "Errors:" heading while still succeeding, and those are covered by the
    # per-provider checks.
    error_lines = [l for l in text.splitlines() if re.match(r"\s*(error\b|✗)", l, re.I)]
    failed = exit_code != 0 or bool(error_lines) or not out.strip()
    if not failed:
        return "PASS", ""
    detail = redact((error_lines or [l for l in text.splitlines() if l.strip()] or [f"exit {exit_code}"])[-1].strip())[:200]
    if re.search(MISSING_KEY, lower):
        return "SKIP", "API key not configured: " + detail
    for pattern, reason in FAIL_PATTERNS:
        if re.search(pattern, lower):
            return "FAIL", f"{reason}: {detail}"
    # ethcli reports connection failures without the underlying cause, so
    # check DNS ourselves: a host that no longer resolves is gone, not flaky.
    if re.search(r"error sending request|connection (reset|refused|closed)", lower):
        m = re.search(r"https?://(?:[^/\s@]*@)?([a-z0-9.-]+)", text, re.I)
        if m and not resolve(m.group(1)):
            return "FAIL", f"DNS failure: host gone: {detail}"
    if re.search(AUTH_PATTERN, lower):
        if keyless:
            return "FAIL", f"keyless API now requires auth: {detail}"
        return "WARN", f"auth/plan problem with configured key: {detail}"
    for pattern, reason in WARN_PATTERNS:
        if re.search(pattern, lower):
            return "WARN", f"{reason}: {detail}"
    return "FAIL", f"unclassified error: {detail}"


def run_check(ethcli, check):
    cmd = [ethcli, *shlex.split(check["args"])]
    start = time.monotonic()
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=TIMEOUT_S, stdin=subprocess.DEVNULL)
        code, out, err = p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        code, out, err = 124, "", f"Error: timed out after {TIMEOUT_S}s"
    status, detail = classify(code, out, err, keyless=check["env"] in ("", "-"))
    return status, detail, round(time.monotonic() - start, 1)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ethcli", required=True)
    ap.add_argument("--checks", default=str(Path(__file__).with_name("checks.txt")))
    ap.add_argument("--json")
    ap.add_argument("--summary", help="append a markdown summary here (e.g. $GITHUB_STEP_SUMMARY)")
    ap.add_argument("--only", help="regex: run only providers matching this")
    args = ap.parse_args()

    results = []
    for check in parse_checks(args.checks):
        if args.only and not re.search(args.only, check["provider"]):
            continue
        ok, missing = env_satisfied(check["env"])
        if not ok:
            r = {**check, "status": "SKIP", "detail": "not configured: " + ", ".join(missing), "secs": 0, "attempts": 0}
        else:
            status, detail, secs = run_check(args.ethcli, check)
            attempts = 1
            if status in ("FAIL", "WARN"):
                time.sleep(RETRY_DELAY_S)
                status2, detail2, secs2 = run_check(args.ethcli, check)
                attempts = 2
                # Keep the better outcome: a transient failure should not fail the run.
                rank = {"PASS": 0, "SKIP": 1, "WARN": 2, "FAIL": 3}
                if rank[status2] <= rank[status]:
                    status, detail, secs = status2, detail2, secs2
            r = {**check, "status": status, "detail": detail, "secs": secs, "attempts": attempts}
        results.append(r)
        print(f"{r['status']:<4} {r['provider']:<18} ethcli {r['args'][:70]:<70} {r['detail'][:120]}", flush=True)

    counts = {s: sum(r["status"] == s for r in results) for s in ("PASS", "FAIL", "WARN", "SKIP")}
    print("\n" + "  ".join(f"{k}={v}" for k, v in counts.items()))

    if args.json:
        Path(args.json).write_text(json.dumps({"counts": counts, "results": results}, indent=2))
    if args.summary:
        icon = {"PASS": "✅", "FAIL": "❌", "WARN": "⚠️", "SKIP": "⏭️"}
        lines = ["## Weekly API check", "",
                 " · ".join(f"{icon[k]} {k} {v}" for k, v in counts.items()), "",
                 "| | Provider | Command | Detail |", "|---|---|---|---|"]
        order = {"FAIL": 0, "WARN": 1, "SKIP": 2, "PASS": 3}
        for r in sorted(results, key=lambda r: (order[r["status"]], r["provider"])):
            cmd = r["args"] if len(r["args"]) <= 60 else r["args"][:57] + "…"
            lines.append(f"| {icon[r['status']]} | {r['provider']} | `ethcli {cmd}` | {r['detail'].replace('|', '/')} |")
        with open(args.summary, "a") as f:
            f.write("\n".join(lines) + "\n")
    return 1 if counts["FAIL"] else 0


if __name__ == "__main__":
    sys.exit(main())
