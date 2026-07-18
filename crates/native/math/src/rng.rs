//! Raven's `holdrand` LCG — THE single definition of the algorithm.
//!
//! Raven copy-pastes this generator (game `q_math.c:1432-1474` compiled into
//! game/cgame/ui, plus `CCMLandScape`'s member copy) — separate STATE per
//! copy, one algorithm. This type mirrors that: owners embed their own
//! instance (state separation preserved); the arithmetic lives only here.
//!
//! Width: 32-bit per the retail-win32 parity bar (2026-07-17 ruling; win32
//! `unsigned long` is 32-bit — at LP64 width `holdrand >> 17` spans the full
//! register and `irand(1,2)` returns ±32k garbage; the level-1 lightning
//! "instagib" live finding).
//!
//! Source: `oracle/codemp/game/q_math.c:1432-1474`
#![allow(non_snake_case)]

use core::ffi::c_int;

/// One `holdrand` state. Raven seeds the file-static with `0x89abcdef`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoldrandLcg(pub u32);

impl HoldrandLcg {
    /// Raven's compile-time initializer (`q_math.c:1432`).
    pub const INIT: u32 = 0x89ab_cdef;

    pub const fn new() -> Self {
        Self(Self::INIT)
    }

    /// Raven `Rand_Init` — reseed (`int` stored into the unsigned state).
    /// Source: `oracle/codemp/game/q_math.c:1434-1437`
    pub fn Rand_Init(&mut self, seed: c_int) {
        self.0 = seed as u32;
    }

    /// Raven `flrand` — `min <= x < max` (never exactly `max`).
    /// Source: `oracle/codemp/game/q_math.c:1441-1450`
    pub fn flrand(&mut self, min: f32, max: f32) -> f32 {
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        // `(float)(holdrand >> 17)` — 32-bit width confines this to [0, 32767].
        let result = (self.0 >> 17) as f32;
        ((result * (max - min)) / 32768.0f32) + min
    }

    /// Raven `Q_flrand` — thin dual over `flrand`.
    /// Source: `oracle/codemp/game/q_math.c:1451-1454`
    pub fn Q_flrand(&mut self, min: f32, max: f32) -> f32 {
        self.flrand(min, max)
    }

    /// Raven `irand` — integer `min ..= max` (Raven pre-increments max).
    /// Source: `oracle/codemp/game/q_math.c:1458-1469`
    pub fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        debug_assert!((max - min) < 32768);
        let max = max + 1;
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        let result = (self.0 >> 17) as c_int;
        (result.wrapping_mul(max - min) >> 15).wrapping_add(min)
    }

    /// Raven `Q_irand` — thin dual over `irand`.
    /// Source: `oracle/codemp/game/q_math.c:1471-1474`
    pub fn Q_irand(&mut self, value1: c_int, value2: c_int) -> c_int {
        self.irand(value1, value2)
    }

    /// Current state (parity-tripwire dumps).
    pub fn state(&self) -> u32 {
        self.0
    }
}

impl Default for HoldrandLcg {
    fn default() -> Self {
        Self::new()
    }
}
