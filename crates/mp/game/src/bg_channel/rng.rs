//! `Rng` — the faithful LCG (Raven's VC-libc `rand()` clone).
//!
//! Raven kept the generator in a single file-static `holdrand` shared by the
//! whole game+bg tier (`q_math.c`). This reproduces
//! the LCG bit-exactly as an owned, threaded generator living in `BgState`;
//! game reaches it via `world.bg_state.rng`. Never the `rand` crate — every
//! `Q_flrand`/`Q_irand` site is parity-visible.
//!
//! Source: `oracle/codemp/game/q_math.c:1425-1474`
#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_ulong};

/// Raven's `holdrand` seed plus the VC-libc `rand()` LCG.
///
/// Raven kept two independent generator states — this file-static `holdrand`
/// (`q_math.c:1432`) and `bg_lib.c`'s file-static `randSeed` (`bg_lib.c:763`)
/// — that never shared state; both are kept threaded here.
///
/// Source: `oracle/codemp/game/q_math.c:1432`
pub struct Rng {
    /// Raven `static unsigned long holdrand = 0x89abcdef;` — platform-width
    /// `c_ulong` by ruling (2026-07-09, reversing the earlier u32
    /// normalization): 32-bit on the retail i686 ship, 64-bit on LP64
    /// referee/native builds, exactly as Raven's `unsigned long` compiles.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    holdrand: c_ulong,

    /// Raven `bg_lib.c`'s `static int randSeed = 0;` — the independent LCG
    /// state backing `bg_lib.c`'s `rand`/`srand` and the `q_shared.h`
    /// `random`/`crandom` macros.
    /// Source: `oracle/codemp/game/bg_lib.c:763`
    randSeed: u32,
}

impl Rng {
    /// The initial `holdrand` value (`0x89abcdef`) — the state a fresh zeroed
    /// `BgState` must carry, so this is what `Default`/`new` install.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    const HOLDRAND_INIT: c_ulong = 0x89ab_cdef;

    /// Fresh generator seeded with Raven's compile-time `holdrand` value and
    /// `bg_lib.c`'s compile-time `randSeed = 0`.
    pub fn new() -> Self {
        Self {
            holdrand: Self::HOLDRAND_INIT,
            randSeed: 0,
        }
    }

    /// Read-only observation of the current `holdrand` state — the parity
    /// tripwire dumped as `rng=%08x` by the pmove differential (a mid-Pmove draw
    /// moves it). Observes only; production behavior is unchanged.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    pub fn holdrand(&self) -> c_ulong {
        self.holdrand
    }

    /// Raven `Rand_Init` — reseed the generator (`int` → `unsigned long`
    /// conversion; Rust's sign-extending `as` cast matches C's value-mod-2^N).
    /// Source: `oracle/codemp/game/q_math.c:1434-1437`
    pub fn Rand_Init(&mut self, seed: c_int) {
        self.holdrand = seed as c_ulong;
    }

    /// Raven `flrand` — returns a float `min <= x < max` (exclusive; will get
    /// `max - 0.00001`, but never `max`).
    /// Source: `oracle/codemp/game/q_math.c:1441-1450`
    pub fn flrand(&mut self, min: f32, max: f32) -> f32 {
        self.holdrand = self.holdrand.wrapping_mul(214013).wrapping_add(2531011);
        // Raven: `(float)(holdrand >> 17)` — full unsigned-long width, so on
        // LP64 this is NOT confined to 0-32767 (referee-proven behavior).
        let result = (self.holdrand >> 17) as f32;
        ((result * (max - min)) / 32768.0f32) + min
    }

    /// Raven `Q_flrand` — the thin dual over `flrand`.
    /// Source: `oracle/codemp/game/q_math.c:1451-1454`
    pub fn Q_flrand(&mut self, min: f32, max: f32) -> f32 {
        self.flrand(min, max)
    }

    /// Raven `irand` — returns an integer `min <= x <= max` (inclusive).
    ///
    /// Raven asserts `(max - min) < 32768`; we preserve the wrapping integer
    /// arithmetic rather than the debug assert. `result = holdrand >> 17` is
    /// an `int`, i.e. the shift happens at `unsigned long` width and then
    /// truncates to 32 bits (arm64 disasm: `lsr x2,x2,#17; madd w2,...`).
    /// Source: `oracle/codemp/game/q_math.c:1458-1469`
    pub fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        debug_assert!((max - min) < 32768);
        let max = max + 1;
        self.holdrand = self.holdrand.wrapping_mul(214013).wrapping_add(2531011);
        let result = (self.holdrand >> 17) as c_int;
        (result.wrapping_mul(max - min) >> 15).wrapping_add(min)
    }

    /// Raven `Q_irand` — the thin dual over `irand`.
    /// Source: `oracle/codemp/game/q_math.c:1471-1474`
    pub fn Q_irand(&mut self, value1: c_int, value2: c_int) -> c_int {
        self.irand(value1, value2)
    }

    /// Raven `bg_lib.c`'s `srand` — (re)seeds the independent `randSeed` LCG.
    /// Source: `oracle/codemp/game/bg_lib.c:765-767`
    pub fn srand(&mut self, seed: c_uint) {
        self.randSeed = seed as u32;
    }

    /// Raven `bg_lib.c`'s `rand` — `randSeed = 69069*randSeed + 1; return
    /// randSeed & 0x7fff;`.
    /// Source: `oracle/codemp/game/bg_lib.c:769-772`
    pub fn rand(&mut self) -> c_int {
        self.randSeed = 69069u32.wrapping_mul(self.randSeed).wrapping_add(1);
        (self.randSeed & 0x7fff) as c_int
    }

    /// Raven `random()` macro — `(rand() & 0x7fff) / ((float)0x7fff)`.
    /// Source: `oracle/codemp/game/q_shared.h:1591`
    pub fn random(&mut self) -> f32 {
        ((self.rand() & 0x7fff) as f32) / (0x7fff as f32)
    }

    /// Raven `crandom()` macro — `2.0 * (random() - 0.5)`.
    /// Source: `oracle/codemp/game/q_shared.h:1592`
    pub fn crandom(&mut self) -> f32 {
        2.0 * (self.random() - 0.5)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

// Ruling (2026-07-09): `holdrand` is `c_ulong` — platform-width, exactly as
// Raven's `unsigned long` compiles per target (retail i686 ship = 32-bit,
// LP64 referee/native = 64-bit). This reverses the 2026-07 u32 normalization:
// the referee A/B oracle proved the 64-bit stream is the ground truth on
// LP64 (t2_wedge `Q_irand` NPC-type picks diverge under a u32 model even
// though the low 32 bits of the stream agree). Goldens for this family are
// regenerated per host width by `tools/jampgame-oracle/run.sh`.
// Source: `oracle/codemp/game/q_math.c:1432` (holdrand*214013+2531011 >>17)
