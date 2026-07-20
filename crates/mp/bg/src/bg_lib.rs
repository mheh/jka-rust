// PORT-COMPLETE: bg_lib.c 8/8
//! `bg_lib.c` — standard library replacement routines for VM code.
//!
//! Raven's faithful implementations of standard C library functions,
//! primarily qsort, memmove, and string-to-number conversion.
//!
//! Raven source: `oracle/codemp/game/bg_lib.c`

use crate::prelude::*;

/// `cmp_t` — comparison function pointer.
///
/// Signature: `int (*)(const void *, const void *)`
/// Returns: negative if first < second, zero if equal, positive if first > second.
type CmpFn = extern "C" fn(*const c_void, *const c_void) -> c_int;

/// Raven `swapfunc`.
///
/// Swaps bytes in two memory regions, handling different alignments
/// and element sizes via the `swaptype` parameter.
/// - swaptype 0: long-aligned swaps
/// - swaptype 1: size-of(long) element swaps
/// - swaptype 2: byte-by-byte swaps
///
/// Source: `oracle/codemp/game/bg_lib.c:80-86`
pub fn swapfunc(a: *mut c_char, b: *mut c_char, n: c_int, swaptype: c_int) {
    let n_usize = n as usize;

    unsafe {
        if swaptype <= 1 {
            // Swap as long-sized elements
            let long_size = std::mem::size_of::<usize>();
            let mut i = n_usize / long_size;
            let mut pi = a as *mut usize;
            let mut pj = b as *mut usize;

            while i > 0 {
                let t = *pi;
                *pi = *pj;
                *pj = t;
                pi = pi.add(1);
                pj = pj.add(1);
                i -= 1;
            }
        } else {
            // Swap byte-by-byte
            let mut i = n_usize;
            let mut pi = a;
            let mut pj = b;

            while i > 0 {
                let t = *pi;
                *pi = *pj;
                *pj = t;
                pi = pi.add(1);
                pj = pj.add(1);
                i -= 1;
            }
        }
    }
}

/// Raven `med3`.
///
/// Selects the median of three values using the provided comparison function.
/// Used by qsort to choose a pivot element.
///
/// Source: `oracle/codemp/game/bg_lib.c:98-103`
pub fn med3(a: *mut c_char, b: *mut c_char, c: *mut c_char, cmp: *mut c_void) -> *mut c_char {
    unsafe {
        let cmp_fn = std::mem::transmute::<*mut c_void, CmpFn>(cmp);

        let ab = cmp_fn(a as *const c_void, b as *const c_void);
        if ab < 0 {
            let bc = cmp_fn(b as *const c_void, c as *const c_void);
            if bc < 0 {
                return b;
            }
            let ac = cmp_fn(a as *const c_void, c as *const c_void);
            if ac < 0 {
                c
            } else {
                a
            }
        } else {
            let bc = cmp_fn(b as *const c_void, c as *const c_void);
            if bc > 0 {
                return b;
            }
            let ac = cmp_fn(a as *const c_void, c as *const c_void);
            if ac < 0 {
                a
            } else {
                c
            }
        }
    }
}

