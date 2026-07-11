//! Differential parity: the Rust RMG/terrain port must reproduce, byte for
//! byte, the dumps produced by the UNMODIFIED Raven C++ terrain/RMG TUs compiled
//! by `tools/rmg-oracle/build.sh` (goldens under `tools/rmg-oracle/golden/`).
//! The dump format mirrors `tools/rmg-oracle/main.cpp` exactly.
//!
//! The oracle emits two committed goldens (`README.md` — #2/#3 are dropped
//! under `-DDEDICATED`):
//!
//! * **`golden/seed.txt` (golden #1)** — the platform-width `holdrand` LCG
//!   substrate the design pins "via `EngineHost::flrand`/`irand`" (RMG-D4f).
//!   [`seed_matches_oracle_golden`] reproduces it through the frozen host seam
//!   (`mp_host_interface::mock::MockHost`, ruling 32 — the reusable goldens
//!   vehicle whose inlined `holdrand` replica is the referee-proven bit-exact
//!   generator), mirroring `main.cpp`'s `dumpSeed` character for character.
//!
//! * **`golden/dedicated.txt` (golden #4)** — the dedicated-server outcome:
//!   the `CmLandScape` ctor + `LoadTerrainDef` GP2 parse, the `RmManager`
//!   `LoadMission` early-out, and the ctor error path. **NOT reproducible from
//!   this integration test under the FROZEN skeleton API** — see the
//!   module-level note on `dedicated_matches_oracle_golden` below and the
//!   returned `problems`. It is intentionally left as a reported blocker rather
//!   than a weakened/partial assertion.
//!
//! Fixtures/goldens are read from `tools/rmg-oracle/` and are never edited; the
//! Rust dump format matches the dumper's `printf`s character for character.

use std::fmt::Write as _;
use std::os::raw::c_ulong;
use std::path::PathBuf;

use mp_host_interface::mock::MockHost;
use mp_host_interface::EngineHost;

/// Repo-relative `tools/rmg-oracle` root (this crate is `crates/mp/engine/rmg`).
fn oracle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/rmg-oracle")
}

// ---------------------------------------------------------------------------
// Golden #1: the holdrand LCG substrate (mirrors main.cpp `dumpSeed`)
// ---------------------------------------------------------------------------

/// Reproduce C `printf("%.*g", prec, x)` for the values `dumpSeed` streams.
///
/// C `%g` (C99 §7.19.6.1): render with `%e` at precision `P-1` to find the
/// decimal exponent `X`; if `-4 <= X < P` use `%f` with precision `P-1-X`,
/// otherwise `%e` with precision `P-1`; then strip trailing fractional zeros
/// and a bare decimal point (no `#` flag). Rust's `{:.*e}` does the same
/// correctly-rounded normalization C's `%e` does (a carry into a new exponent,
/// e.g. `9.99e8 -> 1.00000000e9`, is preserved), so the mantissa/exponent are
/// read back from it rather than re-derived.
fn c_format_g(x: f64, precision: usize) -> String {
    let p = precision.max(1);
    if x == 0.0 {
        return "0".to_string();
    }
    let neg = x.is_sign_negative();
    let m = x.abs();

    // `%e` with P-1 fractional digits: "d.ddddddddeE" (E = decimal exponent).
    let e_str = format!("{:.*e}", p - 1, m);
    let (mant, exp_s) = e_str.split_once('e').expect("LowerExp always emits 'e'");
    let exp: i32 = exp_s.parse().expect("LowerExp exponent is an integer");
    // The P significant digits (drop the '.'): exactly `p` characters.
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();

    let mut body = if exp >= -4 && exp < p as i32 {
        f_style(&digits, exp)
    } else {
        e_style(&digits, exp)
    };
    if neg {
        body.insert(0, '-');
    }
    body
}

/// `%g`'s `%f`-style rendering of `p` significant `digits` scaled by `10^exp`,
/// with trailing fractional zeros and a bare decimal point stripped.
fn f_style(digits: &str, exp: i32) -> String {
    let mut s = if exp >= 0 {
        let split = (exp + 1) as usize;
        if split >= digits.len() {
            // Integer value; the digit run is entirely the integer part.
            digits.to_string()
        } else {
            format!("{}.{}", &digits[..split], &digits[split..])
        }
    } else {
        // 0.00…digits : `-exp-1` leading fractional zeros before the digits.
        let zeros = "0".repeat((-exp - 1) as usize);
        format!("0.{zeros}{digits}")
    };
    strip_fraction(&mut s);
    s
}

/// `%g`'s `%e`-style rendering: one leading digit, the rest fractional (zeros
/// stripped), then `e±dd` (exponent sign always shown, at least two digits).
fn e_style(digits: &str, exp: i32) -> String {
    let mut mant = format!("{}.{}", &digits[..1], &digits[1..]);
    strip_fraction(&mut mant);
    let sign = if exp < 0 { '-' } else { '+' };
    format!("{mant}e{sign}{:02}", exp.abs())
}

