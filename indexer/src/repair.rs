//! Repair OCR'd page numbers using the structure of a fake-book index: it is
//! alphabetical, so the pages must increase (non-decreasing) in reading order.
//!
//! Strategy: take the longest non-decreasing subsequence of the OCR'd pages as
//! "anchors" (the values that agree with each other), and replace everything
//! else by linear interpolation between the surrounding anchors. So `37, <junk>,
//! 39` becomes `38`, and `37, <junk>, 40` becomes ~`38` (off by one beats junk).

/// Repair `raw` pages (in reading order) toward a non-decreasing sequence.
/// Values outside `[min, max]` are never trusted as anchors. Returns the
/// corrected pages and how many entries changed.
pub fn repair_monotonic(raw: &[i32], min: i32, max: i32) -> (Vec<i32>, usize) {
    let n = raw.len();
    if n == 0 {
        return (Vec::new(), 0);
    }

    let in_range = |p: i32| p >= min && p <= max;

    // Longest non-decreasing subsequence over in-range entries (O(n^2), n small).
    // Among equal-length runs we prefer the *tightest* (smallest value span), so a
    // lone spike like [37, 358, 39] keeps {37,39} rather than {37,358}.
    let mut len = vec![0usize; n];
    let mut start = vec![0i32; n]; // first value on the best path ending at i
    let mut prev = vec![usize::MAX; n];
    for i in 0..n {
        if !in_range(raw[i]) {
            continue;
        }
        len[i] = 1;
        start[i] = raw[i];
        for j in 0..i {
            if !in_range(raw[j]) || raw[j] > raw[i] {
                continue;
            }
            let cand = len[j] + 1;
            // longer wins; on a tie, the larger start value gives a tighter span
            if cand > len[i] || (cand == len[i] && start[j] > start[i]) {
                len[i] = cand;
                start[i] = start[j];
                prev[i] = j;
            }
        }
    }

    // Best run: longest, breaking ties toward the smallest span (raw[i] - start[i]).
    let mut best = usize::MAX;
    for i in 0..n {
        if len[i] == 0 {
            continue;
        }
        if best == usize::MAX
            || len[i] > len[best]
            || (len[i] == len[best] && raw[i] - start[i] < raw[best] - start[best])
        {
            best = i;
        }
    }

    let mut is_anchor = vec![false; n];
    let mut cur = best;
    while cur != usize::MAX {
        is_anchor[cur] = true;
        cur = prev[cur];
    }

    let anchors: Vec<usize> = (0..n).filter(|&i| is_anchor[i]).collect();
    if anchors.is_empty() {
        return (raw.to_vec(), 0); // nothing trustworthy; leave as-is
    }

    let mut out = raw.to_vec();
    let mut changed = 0;
    for i in 0..n {
        if is_anchor[i] {
            continue;
        }
        let before = anchors.iter().rev().find(|&&a| a < i).copied();
        let after = anchors.iter().find(|&&a| a > i).copied();
        let value = match (before, after) {
            (Some(a), Some(b)) => {
                let (pa, pb) = (out[a], out[b]);
                pa + ((pb - pa) as f64 * (i - a) as f64 / (b - a) as f64).round() as i32
            }
            (Some(a), None) => out[a] + (i - a) as i32, // trailing: assume +1/entry
            (None, Some(b)) => out[b] - (b - i) as i32, // leading: count back
            (None, None) => raw[i],
        };
        let value = value.clamp(min, max);
        if value != raw[i] {
            changed += 1;
        }
        out[i] = value;
    }
    (out, changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(raw: &[i32]) -> Vec<i32> {
        repair_monotonic(raw, 1, 500).0
    }

    #[test]
    fn leaves_clean_sequences_untouched() {
        let (out, n) = repair_monotonic(&[10, 11, 13, 14, 16], 1, 500);
        assert_eq!(out, vec![10, 11, 13, 14, 16]);
        assert_eq!(n, 0);
    }

    #[test]
    fn fixes_spike_between_neighbors() {
        // 37, <garbage 358>, 39  ->  38
        assert_eq!(fix(&[37, 358, 39]), vec![37, 38, 39]);
        // out-of-range junk (0) between 10 and 12 -> 11
        assert_eq!(fix(&[10, 0, 12]), vec![10, 11, 12]);
    }

    #[test]
    fn fixes_local_inversion() {
        // 13, 14, 13, 16 : the stray 13 becomes 15 (interpolated 14..16)
        assert_eq!(fix(&[13, 14, 13, 16]), vec![13, 14, 15, 16]);
    }

    #[test]
    fn extrapolates_at_the_ends() {
        assert_eq!(fix(&[0, 11, 12]), vec![10, 11, 12]); // leading junk
        assert_eq!(fix(&[10, 11, 999]), vec![10, 11, 12]); // trailing junk (out of range)
    }

    #[test]
    fn reports_change_count() {
        let (_, n) = repair_monotonic(&[37, 358, 39], 1, 500);
        assert_eq!(n, 1);
    }
}
