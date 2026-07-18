//! `QRand` — the engine island's own faithful LCG (Raven's VC-libc `rand()`
//! clone), the engine-tier counterpart of the game-tier
//! `bg_channel::rng::Rng`.
//!
//! Raven kept a single file-static `holdrand` in `q_math.c` shared by every
//! translation unit that linked it. In this port the game DLL and the engine
//! link independent copies of that TU, so by ruling 21 the engine island gets
//! its OWN generator instance at `Engine.common.qrand`, distinct from the
//! game-tier `BgState.rng` — the two never share state, exactly mirroring the
//! separate `holdrand`/`randSeed` statics each linked copy carried.
//!
//! The arithmetic below is copied bit-for-bit from the verified game-tier
//! `Rng` (`crates/mp/game/src/bg_channel/rng.rs`); do not re-derive it.
//!
//! Source: `oracle/codemp/game/q_math.c:1425-1474`
#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_ulong};

use native_math::rng::HoldrandLcg;

/// Raven's `holdrand` seed plus the VC-libc `rand()` LCG, the engine island's
/// own instance (ruling 21).
///
/// Raven kept two independent generator states — the `q_math.c` file-static
/// `holdrand` (`q_math.c:1432`) and `bg_lib.c`'s file-static `randSeed`
/// (`bg_lib.c:763`) — that never shared state; both are kept threaded here.
///
/// Source: `oracle/codemp/game/q_math.c:1432`
pub struct QRand {
    /// Raven `static unsigned long holdrand = 0x89abcdef;` — platform-width
    /// `c_ulong` by ruling (2026-07-09): 32-bit on the retail i686 ship,
    /// 64-bit on LP64 referee/native builds, exactly as Raven's
    /// `unsigned long` compiles.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    holdrand: HoldrandLcg,

    /// Raven `bg_lib.c`'s `static int randSeed = 0;` — the independent LCG
    /// state backing `bg_lib.c`'s `rand`/`srand` and the `q_shared.h`
    /// `random`/`crandom` macros.
    /// Source: `oracle/codemp/game/bg_lib.c:763`
    randSeed: u32,
}

impl QRand {
    /// Fresh generator seeded with Raven's compile-time `holdrand` value and
    /// `bg_lib.c`'s compile-time `randSeed = 0`.
    pub fn new() -> Self {
        Self {
            holdrand: HoldrandLcg::new(),
            randSeed: 0,
        }
    }

    /// Raven `Rand_Init` — reseed the generator (`int` → `unsigned long`
    /// conversion; Rust's sign-extending `as` cast matches C's value-mod-2^N).
    /// Source: `oracle/codemp/game/q_math.c:1434-1437`
    pub fn Rand_Init(&mut self, seed: c_int) {
        self.holdrand.Rand_Init(seed);
    }

    /// Raven `flrand` — returns a float `min <= x < max` (exclusive; will get
    /// `max - 0.00001`, but never `max`).
    /// Source: `oracle/codemp/game/q_math.c:1441-1450`
    pub fn flrand(&mut self, min: f32, max: f32) -> f32 {
        self.holdrand.flrand(min, max)
    }

    /// Raven `Q_flrand` — the thin dual over `flrand`.
    /// Source: `oracle/codemp/game/q_math.c:1451-1454`
    pub fn Q_flrand(&mut self, min: f32, max: f32) -> f32 {
        self.flrand(min, max)
    }

    /// Raven `irand` — returns an integer `min <= x <= max` (inclusive).
    ///
    /// Raven asserts `(max - min) < 32768`; we preserve the wrapping integer
    /// arithmetic rather than the debug assert. `holdrand >> 17` at the
    /// retail-win32 32-bit width is confined to [0, 32767].
    /// Source: `oracle/codemp/game/q_math.c:1458-1469`
    pub fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        self.holdrand.irand(min, max)
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

impl Default for QRand {
    fn default() -> Self {
        Self::new()
    }
}
