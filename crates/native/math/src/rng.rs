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

/// Raven's `flrand`/`irand` divide the LCG's top 15 bits (`holdrand >> 17`)
/// by `32768.0F` (`0x8000`), i.e. treat the output range as `0..0x7fff`
/// inclusive — the VC libc `RAND_MAX` convention. No `#define RAND_MAX`
/// appears in `q_math.c` itself; this names that implicit constant.
/// Source: `oracle/codemp/game/q_math.c:1445-1446,1465-1466`
pub const RAND_MAX: c_int = 0x7fff;

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

// ---------------------------------------------------------------------------
// The two owner state-bundles (2026-07-17 homogenization, step 1: verbatim
// moves from `mp_bg::bg_channel::rng` and `mp_engine_qcommon::common::qrand`;
// step 2 merges them — QRand's bg_lib 69069 rand half is retail-unfaithful,
// the exe's rand() is the MSVC CRT, see jk2mp.vcproj which compiles
// game/q_math.c but NOT bg_lib.c).
// ---------------------------------------------------------------------------

use core::ffi::{c_uint, c_ulong};

/// Raven's `holdrand` seed plus the C runtime `rand()` LCG.
///
/// The retail native module carries two independent generator states — the
/// file-static `holdrand` (`q_math.c:1432`) and the MSVC CRT `rand()`'s own
/// holdrand — that never share state; both are kept threaded here.
///
/// Source: `oracle/codemp/game/q_math.c:1432`
pub struct Rng {
    /// Raven `static unsigned long holdrand = 0x89abcdef;` — 32-bit by the
    /// retail-win32 parity bar (2026-07-17 ruling, applying the 2026-07-12
    /// SnapVector/libm precedent; reverses the 2026-07-09 platform-width
    /// ruling). On LP64 the full-register `holdrand >> 17` made
    /// `irand(1,2)` return ±32k garbage (the level-1 lightning "instagib"
    /// live finding) — retail win32's 32-bit `unsigned long` confines the
    /// draw to [0,32767]. The oracle referee build is patched to 32-bit in
    /// `tools/referee-oracle/build.sh` to match.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    holdrand: HoldrandLcg,

    /// The C runtime `rand()` state backing the native module's `rand`/`srand`
    /// and the `q_shared.h` `random`/`crandom` macros. Retail win32 links the
    /// MSVC CRT here (`bg_lib.c` is `ExcludedFromBuild` in every
    /// `JK2_game.vcproj` win32 config AND its `rand` sits under `#ifdef
    /// Q3_VM`, `bg_lib.c:754` — the bg_lib 69069 LCG is QVM-only). MSVC's
    /// `holdrand` is a 32-bit `long` on win32/win64 alike, initialized to 1.
    crt_holdrand: u32,
}

impl Rng {
    /// Fresh generator seeded with Raven's compile-time `holdrand` value and
    /// the MSVC CRT's compile-time `holdrand = 1L`.
    pub fn new() -> Self {
        Self {
            holdrand: HoldrandLcg::new(),
            crt_holdrand: 1,
        }
    }

    /// Read-only observation of the current `holdrand` state — the parity
    /// tripwire dumped as `rng=%08x` by the pmove differential (a mid-Pmove draw
    /// moves it). Observes only; production behavior is unchanged.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    pub fn holdrand(&self) -> c_ulong {
        self.holdrand.state() as c_ulong
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
    /// retail-win32 32-bit width is confined to [0, 32767], so the result is
    /// `min ..= max` (well, `min ..= max+?` per Raven's own off-by-one shape).
    /// Source: `oracle/codemp/game/q_math.c:1458-1469`
    pub fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        self.holdrand.irand(min, max)
    }

    /// Raven `Q_irand` — the thin dual over `irand`.
    /// Source: `oracle/codemp/game/q_math.c:1471-1474`
    pub fn Q_irand(&mut self, value1: c_int, value2: c_int) -> c_int {
        self.irand(value1, value2)
    }

    /// Current MSVC-CRT `holdrand`, read-only, for referee probes to dump RNG state.
    pub fn dbg_holdrand(&self) -> u32 {
        self.crt_holdrand
    }

    /// The native module's `srand` — the MSVC CRT one retail links (called by
    /// `G_InitGame`'s `srand(randomSeed)`, `g_main.c:929`); reseeds
    /// `crt_holdrand`.
    pub fn srand(&mut self, seed: c_uint) {
        self.crt_holdrand = seed as u32;
    }

    /// The native module's `rand` — the MSVC CRT LCG retail links:
    /// `holdrand = holdrand * 214013 + 2531011; return (holdrand >> 16) &
    /// 0x7fff;`. (bg_lib.c's 69069 LCG is QVM-only — see `crt_holdrand`;
    /// wrong-variant fix found by the lockstep referee, 2026-07-14.)
    pub fn rand(&mut self) -> c_int {
        self.crt_holdrand = self.crt_holdrand.wrapping_mul(214013).wrapping_add(2531011);
        ((self.crt_holdrand >> 16) & 0x7fff) as c_int
    }

    /// Raven `random()` macro — `(rand() & 0x7fff) / ((float)0x7fff)`.
    /// Source: `oracle/codemp/game/q_shared.h:1591`
    pub fn random(&mut self) -> f32 {
        ((self.rand() & 0x7fff) as f32) / (0x7fff as f32)
    }

    /// Raven `crandom()` macro — `2.0 * (random() - 0.5)`. The `2.0`/`0.5`
    /// literals are doubles, so the macro's C type is `double`; the `f32`
    /// `random()` widens exactly to `f64` before the subtract, and every
    /// expression touching a call stays `double` in C until it narrows to a
    /// float lvalue/parameter.
    /// Source: `oracle/codemp/game/q_shared.h:1592`
    pub fn crandom(&mut self) -> f64 {
        2.0 * (self.random() as f64 - 0.5)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    /// The MSVC CRT `rand()` stream — the canonical first draws for the
    /// default seed (1) and a reseed, verifying retail-win32 semantics
    /// (32-bit holdrand, *214013+2531011, >>16, &0x7fff).
    #[test]
    fn crt_rand_matches_msvc_stream() {
        let mut rng = Rng::new();
        let first: Vec<i32> = (0..10).map(|_| rng.rand()).collect();
        assert_eq!(
            first,
            [41, 18467, 6334, 26500, 19169, 15724, 11478, 29358, 26962, 24464]
        );
        rng.srand(42);
        let reseeded: Vec<i32> = (0..4).map(|_| rng.rand()).collect();
        // 42*214013+2531011 = 11519557 -> >>16 = 175, then the chain follows.
        assert_eq!(reseeded[0], 175);
        rng.srand(1);
        assert_eq!(rng.rand(), 41);
    }
}

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
