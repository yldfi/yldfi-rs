//! Alignment of two instruction sequences, for `contract match --diff`.
//!
//! Common prefix and suffix are trimmed first. The middle is aligned by an
//! LCS table when it is small, and otherwise split at instructions that occur
//! exactly once on both sides (patience diff) until the pieces are small.
//! A piece that is still too large and has no such anchor is reported as
//! replaced wholesale, and the result is flagged as approximate.

use std::collections::HashMap;
use std::hash::Hash;

/// One step of an alignment: indexes into the left (artifact) and right
/// (deployed) sequences
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Equal(usize, usize),
    Delete(usize),
    Insert(usize),
}

/// Largest LCS table (cells) built for one piece
const MAX_CELLS: usize = 4_000_000;

/// Align `n` left items with `m` right items. `eq(i, j)` decides equality;
/// `key_l`/`key_r` give a hashable key consistent with `eq` for anchoring
/// (items whose keys are `None` never anchor). Returns the steps and whether
/// some piece had to be treated as replaced without alignment.
pub fn align<K: Hash + Eq>(
    n: usize,
    m: usize,
    eq: &dyn Fn(usize, usize) -> bool,
    key_l: &dyn Fn(usize) -> Option<K>,
    key_r: &dyn Fn(usize) -> Option<K>,
) -> (Vec<Step>, bool) {
    let mut out = Vec::with_capacity(n.max(m));
    let mut approximate = false;
    piece(0, n, 0, m, eq, key_l, key_r, &mut out, &mut approximate);
    (out, approximate)
}

#[allow(clippy::too_many_arguments)]
fn piece<K: Hash + Eq>(
    mut a0: usize,
    mut a1: usize,
    mut b0: usize,
    mut b1: usize,
    eq: &dyn Fn(usize, usize) -> bool,
    key_l: &dyn Fn(usize) -> Option<K>,
    key_r: &dyn Fn(usize) -> Option<K>,
    out: &mut Vec<Step>,
    approximate: &mut bool,
) {
    // Common prefix
    while a0 < a1 && b0 < b1 && eq(a0, b0) {
        out.push(Step::Equal(a0, b0));
        a0 += 1;
        b0 += 1;
    }
    // Common suffix, emitted after the middle
    let mut suffix = 0;
    while a1 > a0 && b1 > b0 && eq(a1 - 1, b1 - 1) {
        a1 -= 1;
        b1 -= 1;
        suffix += 1;
    }
    let (n, m) = (a1 - a0, b1 - b0);
    if n == 0 || m == 0 {
        out.extend((a0..a1).map(Step::Delete));
        out.extend((b0..b1).map(Step::Insert));
    } else if n.saturating_mul(m) <= MAX_CELLS {
        lcs(a0, a1, b0, b1, eq, out);
    } else {
        // Anchors: keys occurring once on each side, in increasing order on
        // both (longest increasing subsequence of right positions)
        let mut left: HashMap<K, (usize, usize)> = HashMap::new();
        for i in a0..a1 {
            if let Some(k) = key_l(i) {
                left.entry(k).or_insert((i, 0)).1 += 1;
            }
        }
        let mut right: HashMap<K, (usize, usize)> = HashMap::new();
        for j in b0..b1 {
            if let Some(k) = key_r(j) {
                right.entry(k).or_insert((j, 0)).1 += 1;
            }
        }
        let mut pairs: Vec<(usize, usize)> = left
            .iter()
            .filter(|(_, (_, c))| *c == 1)
            .filter_map(|(k, (i, _))| match right.get(k) {
                Some((j, 1)) => Some((*i, *j)),
                _ => None,
            })
            .collect();
        pairs.sort_unstable();
        let anchors = longest_increasing(&pairs);
        if anchors.is_empty() {
            *approximate = true;
            out.extend((a0..a1).map(Step::Delete));
            out.extend((b0..b1).map(Step::Insert));
        } else {
            let (mut pa, mut pb) = (a0, b0);
            for (i, j) in anchors {
                piece(pa, i, pb, j, eq, key_l, key_r, out, approximate);
                out.push(Step::Equal(i, j));
                pa = i + 1;
                pb = j + 1;
            }
            piece(pa, a1, pb, b1, eq, key_l, key_r, out, approximate);
        }
    }
    out.extend((0..suffix).map(|k| Step::Equal(a1 + k, b1 + k)));
}

/// Classic LCS table over one piece
fn lcs(
    a0: usize,
    a1: usize,
    b0: usize,
    b1: usize,
    eq: &dyn Fn(usize, usize) -> bool,
    out: &mut Vec<Step>,
) {
    let (n, m) = (a1 - a0, b1 - b0);
    let w = m + 1;
    // table[i][j] = LCS length of a[i..] and b[j..]
    let mut t = vec![0u32; (n + 1) * w];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i * w + j] = if eq(a0 + i, b0 + j) {
                t[(i + 1) * w + j + 1] + 1
            } else {
                t[(i + 1) * w + j].max(t[i * w + j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if eq(a0 + i, b0 + j) {
            out.push(Step::Equal(a0 + i, b0 + j));
            i += 1;
            j += 1;
        } else if t[(i + 1) * w + j] >= t[i * w + j + 1] {
            out.push(Step::Delete(a0 + i));
            i += 1;
        } else {
            out.push(Step::Insert(b0 + j));
            j += 1;
        }
    }
    out.extend((a0 + i..a1).map(Step::Delete));
    out.extend((b0 + j..b1).map(Step::Insert));
}

/// Longest subsequence of `pairs` (sorted by left index) whose right index
/// increases
fn longest_increasing(pairs: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut tails: Vec<usize> = Vec::new(); // index into pairs
    let mut prev: Vec<Option<usize>> = vec![None; pairs.len()];
    for (k, (_, j)) in pairs.iter().enumerate() {
        let pos = tails.partition_point(|&t| pairs[t].1 < *j);
        if pos > 0 {
            prev[k] = Some(tails[pos - 1]);
        }
        if pos == tails.len() {
            tails.push(k);
        } else {
            tails[pos] = k;
        }
    }
    let mut out = Vec::new();
    let mut cur = tails.last().copied();
    while let Some(k) = cur {
        out.push(pairs[k]);
        cur = prev[k];
    }
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(a: &[u8], b: &[u8]) -> Vec<Step> {
        align(
            a.len(),
            b.len(),
            &|i, j| a[i] == b[j],
            &|i| Some(a[i]),
            &|j| Some(b[j]),
        )
        .0
    }

    #[test]
    fn aligns_insert_delete_and_change() {
        let steps = run(b"abcdef", b"abXdeYf");
        use Step::*;
        let eqs = steps.iter().filter(|s| matches!(s, Equal(..))).count();
        assert_eq!(eqs, 5, "{steps:?}");
        assert!(steps.contains(&Delete(2)));
        assert!(steps.contains(&Insert(2)));
        assert!(steps.contains(&Insert(5)));
        assert_eq!(steps.first(), Some(&Equal(0, 0)));
        assert_eq!(steps.last(), Some(&Equal(5, 6)));
    }

    #[test]
    fn empty_sides() {
        assert_eq!(run(b"", b"ab"), vec![Step::Insert(0), Step::Insert(1)]);
        assert_eq!(run(b"ab", b""), vec![Step::Delete(0), Step::Delete(1)]);
    }

    #[test]
    fn lis_picks_ordered_anchors() {
        let p = [(0, 5), (1, 1), (2, 2), (3, 0), (4, 3)];
        assert_eq!(longest_increasing(&p), vec![(1, 1), (2, 2), (4, 3)]);
    }
}