/// Raven `qsort`.
///
/// Quicksort implementation using Bentley-McIlroy algorithm with three-way
/// partitioning. Uses insertion sort for small arrays and tail-recursion
/// optimization via a loop instead of recursive calls.
///
/// Source: `oracle/codemp/game/bg_lib.c:105-181`
pub fn qsort(a: *mut c_void, n: usize, es: usize, cmp: *mut c_void) {
    let cmp_fn = unsafe { std::mem::transmute::<*mut c_void, CmpFn>(cmp) };

    unsafe {
        let mut a = a as *mut c_char;
        let mut n = n;

        'loop_start: loop {
            // Determine swap type based on alignment and element size
            let swaptype = if (a as usize) % std::mem::size_of::<usize>() != 0
                || es % std::mem::size_of::<usize>() != 0
            {
                2
            } else if es == std::mem::size_of::<usize>() {
                0
            } else {
                1
            };

            let mut swap_cnt = 0;

            if n < 7 {
                // Insertion sort for small arrays
                let mut pm = a.add(es);
                while pm < a.add(n * es) {
                    let mut pl = pm;
                    while pl > a && cmp_fn(pl.sub(es) as *const c_void, pl as *const c_void) > 0 {
                        swapfunc(pl, pl.sub(es), es as c_int, swaptype);
                        pl = pl.sub(es);
                    }
                    pm = pm.add(es);
                }
                return;
            }

            let mut pm = a.add((n / 2) * es);

            if n > 7 {
                let mut pl = a;
                let mut pn = a.add((n - 1) * es);

                if n > 40 {
                    let d = (n / 8) * es;
                    pl = med3(pl, pl.add(d), pl.add(2 * d), cmp);
                    pm = med3(pm.sub(d), pm, pm.add(d), cmp);
                    pn = med3(pn.sub(2 * d), pn.sub(d), pn, cmp);
                }
                pm = med3(pl, pm, pn, cmp);
            }

            swapfunc(a, pm, es as c_int, swaptype);
            let mut pa = a.add(es);
            let mut pb = pa;
            let mut pc = a.add((n - 1) * es);
            let mut pd = pc;

            loop {
                while pb <= pc {
                    let r = cmp_fn(pb as *const c_void, a as *const c_void);
                    if r > 0 {
                        break;
                    }
                    if r == 0 {
                        swap_cnt = 1;
                        swapfunc(pa, pb, es as c_int, swaptype);
                        pa = pa.add(es);
                    }
                    pb = pb.add(es);
                }

                while pb <= pc {
                    let r = cmp_fn(pc as *const c_void, a as *const c_void);
                    if r < 0 {
                        break;
                    }
                    if r == 0 {
                        swap_cnt = 1;
                        swapfunc(pc, pd, es as c_int, swaptype);
                        pd = pd.sub(es);
                    }
                    pc = pc.sub(es);
                }

                if pb > pc {
                    break;
                }
                swapfunc(pb, pc, es as c_int, swaptype);
                swap_cnt = 1;
                pb = pb.add(es);
                pc = pc.sub(es);
            }

            if swap_cnt == 0 {
                // Switch to insertion sort if no swaps occurred (already sorted)
                let mut pm = a.add(es);
                while pm < a.add(n * es) {
                    let mut pl = pm;
                    while pl > a && cmp_fn(pl.sub(es) as *const c_void, pl as *const c_void) > 0 {
                        swapfunc(pl, pl.sub(es), es as c_int, swaptype);
                        pl = pl.sub(es);
                    }
                    pm = pm.add(es);
                }
                return;
            }

            let pn = a.add(n * es);

            // Calculate min(pa - a, pb - pa) using byte offsets
            let pa_offset = pa.offset_from(a) as usize;
            let pb_pa_offset = pb.offset_from(pa) as usize;
            let r = if pa_offset < pb_pa_offset {
                pa_offset
            } else {
                pb_pa_offset
            };

            if r > 0 {
                swapfunc(a, pb.sub(r), r as c_int, swaptype);
            }

            // Calculate min(pd - pc, pn - pd - es)
            let pd_pc_offset = pd.offset_from(pc) as usize;
            let pn_pd_es_offset = pn.offset_from(pd.add(es)) as usize;
            let r = if pd_pc_offset < pn_pd_es_offset {
                pd_pc_offset
            } else {
                pn_pd_es_offset
            };

            if r > 0 {
                swapfunc(pb, pn.sub(r), r as c_int, swaptype);
            }

            // Recursively sort left partition
            let r = pb.offset_from(pa) as usize;
            if r > es {
                qsort(a as *mut c_void, r / es, es, cmp);
            }

            // Tail recursion: iterate with right partition instead of recursing
            let r = pd.offset_from(pc) as usize;
            if r > es {
                a = pn.sub(r);
                n = r / es;
                continue 'loop_start;
            }
            return;
        }
    }
}

