//! `Rng` — the faithful LCG (Raven's VC-libc `rand()` clone).
//!
//! Raven kept the generator in a single file-static `holdrand` shared by the
//! whole game+bg tier (`q_math.c`). This reproduces
//! the LCG bit-exactly as an owned, threaded generator living in `BgState`;
//! game reaches it via `world.bg_state.rng`. Never the `rand` crate — every
//! `Q_flrand`/`Q_irand` site is parity-visible.
//!
//! Source: `oracle/oracle/codemp/game/q_math.c:1425-1474`
#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint};

/// Raven's `holdrand` seed plus the VC-libc `rand()` LCG. On the shipping
/// 32-bit `jampded` target `unsigned long` is 32-bit, so the state is a `u32`
/// and every step uses wrapping arithmetic to reproduce the truncation exactly.
///
/// Raven kept two independent generator states — this file-static `holdrand`
/// (`q_math.c:1432`) and `bg_lib.c`'s file-static `randSeed` (`bg_lib.c:763`)
/// — that never shared state; both are kept threaded here.
///
/// Source: `oracle/oracle/codemp/game/q_math.c:1432`
pub struct Rng {
    /// Raven `static unsigned long holdrand = 0x89abcdef;`.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1432`
    holdrand: u32,

    /// Raven `bg_lib.c`'s `static int randSeed = 0;` — the independent LCG
    /// state backing `bg_lib.c`'s `rand`/`srand` and the `q_shared.h`
    /// `random`/`crandom` macros.
    /// Source: `oracle/oracle/codemp/game/bg_lib.c:763`
    randSeed: u32,
}

impl Rng {
    /// The initial `holdrand` value (`0x89abcdef`) — the state a fresh zeroed
    /// `BgState` must carry, so this is what `Default`/`new` install.
    /// Source: `oracle/oracle/codemp/game/q_math.c:1432`
    const HOLDRAND_INIT: u32 = 0x89ab_cdef;

    /// Fresh generator seeded with Raven's compile-time `holdrand` value and
    /// `bg_lib.c`'s compile-time `randSeed = 0`.
    pub fn new() -> Self {
        Self {
            holdrand: Self::HOLDRAND_INIT,
            randSeed: 0,
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

    /// Raven `bg_lib.c`'s `srand` — (re)seeds the independent `randSeed` LCG.
    /// Source: `oracle/oracle/codemp/game/bg_lib.c:765-767`
    pub fn srand(&mut self, seed: c_uint) {
        self.randSeed = seed as u32;
    }

    /// Raven `bg_lib.c`'s `rand` — `randSeed = 69069*randSeed + 1; return
    /// randSeed & 0x7fff;`.
    /// Source: `oracle/oracle/codemp/game/bg_lib.c:769-772`
    pub fn rand(&mut self) -> c_int {
        self.randSeed = 69069u32.wrapping_mul(self.randSeed).wrapping_add(1);
        (self.randSeed & 0x7fff) as c_int
    }

    /// Raven `random()` macro — `(rand() & 0x7fff) / ((float)0x7fff)`.
    /// Source: `oracle/oracle/codemp/game/q_shared.h:1591`
    pub fn random(&mut self) -> f32 {
        ((self.rand() & 0x7fff) as f32) / (0x7fff as f32)
    }

    /// Raven `crandom()` macro — `2.0 * (random() - 0.5)`.
    /// Source: `oracle/oracle/codemp/game/q_shared.h:1592`
    pub fn crandom(&mut self) -> f32 {
        2.0 * (self.random() - 0.5)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

// Parity is bit-exact ONLY for a 32-bit `unsigned long` (the shipping
// `jampded`/i686 target). `holdrand` is modelled as `u32` and every step uses
// wrapping arithmetic to reproduce the truncation; this assert pins that the
// state width has not silently widened. On an LP64 host `unsigned long` is
// 64-bit, so the faithful `>> 17` masking would diverge — the `u32` model is
// what keeps host `cargo check` and the boot target in agreement.
// Source: `oracle/oracle/codemp/game/q_math.c:1432` (holdrand*214013+2531011 >>17)
const _: () = assert!(
    core::mem::size_of::<u32>() == 4,
    "LCG parity requires a 32-bit holdrand state"
);
