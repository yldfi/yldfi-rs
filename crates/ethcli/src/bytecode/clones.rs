//! Clones with immutable args: proxies whose runtime code is a fixed
//! template followed by data appended at deployment
//!
//! Each template below is the exact runtime code the library deploys, taken
//! from its source, with the implementation address and the data-length
//! fields as wildcards. A code matches only when every literal byte is equal,
//! the length fields agree with the code's actual size, and any length
//! suffix agrees too.

use alloy::primitives::Address;
use serde_json::{json, Value};

/// How a template ends its appended data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Suffix {
    /// No suffix: every byte after the template is data
    None,
    /// A 2-byte big-endian suffix equal to the data length (wighawag)
    DataLen,
    /// A 2-byte suffix equal to the data length plus 2 (Solady before
    /// v0.0.230)
    DataLenPlus2,
}

/// A runtime template. In `hex`, `LL` stands for one byte of a data-length
/// field and `II` for one byte of the implementation address; every other
/// pair is a literal byte. All length fields hold the number of bytes after
/// the template (data plus suffix).
struct Template {
    kind: &'static str,
    hex: &'static str,
    suffix: Suffix,
    /// The appended bytes are passed to the implementation after the
    /// calldata (read there with `_getArg*` from calldata); otherwise the
    /// implementation reads them with EXTCODECOPY
    forwarded_in_calldata: bool,
    /// Without a length field, appended bytes are required (otherwise the
    /// code is the plain proxy and is left to the other detectors)
    needs_data: bool,
    /// Where the implementation comes from when it is not in the code
    implementation_from: Option<&'static str>,
}

/// The implementation address placeholder (20 bytes)
macro_rules! addr {
    () => {
        "IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII"
    };
}

/// Kind label of the wighawag v1.1+ / v2 layout
pub const KIND_WIGHAWAG: &str =
    "Clone with immutable args (wighawag ClonesWithImmutableArgs v1.1+)";
/// Kind label of the wighawag v1.0.0 layout
pub const KIND_WIGHAWAG_V1_0: &str =
    "Clone with immutable args (wighawag ClonesWithImmutableArgs v1.0)";
/// Kind label of the PUSH1 layout of Sudoswap LSSVMPairCloner (v1 and v2)
pub const KIND_SUDOSWAP: &str = "Clone with immutable args (Sudoswap LSSVMPairCloner)";
/// Kind label of Solady LibClone's CWIA before v0.0.230
pub const KIND_SOLADY_OLD: &str = "Clone with immutable args (Solady LibClone, before v0.0.230)";
/// Kind label of Solady LibClone's ERC-1167 with immutable args (v0.0.230+)
pub const KIND_SOLADY_1167: &str = "ERC-1167 clone with immutable args (Solady LibClone)";