/// Raven `__builtin___memmove_chk`.
///
/// Memory move that handles overlapping regions correctly by copying
/// backwards when destination > source. The __builtin_object_size
/// parameter is ignored (used by compiler for bounds checking).
///
/// Source: `oracle/codemp/game/bg_lib.c:287-300`
pub fn __builtin___memmove_chk(
    dest: *mut c_void,
    src: *const c_void,
    count: usize,
    _builtin_object_size: *mut c_void,
) -> *mut c_void {
    unsafe {
        let dest_u8 = dest as *mut u8;
        let src_u8 = src as *const u8;

        if dest_u8 > src_u8 as *mut u8 {
            // Overlapping with dest > src: copy backwards to avoid overwriting source
            let mut i = count as isize - 1;
            while i >= 0 {
                *dest_u8.add(i as usize) = *src_u8.add(i as usize);
                i -= 1;
            }
        } else {
            // Non-overlapping or dest <= src: copy forwards
            for i in 0..count {
                *dest_u8.add(i) = *src_u8.add(i);
            }
        }
    }
    dest
}

// `srand`/`rand` are ported as the `randSeed` generator on
// `bg_channel::rng::Rng` (BgState) — reach them via
// `world.bg_state.rng.srand`/`.rand`.
// Source: oracle/codemp/game/bg_lib.c:763-772

/// Raven `atoi`.
///
/// Converts a string to an integer. Skips leading whitespace, honours an
/// optional sign, and reads base-10 digits until a non-digit. Canonical home
/// for the local `extern "C" { fn atoi }` / `atoi_cstr` shims that stood in
/// for the unported `bg_lib.c` function.
///
/// Source: `oracle/codemp/game/bg_lib.c:915-958`
///
/// This is the faithful Q3_VM-bytecode port; that `#if defined(Q3_VM)` span
/// is never compiled into the native game DLL, which links libc `atoi`
/// instead (`nm` shows `_atoi` U). Game-logic call sites use
/// `cstr_util::atoi` (via the prelude), not this fn directly.
pub fn atoi(string: *const c_char) -> c_int {
    unsafe {
        let mut string = string;

        // Skip whitespace. Oracle's `*string` is a signed char, so bytes
        // 0x80-0xFF sign-extend negative and satisfy `<= ' '` (skipped).
        loop {
            let ch = *string as i8 as i32;
            if ch > b' ' as i32 {
                break;
            }
            if ch == 0 {
                return 0;
            }
            string = string.add(1);
        }

        // Check sign
        let sign = match *string as u8 {
            b'+' => {
                string = string.add(1);
                1
            }
            b'-' => {
                string = string.add(1);
                -1
            }
            _ => 1,
        };

        // Read digits
        let mut value: c_int = 0;
        loop {
            let c = *string as u8;
            string = string.add(1);
            if c < b'0' || c > b'9' {
                break;
            }
            // C's `int value = value*10 + c` wraps on overflow (2's complement);
            // Rust's `*`/`+` panic in debug. Use wrapping to reproduce Raven's
            // behavior for out-of-range inputs (e.g. "99999999999").
            // Source: `oracle/codemp/game/bg_lib.c:948-953`
            value = value.wrapping_mul(10).wrapping_add((c - b'0') as c_int);
        }

        value.wrapping_mul(sign)
    }
}

// bg_lib `atof`/`_atof` are §20-dropped (2026-07-19): retail's JK2_game.vcproj
// excludes bg_lib.c from the Release/Final native DLL builds (only QVM and
// Raven's Debug config ever compiled them; OpenJK drops the file too), so game
// call sites bind libc `atof` — `native_string::atof`, reached through the
// `cstr_util::atof` pointer seam.
// Source: `oracle/codemp/game/bg_lib.c:774-907`; `JK2_game.vcproj:405-417`.
