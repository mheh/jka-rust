//! Raven `qsort` (`bg_lib.c`'s verbatim copy of the BSD Bentley-McIlroy
//! quicksort), transcribed index-faithfully over a slice: the tie permutation
//! — which elements comparing equal end up where — is identical to the C
//! body, because the algorithm observes only comparisons and whole-element
//! swaps (the C `swaptype` byte/long chunking is a perf detail invisible to
//! ordering). Pinned against the C ground truth by the `bglib` golden in
//! `jampgame_parity.rs`.
//!
//! Linkage note (2026-07-19): retail's JK2_game.vcproj excludes bg_lib.c from
//! the native DLL, so retail bound **msvcrt's** qsort — a different unstable
//! sort whose tie order can differ. The as-built referee oracle binds Darwin
//! libc (same BSD lineage as this body). Whether to reproduce msvcrt's tie
//! order instead is an open ruling; until then this is the one canonical
//! body.
//!
//! Source: `oracle/codemp/game/bg_lib.c:80-181`

use core::ffi::c_int;

/// Raven `med3` — median-of-three pivot selection, by index.
/// Source: `oracle/codemp/game/bg_lib.c:98-103`
fn med3<T>(
    a: &mut [T],
    x: usize,
    y: usize,
    z: usize,
    cmp: &mut impl FnMut(&T, &T) -> c_int,
) -> usize {
    if cmp(&a[x], &a[y]) < 0 {
        if cmp(&a[y], &a[z]) < 0 {
            y
        } else if cmp(&a[x], &a[z]) < 0 {
            z
        } else {
            x
        }
    } else if cmp(&a[y], &a[z]) > 0 {
        y
    } else if cmp(&a[x], &a[z]) < 0 {
        x
    } else {
        z
    }
}

/// Swap the `len`-element blocks starting at `x` and `y` (the C "swap
/// vectors" step, element-wise).
fn swap_block<T>(a: &mut [T], x: usize, y: usize, len: usize) {
    for k in 0..len {
        a.swap(x + k, y + k);
    }
}

/// Insertion sort over `a[lo..lo+n]` (the C small-array / no-swaps path).
fn insertion<T>(a: &mut [T], lo: usize, n: usize, cmp: &mut impl FnMut(&T, &T) -> c_int) {
    for pm in lo + 1..lo + n {
        let mut pl = pm;
        while pl > lo && cmp(&a[pl - 1], &a[pl]) > 0 {
            a.swap(pl, pl - 1);
            pl -= 1;
        }
    }
}

/// Raven `qsort` — Bentley-McIlroy three-way-partition quicksort: insertion
/// sort under 7 elements, median-of-3 (of-9 over 40) pivot, equal-run
/// collection at both ends swapped to the middle, recursion on the left
/// partition and iteration on the right.
///
/// Source: `oracle/codemp/game/bg_lib.c:105-181`
pub fn qsort<T, F: FnMut(&T, &T) -> c_int>(a: &mut [T], mut cmp: F) {
    let n = a.len();
    qsort_span(a, 0, n, &mut cmp);
}

fn qsort_span<T>(a: &mut [T], mut lo: usize, mut n: usize, cmp: &mut impl FnMut(&T, &T) -> c_int) {
    loop {
        let mut swap_cnt = false;

        if n < 7 {
            insertion(a, lo, n, cmp);
            return;
        }

        let mut pm = lo + n / 2;
        if n > 7 {
            let mut pl = lo;
            let mut pn = lo + n - 1;
            if n > 40 {
                let d = n / 8;
                pl = med3(a, pl, pl + d, pl + 2 * d, cmp);
                pm = med3(a, pm - d, pm, pm + d, cmp);
                pn = med3(a, pn - 2 * d, pn - d, pn, cmp);
            }
            pm = med3(a, pl, pm, pn, cmp);
        }

        // Pivot to a[lo]; pa/pb walk up, pc/pd walk down (indices).
        a.swap(lo, pm);
        let mut pa = lo + 1;
        let mut pb = pa;
        let mut pc = lo + n - 1;
        let mut pd = pc;

        loop {
            while pb <= pc {
                let r = cmp(&a[pb], &a[lo]);
                if r > 0 {
                    break;
                }
                if r == 0 {
                    swap_cnt = true;
                    a.swap(pa, pb);
                    pa += 1;
                }
                pb += 1;
            }
            while pb <= pc {
                let r = cmp(&a[pc], &a[lo]);
                if r < 0 {
                    break;
                }
                if r == 0 {
                    swap_cnt = true;
                    a.swap(pc, pd);
                    pd -= 1;
                }
                pc -= 1;
            }
            if pb > pc {
                break;
            }
            a.swap(pb, pc);
            swap_cnt = true;
            pb += 1;
            pc -= 1;
        }

        if !swap_cnt {
            insertion(a, lo, n, cmp);
            return;
        }

        let pn = lo + n;
        // Move the equal-to-pivot runs from the ends into the middle.
        let r = (pa - lo).min(pb - pa);
        if r > 0 {
            swap_block(a, lo, pb - r, r);
        }
        let r = (pd - pc).min(pn - pd - 1);
        if r > 0 {
            swap_block(a, pb, pn - r, r);
        }

        // Recurse on the left partition, iterate on the right (the C tail
        // loop).
        let left = pb - pa;
        if left > 1 {
            qsort_span(a, lo, left, cmp);
        }
        let right = pd - pc;
        if right > 1 {
            lo = pn - right;
            n = right;
            continue;
        }
        return;
    }
}

#[cfg(test)]
mod qsort_tests {
    use super::qsort;

    #[test]
    fn sorts_ints() {
        let mut v = [5, 3, 9, 1, 1, 8, 0, -4, 7, 2, 6, 6];
        qsort(&mut v, |a, b| a - b);
        assert_eq!(v, [-4, 0, 1, 1, 2, 3, 5, 6, 6, 7, 8, 9]);
    }

    #[test]
    fn small_array_insertion_path() {
        let mut v = [3, 1, 2];
        qsort(&mut v, |a, b| a - b);
        assert_eq!(v, [1, 2, 3]);
    }

    #[test]
    fn already_sorted_no_swap_path() {
        let mut v: Vec<i32> = (0..50).collect();
        let expect = v.clone();
        qsort(&mut v, |a, b| a - b);
        assert_eq!(v, expect);
    }

    #[test]
    fn large_with_ties_matches_std_by_key() {
        // Order-of-ties differs from std's stable sort, but the multiset and
        // key-sortedness must hold; the exact tie permutation is pinned by
        // the bglib golden against the C body.
        let mut v: Vec<i32> = (0..1000).map(|i| (i * 7919) % 100).collect();
        let mut expect = v.clone();
        qsort(&mut v, |a, b| a - b);
        expect.sort();
        assert_eq!(v, expect);
    }
}
