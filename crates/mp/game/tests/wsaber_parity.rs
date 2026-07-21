//! Differential parity test for two pure integer `w_saber.c` leaf functions —
//! `G_SaberLockAnim` and `G_KnockawayForParry` — against the Raven oracle.
//! Reproduces `tests/oracle/golden/wsaber.txt` (generated exclusively
//! by `main_wsaber.c` over the committed `fixtures/wsaber/` sweep bounds) by
//! driving the PORTED functions over the identical integer sweeps and
//! byte-comparing to the committed golden.
//!
//! Both functions are pure integer switch/table logic over their arguments (no
//! state, no RNG, no floats), so one `#[test]` builds the whole dump and
//! compares once. The golden is committed, so `cargo test` needs no C toolchain.
#![allow(non_snake_case)]

use std::fmt::Write as _;
use std::path::PathBuf;

use mp_game::w_saber::{G_KnockawayForParry, G_SaberLockAnim};
use testkit::{compare, oracle_dir, pi};

fn fixtures_dir() -> PathBuf {
    oracle_dir(env!("CARGO_MANIFEST_DIR")).join("fixtures/wsaber")
}

// Read the single `sweep ...` line's tokens from a fixture (mirrors
// main_wsaber.c read_sweep).
fn read_sweep(fname: &str) -> Vec<i32> {
    let p = fixtures_dir().join(fname);
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
    for line in text.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() || tok[0].starts_with('#') || tok[0] != "sweep" {
            continue;
        }
        return tok[1..].iter().map(|t| pi(t)).collect();
    }
    panic!("no sweep line in {fname}");
}

fn sec_lockanim(o: &mut String) {
    o.push_str("== lockanim ==\n");
    let b = read_sweep("lockanim.txt");
    let (s_lo, s_hi, t_lo, t_hi, l_lo, l_hi, w_lo, w_hi) =
        (b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]);
    for a in s_lo..=s_hi {
        for d in s_lo..=s_hi {
            for t in t_lo..=t_hi {
                for l in l_lo..=l_hi {
                    for w in w_lo..=w_hi {
                        let r = G_SaberLockAnim(a, d, t, l, w);
                        let _ = writeln!(o, "la {a} {d} {t} {l} {w} {r}");
                    }
                }
            }
        }
    }
}

fn sec_knockaway(o: &mut String) {
    o.push_str("== knockaway ==\n");
    let b = read_sweep("knockaway.txt");
    let (lo, hi) = (b[0], b[1]);
    for m in lo..=hi {
        let r = G_KnockawayForParry(m);
        let _ = writeln!(o, "kp {m} {r}");
    }
}

#[test]
fn wsaber_parity() {
    let mut o = String::new();
    o.push_str("== wsaber ==\n");
    sec_lockanim(&mut o);
    sec_knockaway(&mut o);
    o.push_str("== end ==\n");
    compare(env!("CARGO_MANIFEST_DIR"), "wsaber", &o);
}