const TEMPLATES: [Template; 9] = [
        // wighawag/clones-with-immutable-args v1.1.0 to v2.0.x
        // (ClonesWithImmutableArgs.getCreationBytecode)
        Template {
            kind: KIND_WIGHAWAG,
            hex: concat!(
                "3d3d3d3d363d3d3761LLLL603736393661LLLL013d73",
                addr!(),
                "5af43d3d93803e603557fd5bf3"
            ),
            suffix: Suffix::DataLen,
            forwarded_in_calldata: true,
            needs_data: false,
            implementation_from: None,
        },
        // wighawag/clones-with-immutable-args v1.0.0
        Template {
            kind: KIND_WIGHAWAG_V1_0,
            hex: concat!(
                "363d3d3761LLLL603836393d3d3d3661LLLL013d73",
                addr!(),
                "5af43d82803e903d91603657fd5bf3"
            ),
            suffix: Suffix::DataLen,
            forwarded_in_calldata: true,
            needs_data: false,
            implementation_from: None,
        },
        // sudoswap/lssvm and sudoswap/lssvm2 LSSVMPairCloner: the wighawag
        // layout with PUSH1 lengths and no length suffix
        Template {
            kind: KIND_SUDOSWAP,
            hex: concat!(
                "3d3d3d3d363d3d3760LL603536393660LL013d73",
                addr!(),
                "5af43d3d93803e603357fd5bf3"
            ),
            suffix: Suffix::None,
            forwarded_in_calldata: true,
            needs_data: false,
            implementation_from: None,
        },
        // Vectorized/solady LibClone.clone(implementation, data) before
        // v0.0.230 (with a receive path that emits ReceiveETH(uint256))
        Template {
            kind: KIND_SOLADY_OLD,
            hex: concat!(
                "36602c57343d527f9e4ac34f21c619cefc926c8bd93b54bf5a39c7ab2127a895af1cc0691d7e3dff593da1005b363d3d373d3d3d3d61LLLL806062363936013d73",
                addr!(),
                "5af43d3d93803e606057fd5bf3"
            ),
            suffix: Suffix::DataLenPlus2,
            forwarded_in_calldata: true,
            needs_data: false,
            implementation_from: None,
        },
        // Solady LibClone.clone(implementation, args) from v0.0.230: the
        // EIP-1167 runtime followed by the args
        Template {
            kind: KIND_SOLADY_1167,
            hex: concat!(
                "363d3d373d3d3d363d73",
                addr!(),
                "5af43d82803e903d91602b57fd5bf3"
            ),
            suffix: Suffix::None,
            forwarded_in_calldata: false,
            needs_data: true,
            implementation_from: None,
        },
        // Solady LibClone.deployERC1967(implementation, args)
        Template {
            kind: "ERC-1967 proxy with immutable args (Solady LibClone)",
            hex: "363d3d373d3d363d7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545af43d6000803e6038573d6000fd5b3d6000f3",
            suffix: Suffix::None,
            forwarded_in_calldata: false,
            needs_data: true,
            implementation_from: Some("EIP-1967 implementation slot"),
        },
        // Solady LibClone.deployERC1967I(implementation, args)
        Template {
            kind: "ERC-1967I proxy with immutable args (Solady LibClone)",
            hex: "365814604357363d3d373d3d363d7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545af43d6000803e603e573d6000fd5b3d6000f35b6020600f3d393d51543d52593df3",
            suffix: Suffix::None,
            forwarded_in_calldata: false,
            needs_data: true,
            implementation_from: Some("EIP-1967 implementation slot"),
        },
        // Solady LibClone.deployERC1967BeaconProxy(beacon, args)
        Template {
            kind: "ERC-1967 beacon proxy with immutable args (Solady LibClone)",
            hex: "363d3d373d3d363d602036600436635c60da1b60e01b36527fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50545afa5036515af43d6000803e604d573d6000fd5b3d6000f3",
            suffix: Suffix::None,
            forwarded_in_calldata: false,
            needs_data: true,
            implementation_from: Some("beacon in the EIP-1967 beacon slot"),
        },
        // Solady LibClone.deployERC1967IBeaconProxy(beacon, args)
        Template {
            kind: "ERC-1967I beacon proxy with immutable args (Solady LibClone)",
            hex: "363d3d373d3d363d602036600436635c60da1b60e01b36527fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50545afa361460525736515af43d600060013e6052573d6001fd5b3d6001f3",
            suffix: Suffix::None,
            forwarded_in_calldata: false,
            needs_data: true,
            implementation_from: Some("beacon in the EIP-1967 beacon slot"),
        },
];

/// A recognised clone with immutable args
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmutableArgsClone {
    /// Which template matched
    pub kind: &'static str,
    /// The implementation address embedded in the code (None when the
    /// template reads it from storage or a beacon)
    pub implementation: Option<Address>,
    /// Where the implementation comes from when it is not in the code
    pub implementation_from: Option<&'static str>,
    /// The appended data, without any length suffix
    pub args: Vec<u8>,
    /// The 2-byte length suffix, when the template has one
    pub length_suffix: Option<u16>,
    /// Whether the data is appended to the calldata of the delegatecall
    pub forwarded_in_calldata: bool,
}