/// Drop trailing zeros from a decimal fraction, then a bare trailing `.`.
fn strip_fraction(s: &mut String) {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

/// Reproduce `main.cpp`'s `dumpSeed` (golden #1): the deterministic
/// `irand(4,9)`/`irand(0,100)`/`irand(0,255)`/`flrand(-1,1)`/`flrand(0,2)` draw
/// sequence over four seeds, with the raw `holdrand` state printed after each
/// draw. Drives `MockHost` (ruling 32) — the same seam `RMG_CreateSeed` and the
/// ctor seed consume — through `rand_init`/`irand`/`flrand`/`rng_state`.
fn dump_seed() -> String {
    let mut out = String::new();
    // `printf("== holdrand LCG substrate (platform-width c_ulong; RMG-D4f) ==\n")`.
    out.push_str("== holdrand LCG substrate (platform-width c_ulong; RMG-D4f) ==\n");
    // `printf("sizeof(unsigned long)=%d\n", (int)sizeof(unsigned long))`.
    writeln!(
        out,
        "sizeof(unsigned long)={}",
        core::mem::size_of::<c_ulong>()
    )
    .unwrap();

    // `static const unsigned seeds[] = { 0x89abcdefu, 1, 42, 1234567 };`.
    const SEEDS: [u32; 4] = [0x89ab_cdef, 1, 42, 1_234_567];

    let mut host = MockHost::new();
    for seed in SEEDS {
        // `Rand_Init((int)seeds[s])` then the `-- seed 0x%08x  state=0x%016lx --`
        // header ((unsigned)seed for the label, (int)seed for the re-seed).
        host.rand_init(seed as i32);
        writeln!(
            out,
            "-- seed 0x{seed:08x}  state=0x{:016x} --",
            host.rng_state()
        )
        .unwrap();

        for i in 0..6 {
            let a = host.irand(4, 9);
            let sa = host.rng_state();
            let b = host.irand(0, 100);
            let sb = host.rng_state();
            let c = host.irand(0, 255);
            let sc = host.rng_state();
            let d = host.flrand(-1.0, 1.0);
            let sd = host.rng_state();
            let e = host.flrand(0.0, 2.0);
            let se = host.rng_state();
            // The dumper's single wide `printf`, `%.9g` on the two floats
            // (each promoted `float -> double`, exactly `d as f64`).
            writeln!(
                out,
                "  [{i}] irand4_9={a} s=0x{sa:016x} | irand0_100={b} s=0x{sb:016x} | \
                 irand0_255={c} s=0x{sc:016x} | flrandm1_1={} s=0x{sd:016x} | \
                 flrand0_2={} s=0x{se:016x}",
                c_format_g(d as f64, 9),
                c_format_g(e as f64, 9),
            )
            .unwrap();
        }
    }
    // `printf("== end ==\n")`.
    out.push_str("== end ==\n");
    out
}

#[test]
fn seed_matches_oracle_golden() {
    let golden_path = oracle_root().join("golden").join("seed.txt");
    let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!("missing golden {golden_path:?} — run tools/rmg-oracle/build.sh --regen")
    });

    assert_eq!(
        dump_seed(),
        golden,
        "holdrand LCG substrate diverges from the terrain/RMG C++ oracle"
    );
}

// ---------------------------------------------------------------------------
// Golden #4: the dedicated-server outcome (mirrors main.cpp `dumpDedicated`)
// ---------------------------------------------------------------------------
//
// `golden/dedicated.txt` is NOT reproducible from a `mp_engine_rmg` integration
// test against the FROZEN skeleton API, for three independent reasons — all in
// the frozen surface, none fixable from a test:
//
//   1. **`CollisionWorld` is unconstructable outside `mp_engine_qcommon`.** It
//      has a private `_private: ()` field and no `pub` constructor / `Default`
//      (only `Engine::new` builds one). Every reachable path into golden #4 —
//      `register_terrain(&mut CollisionWorld, …)`, the `terrain_*` accessors,
//      `RmManager::load_mission(&mut CollisionWorld, …)` — needs a `&mut
//      CollisionWorld`, so the whole dump cannot even begin.
//   2. **The dumped internal state has no `pub` accessor.** `dumpDedicated`
//      prints `GetWidth`/`GetHeight`/`GetBlockWidth`/`GetBlockHeight`/
//      `GetBlockCount`/`GetTerxels` (the `dims:` line), `GetBaseWaterHeight`
//      (the `water:` line), and `GetSurfaceFlags(h)`/`GetContentFlags(h)` (the
//      `altitude flags` block). On the port these are `pub(crate)`/private
//      fields of `CmLandScape` (RMG-D4h), invisible to an external test crate —
//      only the live snapshot reads (`flatten_map`/`real_area`/`get_rand_seed`)
//      and the `CollisionWorld` water/patch-scalar forwarders are `pub`.
//   3. **The trailing `... Shutting down TheRandomMissionManager` line** is
//      emitted by `~CRMManager` (`RM_Manager.cpp:56`); the frozen `RmManager`
//      has no `Drop`/host seam, so it is not reproducible.
//
// Weakening the assertion to a reachable subset, or fabricating the unreachable
// lines, is disallowed (never weaken; goldens are byte-for-byte). This gap is
// reported back for a design ruling (widen the frozen API with `pub` terrain
// accessors + a `CollisionWorld` test/construction seam, or relocate golden #4
// into a `mp_engine_qcommon` in-crate test that can see `pub(crate)` state and
// still reach `RmManager` — which a qcommon test cannot, since `rmg` sits above
// `qcommon`). No `dedicated_matches_oracle_golden` test is emitted until then.
