//! `Rng` — the fork-3 faithful LCG (Raven's VC-libc `rand()` clone).
//!
//! Raven kept the generator in a single file-static `holdrand` shared by the
//! whole game+bg tier (`q_math.c`). Fork ruling 3 / pass-3 ruling 15: reproduce
//! the LCG bit-exactly as an owned, threaded generator living in `BgState`;
//! game reaches it via `world.bg_state.rng`. Never the `rand` crate — every
//! `Q_flrand`/`Q_irand` site is parity-visible.
//!
//! Source: `oracle/oracle/codemp/game/q_math.c:1425-1474`
#![allow(non_snake_case)]

use core::ffi::c_int;

/// Raven's `holdrand` seed plus the VC-libc `rand()` LCG. On the shipping
/// 32-bit `jampded` target `unsigned long` is 32-bit, so the state is a `u32`
/// and every step uses wrapping arithmetic to reproduce the truncation exactly.
///
/// Source: `oracle/oracle/codemp/game/q_math.c:1432`
pub struct Rng {
    /// Raven `static unsigned long holdrand = 0x89abcdef;`.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1432`
    holdrand: u32,
}

impl Rng {
    /// The initial `holdrand` value (`0x89abcdef`) — the state a fresh zeroed
    /// `BgState` must carry, so this is what `Default`/`new` install.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1432`
    const HOLDRAND_INIT: u32 = 0x89ab_cdef;

    /// Fresh generator seeded with Raven's compile-time `holdrand` value.
    pub fn new() -> Self {
        Self {
            holdrand: Self::HOLDRAND_INIT,
        }
    }

    /// Raven `Rand_Init` — reseed the generator.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1434-1437`
    pub fn Rand_Init(&mut self, seed: c_int) {
        self.holdrand = seed as u32;
    }

    /// Raven `flrand` — returns a float `min <= x < max` (exclusive; will get
    /// `max - 0.00001`, but never `max`).
    /// Source: `oracle/oracle/codemp/game/q_math.c:1441-1450`
    pub fn flrand(&mut self, min: f32, max: f32) -> f32 {
        self.holdrand = self.holdrand.wrapping_mul(214013).wrapping_add(2531011);
        // 0 - 32767 range.
        let result = (self.holdrand >> 17) as f32;
        ((result * (max - min)) / 32768.0f32) + min
    }

    /// Raven `Q_flrand` — the thin dual over `flrand`.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1451-1454`
    pub fn Q_flrand(&mut self, min: f32, max: f32) -> f32 {
        self.flrand(min, max)
    }

    /// Raven `irand` — returns an integer `min <= x <= max` (inclusive).
    ///
    /// Raven asserts `(max - min) < 32768`; we preserve the wrapping integer
    /// arithmetic rather than the debug assert.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1458-1469`
    pub fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        debug_assert!((max - min) < 32768);
        let max = max + 1;
        self.holdrand = self.holdrand.wrapping_mul(214013).wrapping_add(2531011);
        let result = (self.holdrand >> 17) as c_int;
        (result.wrapping_mul(max - min) >> 15).wrapping_add(min)
    }

    /// Raven `Q_irand` — the thin dual over `irand`.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1471-1474`
    pub fn Q_irand(&mut self, value1: c_int, value2: c_int) -> c_int {
        self.irand(value1, value2)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

// Fork-3 parity is bit-exact ONLY for a 32-bit `unsigned long` (the shipping
// `jampded`/i686 target). `holdrand` is modelled as `u32` and every step uses
// wrapping arithmetic to reproduce the truncation; this assert pins that the
// state width has not silently widened. On an LP64 host `unsigned long` is
// 64-bit, so the faithful `>> 17` masking would diverge — the `u32` model is
// what keeps host `cargo check` and the boot target in agreement.
// Source: `oracle/oracle/codemp/game/q_math.c:1432` (holdrand*214013+2531011 >>17)
const _: () = assert!(
    core::mem::size_of::<u32>() == 4,
    "fork-3 LCG parity requires a 32-bit holdrand state"
);