/// Match `code` against one template
fn match_template(t: &Template, code: &[u8]) -> Option<ImmutableArgsClone> {
    let pattern = t.hex.as_bytes();
    let header = pattern.len() / 2;
    if code.len() < header {
        return None;
    }
    let appended = code.len() - header;
    let mut implementation = [0u8; 20];
    let mut addr_i = 0;
    // Each run of LL bytes is one big-endian field
    let mut fields: Vec<usize> = Vec::new();
    let mut in_field = false;
    for (i, &byte) in code[..header].iter().enumerate() {
        let pair = &pattern[2 * i..2 * i + 2];
        match pair {
            b"LL" => {
                if !in_field {
                    fields.push(0);
                    in_field = true;
                }
                let last = fields.last_mut()?;
                *last = (*last << 8) | byte as usize;
                continue;
            }
            b"II" => {
                implementation[addr_i] = byte;
                addr_i += 1;
            }
            _ => {
                let lit = u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?;
                if lit != byte {
                    return None;
                }
            }
        }
        in_field = false;
    }
    if fields.iter().any(|&f| f != appended) {
        return None;
    }
    if t.needs_data && appended == 0 {
        return None;
    }
    let tail = &code[header..];
    let (args, length_suffix) = match t.suffix {
        Suffix::None => (tail.to_vec(), None),
        Suffix::DataLen | Suffix::DataLenPlus2 => {
            if tail.len() < 2 {
                return None;
            }
            let data_len = tail.len() - 2;
            let suffix = u16::from_be_bytes([tail[data_len], tail[data_len + 1]]);
            let expected = match t.suffix {
                Suffix::DataLen => data_len,
                _ => data_len + 2,
            };
            if suffix as usize != expected {
                return None;
            }
            (tail[..data_len].to_vec(), Some(suffix))
        }
    };
    Some(ImmutableArgsClone {
        kind: t.kind,
        implementation: (addr_i == 20).then(|| Address::from(implementation)),
        implementation_from: t.implementation_from,
        args,
        length_suffix,
        forwarded_in_calldata: t.forwarded_in_calldata,
    })
}

/// Recognise a clone with immutable args by its exact runtime template
#[must_use]
pub fn detect_immutable_args_clone(code: &[u8]) -> Option<ImmutableArgsClone> {
    TEMPLATES.iter().find_map(|t| match_template(t, code))
}

/// At most this many words and address-looking chunks are listed
const MAX_LISTED: usize = 64;

/// Whether 20 bytes look like an address: at most 3 zero bytes
fn looks_like_address(chunk: &[u8]) -> bool {
    chunk.len() == 20 && chunk.iter().filter(|&&b| b == 0).count() <= 3
}

/// Packed 20-byte chunks that look like addresses, scanning left to right.
/// A window that starts with a small byte is skipped when the next window
/// also qualifies (a packed uint8 before an address, as in Sudoswap pairs)
fn packed_addresses(data: &[u8]) -> Vec<(usize, Address)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 20 <= data.len() && out.len() < MAX_LISTED {
        let here = looks_like_address(&data[i..i + 20]);
        let next = i + 21 <= data.len() && looks_like_address(&data[i + 1..i + 21]);
        if here && !(data[i] < 0x10 && next) {
            out.push((i, Address::from_slice(&data[i..i + 20])));
            i += 20;
        } else {
            i += 1;
        }
    }
    out
}

