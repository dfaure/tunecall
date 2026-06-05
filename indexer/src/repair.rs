//! Repair *grossly* wrong OCR'd page numbers.
//!
//! A fake-book index is alphabetical, and the book is only *roughly* in that
//! same order, so page numbers trend upward but with real small inversions
//! (e.g. ALWAYS on page 23 listed right after ALRIGHT on page 24). We must NOT
//! force monotonicity, or we'd corrupt that real data.
//!
//! Instead we find the dominant upward trend — the longest non-decreasing run of
//! in-range OCR'd pages ("anchors") — and only replace a value when it deviates
//! from the interpolated trend by more than `tolerance`, or is out of range.
//! Everything within tolerance (including genuine small inversions) is left as
//! OCR'd.

/// Correct gross page-number outliers in `raw` (reading order). `min`/`max`
/// bound plausible pages; deviations up to `tolerance` from the trend are kept.
/// Returns the corrected pages and how many were changed.
pub fn repair_pages(raw: &[i32], min: i32, max: i32, tolerance: i32) -> (Vec<i32>, usize) {
    let n = raw.len();
    if n == 0 {
        return (Vec::new(), 0);
    }
    let in_range = |p: i32| p >= min && p <= max;

    // Longest non-decreasing run over in-range entries = the dominant trend.
    // On ties prefer the tightest span so a lone spike isn't mistaken for trend.
    let mut len = vec![0usize; n];
    let mut start = vec![0i32; n];
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
            if cand > len[i] || (cand == len[i] && start[j] > start[i]) {
                len[i] = cand;
                start[i] = start[j];
                prev[i] = j;
            }
        }
    }
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
        let expected = match (before, after) {
            (Some(a), Some(b)) => {
                let (pa, pb) = (out[a], out[b]);
                pa + ((pb - pa) as f64 * (i - a) as f64 / (b - a) as f64).round() as i32
            }
            (Some(a), None) => out[a] + (i - a) as i32,
            (None, Some(b)) => out[b] - (b - i) as i32,
            (None, None) => raw[i],
        }
        .clamp(min, max);

        // Only correct gross outliers; keep genuine small inversions as OCR'd.
        let gross = !in_range(raw[i]) || (raw[i] - expected).abs() > tolerance;
        if gross && expected != raw[i] {
            out[i] = expected;
            changed += 1;
        }
    }
    (out, changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(raw: &[i32]) -> Vec<i32> {
        repair_pages(raw, 1, 500, 20).0
    }

    #[test]
    fn leaves_clean_sequences_untouched() {
        let (out, n) = repair_pages(&[10, 11, 13, 14, 16], 1, 500, 20);
        assert_eq!(out, vec![10, 11, 13, 14, 16]);
        assert_eq!(n, 0);
    }

    #[test]
    fn keeps_real_small_inversions() {
        // ALRIGHT 24, ALWAYS 23, ANA MARIA 26 — the 23 is real, must be kept.
        assert_eq!(fix(&[24, 23, 26]), vec![24, 23, 26]);
        // A larger-but-still-small inversion within tolerance is also kept.
        assert_eq!(fix(&[100, 88, 102]), vec![100, 88, 102]);
    }

    #[test]
    fn fixes_gross_spikes() {
        assert_eq!(fix(&[37, 358, 39]), vec![37, 38, 39]); // off by ~320
        assert_eq!(fix(&[120, 184, 122, 124]), vec![120, 121, 122, 124]); // off by ~60
    }

    #[test]
    fn fixes_out_of_range_regardless_of_tolerance() {
        assert_eq!(fix(&[10, 0, 12]), vec![10, 11, 12]); // 0 < min
        assert_eq!(fix(&[10, 11, 999]), vec![10, 11, 12]); // 999 > max
    }

    #[test]
    fn reports_only_gross_changes() {
        // one gross (358) + one real inversion (23): only the gross is counted.
        let (out, n) = repair_pages(&[24, 23, 26, 358, 40], 1, 500, 20);
        assert_eq!(n, 1);
        assert_eq!(out[1], 23); // inversion kept
        assert_ne!(out[3], 358); // spike fixed
    }
}