impl ImmutableArgsClone {
    /// JSON for output: the data in hex plus a best-effort split, labelled
    /// as raw data
    #[must_use]
    pub fn to_json(&self) -> Value {
        let words: Vec<Value> = self
            .args
            .chunks(32)
            .take(MAX_LISTED)
            .enumerate()
            .map(|(i, w)| json!({"offset": i * 32, "hex": format!("0x{}", hex::encode(w))}))
            .collect();
        let abi_addresses: Vec<Value> = self
            .args
            .chunks(32)
            .enumerate()
            .filter(|(_, w)| {
                w.len() == 32 && w[..12].iter().all(|&b| b == 0) && looks_like_address(&w[12..])
            })
            .take(MAX_LISTED)
            .map(|(i, w)| {
                json!({"offset": i * 32 + 12, "address": Address::from_slice(&w[12..]).to_checksum(None)})
            })
            .collect();
        let packed: Vec<Value> = packed_addresses(&self.args)
            .into_iter()
            .map(|(o, a)| json!({"offset": o, "address": a.to_checksum(None)}))
            .collect();
        json!({
            "template": self.kind,
            "implementation": self.implementation.map(|a| a.to_checksum(None)),
            "implementation_from": self.implementation_from.unwrap_or("code"),
            "args_hex": format!("0x{}", hex::encode(&self.args)),
            "args_length": self.args.len(),
            "length_suffix": self.length_suffix,
            "forwarded_in_calldata": self.forwarded_in_calldata,
            "words": words,
            "address_like": {
                "packed": packed,
                "abi_words": abi_addresses,
            },
            "note": "raw data appended to the clone's code; the split into 32-byte words and address-looking 20-byte chunks is a guess, and its meaning depends on the implementation",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMPL: &str = "5615deb798bb3e4dfa0139dfa1b3d433cc23b72f";

    fn code(h: &str) -> Vec<u8> {
        hex::decode(h).unwrap()
    }

    fn imp() -> Address {
        IMPL.parse().unwrap()
    }

    // Runtime code generated with forge from each library's source, with
    // implementation 0x5615...b72f and data 0xaabbccdd

    #[test]
    fn wighawag_current() {
        let c = code(&format!("3d3d3d3d363d3d376100066037363936610006013d73{IMPL}5af43d3d93803e603557fd5bf3aabbccdd0004"));
        let d = detect_immutable_args_clone(&c).unwrap();
        assert_eq!(d.kind, KIND_WIGHAWAG);
        assert_eq!(d.implementation, Some(imp()));
        assert_eq!(d.args, code("aabbccdd"));
        assert_eq!(d.length_suffix, Some(4));
        assert!(d.forwarded_in_calldata);
        // A wrong suffix or a length field that disagrees with the size
        let mut bad = c.clone();
        *bad.last_mut().unwrap() = 5;
        assert!(detect_immutable_args_clone(&bad).is_none());
        let mut short = c.clone();
        short.pop();
        assert!(detect_immutable_args_clone(&short).is_none());
    }

    #[test]
    fn wighawag_v1_0() {
        let c = code(&format!(
            "363d3d37610006603836393d3d3d36610006013d73{IMPL}5af43d82803e903d91603657fd5bf3aabbccdd0004"
        ));
        let d = detect_immutable_args_clone(&c).unwrap();
        assert_eq!(d.kind, KIND_WIGHAWAG_V1_0);
        assert_eq!(d.implementation, Some(imp()));
        assert_eq!(d.args, code("aabbccdd"));
    }

    #[test]
    fn solady_before_0_0_230() {
        let c = code(&format!("36602c57343d527f9e4ac34f21c619cefc926c8bd93b54bf5a39c7ab2127a895af1cc0691d7e3dff593da1005b363d3d373d3d3d3d610006806062363936013d73{IMPL}5af43d3d93803e606057fd5bf3aabbccdd0006"));
        let d = detect_immutable_args_clone(&c).unwrap();
        assert_eq!(d.kind, KIND_SOLADY_OLD);
        assert_eq!(d.implementation, Some(imp()));
        assert_eq!(d.args, code("aabbccdd"));
        assert_eq!(d.length_suffix, Some(6));
    }

    #[test]
    fn solady_erc1167_with_args() {
        let c = code(&format!(
            "363d3d373d3d3d363d73{IMPL}5af43d82803e903d91602b57fd5bf3aabbccdd"
        ));
        let d = detect_immutable_args_clone(&c).unwrap();
        assert_eq!(d.kind, KIND_SOLADY_1167);
        assert_eq!(d.implementation, Some(imp()));
        assert_eq!(d.args, code("aabbccdd"));
        assert!(!d.forwarded_in_calldata);
        // The plain 45-byte EIP-1167 clone is not a CWIA clone
        let plain = code(&format!(
            "363d3d373d3d3d363d73{IMPL}5af43d82803e903d91602b57fd5bf3"
        ));
        assert!(detect_immutable_args_clone(&plain).is_none());
    }

    #[test]
    fn solady_erc1967_family_with_args() {
        for (h, from) in [
            ("363d3d373d3d363d7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545af43d6000803e6038573d6000fd5b3d6000f3", "EIP-1967 implementation slot"),
            ("365814604357363d3d373d3d363d7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545af43d6000803e603e573d6000fd5b3d6000f35b6020600f3d393d51543d52593df3", "EIP-1967 implementation slot"),
            ("363d3d373d3d363d602036600436635c60da1b60e01b36527fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50545afa5036515af43d6000803e604d573d6000fd5b3d6000f3", "beacon in the EIP-1967 beacon slot"),
            ("363d3d373d3d363d602036600436635c60da1b60e01b36527fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50545afa361460525736515af43d600060013e6052573d6001fd5b3d6001f3", "beacon in the EIP-1967 beacon slot"),
        ] {
            let d = detect_immutable_args_clone(&code(&format!("{h}aabbccdd"))).unwrap();
            assert_eq!(d.implementation, None);
            assert_eq!(d.implementation_from, Some(from));
            assert_eq!(d.args, code("aabbccdd"));
            // Without args it is the plain proxy
            assert!(detect_immutable_args_clone(&code(h)).is_none());
        }
    }

    #[test]
    fn sudoswap_pairs_from_mainnet() {
        // v1 ERC721/ETH pair 0x3689f77a209ea08b05cc2db202ee9694fa4791d8
        let v1 = code("3d3d3d3d363d3d37603d6035363936603d013d7308ce97807a81896e85841d74fb7e7b065ab3ef055af43d3d93803e603357fd5bf3b16c1342e617a5b6e4b631eb114483fdb289c0a45b6ac51d9b1cede0068a1b26533cace807f883ee5af0d9827e0c53e4799bb226655a1de152a425a501");
        let d = detect_immutable_args_clone(&v1).unwrap();
        assert_eq!(d.kind, KIND_SUDOSWAP);
        assert_eq!(
            d.implementation,
            Some(
                "0x08ce97807a81896e85841d74fb7e7b065ab3ef05"
                    .parse()
                    .unwrap()
            )
        );
        assert_eq!(d.args.len(), 61);
        let found: Vec<usize> = packed_addresses(&d.args).iter().map(|(o, _)| *o).collect();
        assert_eq!(found, vec![0, 20, 40]);
        // v2 ERC721/ETH pair 0x93a47564539be57ae27163ab71fb641a5c8bf172
        let v2 = code("3d3d3d3d363d3d37605160353639366051013d734f3742021ed9ee6109bde80814bdad9859cebc5f5af43d3d93803e603357fd5bf3a020d57ab0448ef74115c112d18a9c231cc860001fd5876d4a3860eb0159055a3b7cb79fdfff6b67fed18c828277e3bd8610f9bae432e65a651706f7010000000000000000000000000000000000000000");
        let d = detect_immutable_args_clone(&v2).unwrap();
        assert_eq!(d.kind, KIND_SUDOSWAP);
        assert_eq!(d.args.len(), 0x51);
        assert_eq!(
            d.args[0..20],
            code("a020d57ab0448ef74115c112d18a9c231cc86000")[..]
        );
        let json = d.to_json();
        assert_eq!(json["args_length"], 81);
        assert_eq!(json["words"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn packed_address_after_a_small_byte() {
        // a, b, then uint8 2, then an address: the address is read at 41
        let a = [0x11u8; 20];
        let b = [0x22u8; 20];
        let t = [0x33u8; 20];
        let data = [&a[..], &b[..], &[2u8][..], &t[..]].concat();
        let found: Vec<usize> = packed_addresses(&data).iter().map(|(o, _)| *o).collect();
        assert_eq!(found, vec![0, 20, 41]);
    }

    #[test]
    fn unrelated_code_is_not_matched() {
        assert!(detect_immutable_args_clone(&code("6080604052")).is_none());
        assert!(detect_immutable_args_clone(&[]).is_none());
    }
}
