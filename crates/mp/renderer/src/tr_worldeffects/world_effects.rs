//! Raven `tr_WorldEffects.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_WorldEffects.cpp`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cm_test::CM_PointContents;
use mp_engine_qcommon::cmd_common::Cmd_ArgsBuffer;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::{com_error, com_printf};
use mp_engine_qcommon::common_fns::Com_Milliseconds;
use mp_engine_qcommon::cvar_fns::Cvar_VariableIntegerValue;
use mp_qshared::shared::com_parse::{COM_ParseExt, QSharedScratch};
use mp_qshared::shared::vec3_t;
use mp_qshared::shared::{
    errorParm_t, CONTENTS_INSIDE, CONTENTS_OUTSIDE, CONTENTS_SOLID, CONTENTS_WATER,
};
use native_math::qmath::{MakeNormalVectors, VectorNormalize, _DotProduct};
use native_math::rng::{Rng, RAND_MAX};
use native_string::atoi;

use crate::gl_constants::GL_CLAMP;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::TrRefdef;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_backend::GL_Bind;
use crate::tr_image::{R_FindImageFile, TrImageState};
use crate::tr_model::render_models::RenderModels;
use crate::tr_public::ref_flags::{RDF_NOWORLDMODEL, RDF_SKYBOXPORTAL};
use crate::tr_shader::ParseVector;

/// Raven `POINTCACHE_CELL_SIZE` — the weather point-cache cell edge length.
/// Both preprocessor branches (`_XBOX` and the PC `#else`) define the same
/// `96.0f`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:51` and
/// `oracle/codemp/renderer/tr_WorldEffects.cpp:61`
pub const POINTCACHE_CELL_SIZE: f32 = 96.0;

/// Raven `MAX_WEATHER_ZONES` — the `ratl::vector_vs` capacity backing
/// `COutside::mWeatherZones` (its `full()` bound).
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:46`
pub const MAX_WEATHER_ZONES: usize = 10;

/// Raven `MAX_WIND_ZONES` — the `ratl::vector_vs` capacity backing
/// `mWindZones` (its `full()` bound).
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:45`
pub const MAX_WIND_ZONES: usize = 10;

/// Raven `MAX_PARTICLE_CLOUDS` — the `ratl::vector_vs` capacity backing
/// `mParticleClouds` (its `full()` bound).
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:48`
pub const MAX_PARTICLE_CLOUDS: usize = 5;

/// Raven `WE_flrand`.
///
/// Raven: "Returns a float min <= x < max (exclusive; will get max - 0.00001;
/// but never max)".
///
/// Raven's `rand()`/`RAND_MAX` are the C runtime's (msvcrt on retail), not
/// `q_math.c`'s `holdrand` generator; `Rng::rand` is the exact msvcrt replica
/// and `Rng` is threaded rather than reached (porting-rules §B4).
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:13-15`
pub fn WE_flrand(rng: &mut Rng, min: f32, max: f32) -> f32 {
    ((rng.rand() as f32 * (max - min)) / (RAND_MAX + 1) as f32) + min
}

/// Raven `VectorFloor`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:83-88`
pub fn VectorFloor(v: &mut vec3_t) {
    v[0] = v[0].floor();
    v[1] = v[1].floor();
    v[2] = v[2].floor();
}

/// Raven `VectorCeil`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:90-95`
pub fn VectorCeil(v: &mut vec3_t) {
    v[0] = v[0].ceil();
    v[1] = v[1].ceil();
    v[2] = v[2].ceil();
}

/// Raven `FloatRand`.
///
/// Same msvcrt `rand()`/`RAND_MAX` pair as `WE_flrand`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:97-100`
pub fn FloatRand(rng: &mut Rng) -> f32 {
    rng.rand() as f32 / RAND_MAX as f32
}

/// Raven `SnapFloatToGrid`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:102-129`
pub fn SnapFloatToGrid(f: &mut f32, grid_size: i32) {
    *f = (*f as i32) as f32;

    let f_neg = *f < 0.0;
    if f_neg {
        *f *= -1.0; // Temporarly make it positive
    }

    let offset = (*f as i32) % grid_size;
    let offset_abs = offset.abs();
    let offset = if offset_abs > (grid_size / 2) {
        (grid_size - offset_abs) * -1
    } else {
        offset
    };

    *f -= offset as f32;

    if f_neg {
        *f *= -1.0; // Put It Back To Negative
    }

    *f = (*f as i32) as f32;

    debug_assert_eq!((*f as i32) % grid_size, 0);
}

/// Raven `SnapVectorToGrid`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:131-136`
pub fn SnapVectorToGrid(v: &mut vec3_t, grid_size: i32) {
    SnapFloatToGrid(&mut v[0], grid_size);
    SnapFloatToGrid(&mut v[1], grid_size);
    SnapFloatToGrid(&mut v[2], grid_size);
}

/// Raven `SVecRange` — an inclusive min/max box range over a 3-vector.
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`SVecRange` class, ~line 140s)
pub struct SVecRange {
    pub mMins: vec3_t,
    pub mMaxs: vec3_t,
}

impl SVecRange {
    /// Raven `SVecRange::Clear`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:150-154`
    pub fn Clear(&mut self) {
        self.mMins = [0.0; 3];
        self.mMaxs = [0.0; 3];
    }

    /// Raven `SVecRange::Wrap`.
    ///
    /// Raven's `spawnRange` parameter is unused in the original body — kept
    /// for signature fidelity.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:162-196`
    pub fn Wrap(&self, v: &mut vec3_t, _spawn_range: &mut SVecRange) {
        for i in 0..3 {
            if v[i] < self.mMins[i] {
                let d = self.mMins[i] - v[i];
                v[i] = self.mMaxs[i] - d % (self.mMaxs[i] - self.mMins[i]);
            }
            if v[i] > self.mMaxs[i] {
                let d = v[i] - self.mMaxs[i];
                v[i] = self.mMins[i] + d % (self.mMaxs[i] - self.mMins[i]);
            }
        }
    }

    // PORT-NOTE: Raven's `CVec3::operator>`/`operator<` (used here as
    // `V>mMins && V<mMaxs`) aren't in this packet's excerpt (a different
    // oracle header, out of the wave-partition graph). Ported as the
    // standard component-wise AABB test.
    /// Raven `SVecRange::In`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:198-201`
    pub fn In(&self, v: &vec3_t) -> bool {
        (v[0] > self.mMins[0] && v[1] > self.mMins[1] && v[2] > self.mMins[2])
            && (v[0] < self.mMaxs[0] && v[1] < self.mMaxs[1] && v[2] < self.mMaxs[2])
    }

    /// Raven `SVecRange::Pick`.
    ///
    /// Raven's `CVec3&` out-param becomes the `v: &mut vec3_t` out-param
    /// (porting-rules §C7); `WE_flrand` is threaded through `rng` rather than
    /// reaching the msvcrt `rand()` stream (porting-rules §B4).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:156-161`
    pub fn Pick(&self, rng: &mut Rng, v: &mut vec3_t) {
        v[0] = WE_flrand(rng, self.mMins[0], self.mMaxs[0]);
        v[1] = WE_flrand(rng, self.mMins[1], self.mMaxs[1]);
        v[2] = WE_flrand(rng, self.mMins[2], self.mMaxs[2]);
    }
}

/// Raven `SFloatRange` — a min/max float range.
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`SFloatRange` class, ~line 205s)
pub struct SFloatRange {
    pub mMin: f32,
    pub mMax: f32,
}

impl SFloatRange {
    /// Raven `SFloatRange::Clear`.
    ///
    /// Raven's body writes `mMin` twice (`mMin = 0; mMin = 0;`) and never
    /// touches `mMax` — a real Raven bug, preserved faithfully (porting-rules
    /// §A2: no speculative behavior).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:209-213`
    pub fn Clear(&mut self) {
        self.mMin = 0.0;
        self.mMin = 0.0;
    }

    /// Raven `SFloatRange::In`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:218-221`
    pub fn In(&self, v: f32) -> bool {
        v > self.mMin && v < self.mMax
    }

    /// Raven `SFloatRange::Pick`.
    ///
    /// Raven's `float&` out-param becomes the `v: &mut f32` out-param
    /// (porting-rules §C7).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:214-217`
    pub fn Pick(&self, rng: &mut Rng, v: &mut f32) {
        *v = WE_flrand(rng, self.mMin, self.mMax);
    }
}

/// Raven `SIntRange` — a min/max int range.
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`SIntRange` class, ~line 225s)
pub struct SIntRange {
    pub mMin: i32,
    pub mMax: i32,
}

impl SIntRange {
    /// Raven `SIntRange::Clear`.
    ///
    /// Same double-`mMin` bug as `SFloatRange::Clear` — preserved faithfully.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:229-233`
    pub fn Clear(&mut self) {
        self.mMin = 0;
        self.mMin = 0;
    }

    /// Raven `SIntRange::Pick`.
    ///
    /// Raven's `int&` out-param becomes the return value (porting-rules §C7);
    /// `Q_irand` is `q_math.c`'s `holdrand` LCG, threaded rather than reached.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:234-237`
    pub fn Pick(&self, rng: &mut Rng) -> i32 {
        rng.Q_irand(self.mMin, self.mMax)
    }

    /// Raven `SIntRange::In`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:238-241`
    pub fn In(&self, v: i32) -> bool {
        v > self.mMin && v < self.mMax
    }
}

/// Raven `CWeatherParticle` — one weather particle: alpha, render/fade
/// flags, position, velocity and mass.
///
/// Raven's `TFlags mFlags` is `ratl::bits_vs<FLAG_MAX>`, a four-bit set held
/// in one word and cleared by its default constructor; a `u32` bit mask
/// indexed by the `FLAG_*` constants below carries it.
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp:251-271`
pub struct CWeatherParticle {
    pub mAlpha: f32,
    pub mFlags: u32,
    pub mPosition: vec3_t,
    pub mVelocity: vec3_t,
    /// Raven: "A higher number will more greatly resist force and result in
    /// greater gravity".
    pub mMass: f32,
}

impl CWeatherParticle {
    /// Raven `CWeatherParticle::FLAG_RENDER`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:254-263`
    pub const FLAG_RENDER: u32 = 0;
    /// Raven `CWeatherParticle::FLAG_FADEIN`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:254-263`
    pub const FLAG_FADEIN: u32 = 1;
    /// Raven `CWeatherParticle::FLAG_FADEOUT`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:254-263`
    pub const FLAG_FADEOUT: u32 = 2;
    /// Raven `CWeatherParticle::FLAG_RESPAWN`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:254-263`
    pub const FLAG_RESPAWN: u32 = 3;
    /// Raven `CWeatherParticle::FLAG_MAX` — the `TFlags` bit width.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:254-263`
    pub const FLAG_MAX: u32 = 4;

    // PORT-NOTE: not a named Raven symbol — `new CWeatherParticle[count]`
    // default-constructs each element: `mFlags`'s `ratl::bits_vs` ctor clears
    // every bit, the four POD members are left indeterminate. §19: reading
    // indeterminate storage is UB; zero is the one defined value picked here,
    // and it is unobservable — `CWeatherParticleCloud::Initialize` writes all
    // four for every element immediately after allocating.
    fn zeroed() -> Self {
        Self {
            mAlpha: 0.0,
            mFlags: 0,
            mPosition: [0.0; 3],
            mVelocity: [0.0; 3],
            mMass: 0.0,
        }
    }
}

/// Raven `CWindZone` — one wind-zone volume (bounds + velocity distribution).
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp:276-356`
pub struct CWindZone {
    pub mRBounds: SVecRange,
    pub mGlobal: bool,
    pub mRVelocity: SVecRange,
    pub mMaxDeltaVelocityPerUpdate: f32,
    pub mRDuration: SIntRange,
    pub mChanceOfDeadTime: f32,
    pub mRDeadTime: SIntRange,
    pub mCurrentVelocity: vec3_t,
    pub mTargetVelocity: vec3_t,
    pub mTargetVelocityTimeRemaining: i32,
}

impl CWindZone {
    /// Raven `CWindZone::Initialize`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:300-322`
    pub fn Initialize(&mut self) {
        self.mRBounds.Clear();
        self.mGlobal = true;

        self.mRVelocity.mMins = [-1500.0; 3];
        self.mRVelocity.mMins[2] = -10.0;
        self.mRVelocity.mMaxs = [1500.0; 3];
        self.mRVelocity.mMaxs[2] = 10.0;

        self.mMaxDeltaVelocityPerUpdate = 10.0;

        self.mRDuration.mMin = 1000;
        self.mRDuration.mMax = 2000;

        self.mChanceOfDeadTime = 0.3;
        self.mRDeadTime.mMin = 1000;
        self.mRDeadTime.mMax = 3000;

        self.mCurrentVelocity = [0.0; 3];
        self.mTargetVelocity = [0.0; 3];
        self.mTargetVelocityTimeRemaining = 0;
    }

    /// Raven `CWindZone::Update`.
    ///
    /// `FloatRand`/`SIntRange::Pick`/`SVecRange::Pick` are threaded through
    /// `rng` rather than reaching the msvcrt `rand()` stream (porting-rules
    /// §B4). Raven's `CVec3::ScaleAdd`/`operator*=` are expanded component-
    /// wise (no `vec3_t` operator overloads under the interior-safety law).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:327-355`
    pub fn Update(&mut self, rng: &mut Rng) {
        if self.mTargetVelocityTimeRemaining == 0 {
            if FloatRand(rng) < self.mChanceOfDeadTime {
                self.mTargetVelocityTimeRemaining = self.mRDeadTime.Pick(rng);
                self.mTargetVelocity = [0.0; 3];
            } else {
                self.mTargetVelocityTimeRemaining = self.mRDuration.Pick(rng);
                self.mRVelocity.Pick(rng, &mut self.mTargetVelocity);
            }
        } else if self.mTargetVelocityTimeRemaining != -1 {
            self.mTargetVelocityTimeRemaining -= 1;

            let mut delta_velocity: vec3_t = [
                self.mTargetVelocity[0] - self.mCurrentVelocity[0],
                self.mTargetVelocity[1] - self.mCurrentVelocity[1],
                self.mTargetVelocity[2] - self.mCurrentVelocity[2],
            ];
            let mut delta_velocity_len = VectorNormalize(&mut delta_velocity);
            if delta_velocity_len > self.mMaxDeltaVelocityPerUpdate {
                delta_velocity_len = self.mMaxDeltaVelocityPerUpdate;
            }
            // Raven `DeltaVelocity *= (DeltaVelocityLen);`
            for i in 0..3 {
                delta_velocity[i] *= delta_velocity_len;
            }
            for i in 0..3 {
                self.mCurrentVelocity[i] += delta_velocity[i];
            }
        }
    }

    // PORT-NOTE: not a named Raven symbol — `ratl::vector_vs::push_back()`
    // default-constructs the new element in place (a no-op ctor; `CWindZone`
    // has no explicit constructor in the oracle), and every call site
    // immediately follows with `.Initialize()`, which this wave confirms
    // sets all 10 fields (§B3: no field is read before `Initialize` writes
    // it). This helper is that zero-valued placeholder, standing in for the
    // C++ implicit default ctor's emplace slot.
    fn zeroed() -> Self {
        Self {
            mRBounds: SVecRange {
                mMins: [0.0; 3],
                mMaxs: [0.0; 3],
            },
            mGlobal: false,
            mRVelocity: SVecRange {
                mMins: [0.0; 3],
                mMaxs: [0.0; 3],
            },
            mMaxDeltaVelocityPerUpdate: 0.0,
            mRDuration: SIntRange { mMin: 0, mMax: 0 },
            mChanceOfDeadTime: 0.0,
            mRDeadTime: SIntRange { mMin: 0, mMax: 0 },
            mCurrentVelocity: [0.0; 3],
            mTargetVelocity: [0.0; 3],
            mTargetVelocityTimeRemaining: 0,
        }
    }
}

/// Raven's wind-zone globals (`mGlobalWindDirection`, `mGlobalWindSpeed`) —
/// file-scope statics in `tr_WorldEffects.cpp` promoted to an owned carrier
/// (DEC-37 A13.3: a kind-3 static gets a field on its owning subsystem
/// struct, never a Rust `static`). Named by this wave.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`mGlobalWindDirection`, `mGlobalWindSpeed`)
#[derive(Default)]
pub struct WindZoneState {
    pub global_wind_direction: vec3_t,
    pub global_wind_speed: f32,
    /// Raven `mGlobalWindVelocity` — the un-normalized accumulated velocity
    /// `RB_RenderWorldEffects` sums every global `CWindZone`'s
    /// `mCurrentVelocity` into each frame, then normalizes into
    /// `global_wind_direction`/`global_wind_speed`. Same DEC-37 A13.3 kind-3
    /// static promotion as the two fields above.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`mGlobalWindVelocity`)
    pub global_wind_velocity: vec3_t,
}

impl WindZoneState {
    /// Raven `R_GetWindVector`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:359-363`
    pub fn R_GetWindVector(&self) -> (bool, vec3_t) {
        (true, self.global_wind_direction)
    }

    /// Raven `R_GetWindSpeed`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:365-369`
    pub fn R_GetWindSpeed(&self) -> (bool, f32) {
        (true, self.global_wind_speed)
    }

    /// Raven `R_GetWindGusting`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:371-374`
    pub fn R_GetWindGusting(&self) -> bool {
        self.global_wind_speed > 1000.0
    }
}

/// Raven `SWeatherZone` — one weather-occlusion zone: a spatial point-cache
/// grid marking whether each cell is "outside".
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`SWeatherZone` class, ~line 380s)
pub struct SWeatherZone {
    pub mExtents: SVecRange,
    pub mSize: SVecRange,
    pub mWidth: i32,
    pub mHeight: i32,
    pub mDepth: i32,
    pub mPointCache: Vec<u32>,
}

impl SWeatherZone {
    /// Raven `SWeatherZone::ConvertToCell`.
    ///
    /// Raven's four `int&` out-params become the returned `(x, y, z, bit)`
    /// tuple (porting-rules §C7).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:409-417`
    pub fn ConvertToCell(&self, pos: &vec3_t) -> (i32, i32, i32, i32) {
        let x = ((pos[0] / POINTCACHE_CELL_SIZE) - self.mSize.mMins[0]) as i32;
        let y = ((pos[1] / POINTCACHE_CELL_SIZE) - self.mSize.mMins[1]) as i32;
        let mut z = ((pos[2] / POINTCACHE_CELL_SIZE) - self.mSize.mMins[2]) as i32;

        let bit = z & 31;
        z >>= 5;

        (x, y, z, bit)
    }

    /// Raven `SWeatherZone::CellOutside`.
    ///
    /// `mMarkedOutside` is Raven's `SWeatherZone::mMarkedOutside` static
    /// (shared by every `SWeatherZone` instance); relocated onto `COutside`
    /// (see `COutside::mMarkedOutside`) and threaded in here rather than
    /// reached (porting-rules §B3/§B4).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:422-429`
    pub fn CellOutside(&self, x: i32, y: i32, z: i32, bit: i32, marked_outside: bool) -> bool {
        if x < 0
            || x >= self.mWidth
            || y < 0
            || y >= self.mHeight
            || z < 0
            || z >= self.mDepth
            || bit < 0
            || bit >= 32
        {
            return !marked_outside;
        }
        let idx = ((z * self.mWidth * self.mHeight) + (y * self.mWidth) + x) as usize;
        marked_outside == ((self.mPointCache[idx] & (1u32 << bit)) != 0)
    }
}

/// Raven `COutside` — the "is this point outside" subsystem: weather-zone
/// point caches plus rain/outside ambience flags.
///
/// `mMarkedOutside` was Raven's `SWeatherZone::mMarkedOutside` static,
/// shared by every `SWeatherZone` instance; relocated onto `COutside`, the
/// owning subsystem aggregate (porting-rules §B3 — no Rust `static`).
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`COutside` class, ~line 435+)
pub struct COutside {
    pub mOutsideShake: bool,
    pub mOutsidePain: f32,
    pub mCacheInit: bool,
    pub mMarkedOutside: bool,
    pub mWeatherZones: Vec<SWeatherZone>,
}

impl COutside {
    /// Raven `COutside::COutside` — delegates to `Reset`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:494-497`
    pub fn new() -> Self {
        let mut this = Self {
            mOutsideShake: false,
            mOutsidePain: 0.0,
            mCacheInit: false,
            mMarkedOutside: false,
            mWeatherZones: Vec::new(),
        };
        this.Reset();
        this
    }

    /// Raven `COutside::ContentsOutside`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:456-471`
    pub fn ContentsOutside(&self, contents: c_int) -> bool {
        if contents & CONTENTS_WATER != 0 || contents & CONTENTS_SOLID != 0 {
            return false;
        }
        if self.mCacheInit {
            if self.mMarkedOutside {
                return contents & CONTENTS_OUTSIDE != 0;
            }
            return contents & CONTENTS_INSIDE == 0;
        }
        contents & CONTENTS_OUTSIDE != 0
    }

    /// Raven `COutside::Reset`.
    ///
    /// Raven frees each zone's `mPointCache` via `Z_Free` (Hunk/Z_Malloc
    /// pool -> ownership, porting-rules §C9); `mPointCache` is now an owned
    /// `Vec`, so `Vec`'s `Drop` performs the equivalent free — `Z_Free` is
    /// not called.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:480-492`
    pub fn Reset(&mut self) {
        self.mOutsideShake = false;
        self.mOutsidePain = 0.0;
        self.mCacheInit = false;
        self.mMarkedOutside = false;
        self.mWeatherZones.clear();
    }

    /// Raven `COutside::Initialized`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:503-506`
    pub fn Initialized(&self) -> bool {
        self.mCacheInit
    }

    /// Raven `COutside::AddWeatherZone` — "Will add a zone of mins and maxes".
    ///
    /// Raven's `ratl::vector_vs<SWeatherZone, MAX_WEATHER_ZONES>::full()`
    /// becomes the `Vec` length bound; the zeroing `Z_Malloc(…, qtrue)` point
    /// cache becomes an owned zero-filled `Vec` (porting-rules §C9).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:511-534`
    pub fn AddWeatherZone(&mut self, mins: vec3_t, maxs: vec3_t) {
        if self.mWeatherZones.len() < MAX_WEATHER_ZONES {
            let mut wz = SWeatherZone {
                mExtents: SVecRange {
                    mMins: mins,
                    mMaxs: maxs,
                },
                mSize: SVecRange {
                    mMins: [0.0; 3],
                    mMaxs: [0.0; 3],
                },
                mWidth: 0,
                mHeight: 0,
                mDepth: 0,
                mPointCache: Vec::new(),
            };

            SnapVectorToGrid(&mut wz.mExtents.mMins, POINTCACHE_CELL_SIZE as i32);
            SnapVectorToGrid(&mut wz.mExtents.mMaxs, POINTCACHE_CELL_SIZE as i32);

            wz.mSize.mMins = wz.mExtents.mMins;
            wz.mSize.mMaxs = wz.mExtents.mMaxs;

            for i in 0..3 {
                wz.mSize.mMins[i] /= POINTCACHE_CELL_SIZE;
                wz.mSize.mMaxs[i] /= POINTCACHE_CELL_SIZE;
            }
            wz.mWidth = (wz.mSize.mMaxs[0] - wz.mSize.mMins[0]) as i32;
            wz.mHeight = (wz.mSize.mMaxs[1] - wz.mSize.mMins[1]) as i32;
            wz.mDepth = (((wz.mSize.mMaxs[2] - wz.mSize.mMins[2]) as i32) + 31) >> 5;

            let array_size = wz.mWidth * wz.mHeight * wz.mDepth;
            wz.mPointCache = vec![0u32; array_size as usize];

            self.mWeatherZones.push(wz);
        }
    }

    /// Raven `COutside::Cache` — "Will Scan the World, Creating The Cache".
    ///
    /// `world_bmodel_bounds` is Raven's `tr.world->bmodels[0].bounds`, threaded
    /// rather than reached (porting-rules §B4); `None` is Raven's `!tr.world`
    /// early-out. Raven copies each zone by value (`SWeatherZone wz = …`) and
    /// marks through the copy's shared `mPointCache` pointer — with an owned
    /// `Vec` that copy would deep-copy, so the cache is written back through
    /// the zone index instead.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:541-632`
    pub fn Cache(&mut self, host: &mut EngineHostView, world_bmodel_bounds: Option<[vec3_t; 2]>) {
        let Some(bounds) = world_bmodel_bounds else {
            return;
        };
        if self.mCacheInit {
            return;
        }

        // Record The Extents Of The World Incase No Other Weather Zones Exist
        //---------------------------------------------------------------------
        if self.mWeatherZones.is_empty() {
            com_printf(host.common, "WARNING: No Weather Zones Encountered");
            self.AddWeatherZone(bounds[0], bounds[1]);
        }

        // Iterate Over All Weather Zones
        //--------------------------------
        for zone in 0..self.mWeatherZones.len() {
            let width = self.mWeatherZones[zone].mWidth;
            let height = self.mWeatherZones[zone].mHeight;
            let depth = self.mWeatherZones[zone].mDepth;

            // Make Sure Point Contents Checks Occur At The CENTER Of The Cell
            //-----------------------------------------------------------------
            let mut mins = self.mWeatherZones[zone].mExtents.mMins;
            for x in 0..3 {
                mins[x] += POINTCACHE_CELL_SIZE / 2.0;
            }

            // Start Scanning
            //----------------
            for z in 0..depth {
                for q in 0..32 {
                    let bit = 1u32 << q;
                    let zbase = z << 5;

                    for x in 0..width {
                        for y in 0..height {
                            let cur_pos: vec3_t = [
                                x as f32 * POINTCACHE_CELL_SIZE + mins[0],
                                y as f32 * POINTCACHE_CELL_SIZE + mins[1],
                                (zbase + q) as f32 * POINTCACHE_CELL_SIZE + mins[2],
                            ];

                            let contents = CM_PointContents(host.cm, cur_pos, 0);
                            if contents & CONTENTS_INSIDE != 0 || contents & CONTENTS_OUTSIDE != 0 {
                                let cur_pos_outside = (contents & CONTENTS_OUTSIDE) != 0;
                                if !self.mCacheInit {
                                    self.mCacheInit = true;
                                    self.mMarkedOutside = cur_pos_outside;
                                } else if self.mMarkedOutside != cur_pos_outside {
                                    debug_assert!(false);
                                    // Raven's trailing `return;` is dead — `com_error`
                                    // does not return (STATE-Q4/DEC-08).
                                    com_error(
                                        errorParm_t::ERR_DROP,
                                        "Weather Effect: Both Indoor and Outdoor brushs encountered in map.\n"
                                            .to_string(),
                                    );
                                }

                                // Mark The Point
                                //----------------
                                let idx = ((z * width * height) + (y * width) + x) as usize;
                                self.mWeatherZones[zone].mPointCache[idx] |= bit;
                            }
                        } // for (y)
                    } // for (x)
                } // for (q)
            } // for (z)
        }

        // If no indoor or outdoor brushes were found
        //--------------------------------------------
        if !self.mCacheInit {
            self.mCacheInit = true;
            self.mMarkedOutside = false; // Assume All Is Outside, Except Solid
        }
    }

    /// Raven `COutside::PointOutside` — "Test to see if a given point is
    /// outside".
    ///
    /// `cm` carries `CM_PointContents`' collision world (porting-rules §B4).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:641-659`
    pub fn PointOutside(&self, cm: &mut CollisionWorld, pos: &vec3_t) -> bool {
        if !self.mCacheInit {
            return self.ContentsOutside(CM_PointContents(cm, *pos, 0));
        }
        for wz in &self.mWeatherZones {
            if wz.mExtents.In(pos) {
                let (x, y, z, bit) = wz.ConvertToCell(pos);
                return wz.CellOutside(x, y, z, bit, self.mMarkedOutside);
            }
        }
        !self.mMarkedOutside
    }

    /// Raven `COutside::PointOutside` — "Test to see if a given bounded plane
    /// is outside".
    ///
    /// PORT-NOTE: Raven overloads `PointOutside` with this three-argument
    /// variant; Rust has no overloading, so this symbol is disambiguated as
    /// `PointOutsideBounded`. Raven's `mWCells`/`mHCells`/`mXCell`/`mYCell`/
    /// `mZBit`/`mXMax`/`mYMax`/`mZMax` are declared as `COutside` members but
    /// read and written only inside this body, so they are locals here.
    ///
    /// Raven's `mYMax = y + mWCells` (width cells on the Y axis) and the
    /// inner `CellOutside(…, z, mZBit)` (cell `z`, not `mZBit`, on the third
    /// axis) are preserved as written (porting-rules §A2).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:665-703`
    pub fn PointOutsideBounded(&self, pos: &vec3_t, width: f32, height: f32) -> bool {
        for wz in &self.mWeatherZones {
            if wz.mExtents.In(pos) {
                let (x, y, z, bit) = wz.ConvertToCell(pos);
                if width < POINTCACHE_CELL_SIZE || height < POINTCACHE_CELL_SIZE {
                    return wz.CellOutside(x, y, z, bit, self.mMarkedOutside);
                }

                let w_cells = ((width as i32) as f32 / POINTCACHE_CELL_SIZE) as i32;
                let h_cells = ((height as i32) as f32 / POINTCACHE_CELL_SIZE) as i32;

                let x_max = x + w_cells;
                let y_max = y + w_cells;
                let z_max = bit + h_cells;

                let mut x_cell = x - w_cells;
                while x_cell <= x_max {
                    let mut y_cell = y - w_cells;
                    while y_cell <= y_max {
                        let mut z_bit = bit - h_cells;
                        while z_bit <= z_max {
                            if !wz.CellOutside(x_cell, y_cell, z, z_bit, self.mMarkedOutside) {
                                return false;
                            }
                            z_bit += 1;
                        }
                        y_cell += 1;
                    }
                    x_cell += 1;
                }
                return true;
            }
        }
        !self.mMarkedOutside
    }
}

impl Default for COutside {
    fn default() -> Self {
        Self::new()
    }
}

/// Raven `COutside::~COutside` — delegates to `Reset`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:498-501`
impl Drop for COutside {
    fn drop(&mut self) {
        self.Reset();
    }
}

/// Raven's world-effects TU-scope singletons (`mOutside`, `mParticleClouds`,
/// `mWindZones`) — file-scope statics in `tr_WorldEffects.cpp` promoted to
/// one owned carrier (DEC-37 A13.3: a kind-3 static gets a field on its
/// owning subsystem struct, never a Rust `static`). Named by this wave.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`mOutside`,
/// `mParticleClouds`, `mWindZones`)
#[derive(Default)]
pub struct WorldEffectsState {
    pub mOutside: COutside,
    pub mParticleClouds: Vec<CWeatherParticleCloud>,
    pub mWindZones: Vec<CWindZone>,
    /// Raven's `mGlobalWindVelocity`, `mGlobalWindDirection` and `mGlobalWindSpeed` file statics, the same DEC-37 A13.3 promotion as `mOutside`.
    /// The trio had no owner before this step, so nothing could call the functions that take it.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:73-75`
    pub wind: WindZoneState,
    /// The C runtime's `holdrand` (`srand`/`rand`, seeded by
    /// `R_InitWorldEffects`) plus `q_math.c`'s `holdrand` behind `Q_irand` —
    /// both TU-invisible globals in Raven, owned here as one field on the
    /// subsystem carrier (DEC-37 A13.3) and threaded into `WE_flrand`/
    /// `FloatRand`/`SIntRange::Pick`.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1491`
    pub rng: Rng,
    /// Raven `mFrozen` — freezes wind-zone/particle-cloud updates while
    /// still true when set (`CWeatherParticleCloud::Update`'s early return,
    /// `RB_RenderWorldEffects`'s wind-zone update guard). Same DEC-37 A13.3
    /// promotion as `mOutside`/`mParticleClouds`/`mWindZones` above.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`mFrozen`)
    pub mFrozen: bool,
    /// Raven `mMillisecondsElapsed` — this frame's clamped `backEnd.refdef
    /// .frametime`, written by `RB_RenderWorldEffects` and read nowhere else
    /// in this wave's fns (kept for fidelity; `mSecondsElapsed` is the
    /// derived value consumers read).
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`mMillisecondsElapsed`)
    pub mMillisecondsElapsed: f32,
    /// Raven `mSecondsElapsed` — `mMillisecondsElapsed / 1000`, read by every
    /// `CWeatherParticleCloud::Update` this frame.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`mSecondsElapsed`)
    pub mSecondsElapsed: f32,
    /// Raven `mParticlesRendered` — the debug counter each cloud's `Render`
    /// accumulates into (`if (false) Com_Printf(...)` — Raven's own dead
    /// debug print, preserved verbatim per porting-rules §A2).
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp` (`mParticlesRendered`)
    pub mParticlesRendered: i32,
}

/// Raven `CWeatherParticleCloud` — one weather-particle emitter (rain/snow
/// cloud): spawn-plane geometry, per-particle physics ranges, and the owned
/// particle buffer.
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.cpp:827-892`
pub struct CWeatherParticleCloud {
    // PORT-NOTE: Raven's `mImage` is a raw image pointer, null (`0`) by
    // default; the interior-safety law forbids raw pointers, so it becomes
    // an optional handle into `RenderAssets::images`.
    pub mImage: Option<ImageHandle>,
    pub mParticleCount: i32,
    // PORT-NOTE: Raven's `mParticles` is a `new[]`/`delete[]`-owned heap array
    // of `CWeatherParticle`; an owned `Vec` replaces the manual alloc/free
    // (porting-rules §C9).
    pub mParticles: Vec<CWeatherParticle>,
    pub mPopulated: bool,
    pub mOrientWithVelocity: bool,
    pub mWaterParticles: bool,
    pub mSpawnPlaneDistance: f32,
    pub mSpawnPlaneSize: f32,
    pub mSpawnRange: SVecRange,
    pub mGravity: f32,
    // Raven `CVec4 mColor` — RGBA; `Reset`'s `mColor = 1.0f` is a CVec4
    // broadcast assign.
    pub mColor: [f32; 4],
    pub mVertexCount: i32,
    pub mWidth: f32,
    pub mHeight: f32,
    pub mBlendMode: i32,
    pub mFilterMode: i32,
    pub mFade: f32,
    pub mRotation: SFloatRange,
    pub mRotationDelta: f32,
    pub mRotationDeltaTarget: f32,
    pub mRotationCurrent: f32,
    pub mRotationChangeNext: i32,
    pub mRotationChangeTimer: SIntRange,
    pub mMass: SFloatRange,
    pub mFrictionInverse: f32,
    // PORT-NOTE: the fields below aren't set by `Reset` (Raven's own
    // `CWeatherParticleCloud::Reset` body doesn't touch them either — they
    // are only ever written by `Update`, every call, before being read) and
    // so aren't part of this packet's `Reset` excerpt or this type's
    // originally-ported field set; added here because `Update`/`Render`
    // (this wave) need them. Zero-initialized in `new()`, matching every
    // caller's unconditional overwrite before first read.
    /// Raven `mCameraPosition` — this frame's view origin.
    pub mCameraPosition: vec3_t,
    /// Raven `mCameraForward` — this frame's view forward axis.
    pub mCameraForward: vec3_t,
    /// Raven `mCameraLeft` — this frame's view left axis, scaled by
    /// `mWidth`/rotated by the cloud's billboard rotation.
    pub mCameraLeft: vec3_t,
    /// Raven `mCameraDown` — this frame's view down axis, scaled by
    /// `mHeight`/rotated by the cloud's billboard rotation.
    pub mCameraDown: vec3_t,
    /// Raven `mRange` — this frame's spawn/despawn box, `mCameraPosition +
    /// mSpawnRange`.
    pub mRange: SVecRange,
    /// Raven `mCameraLeftPlusUp` — `mCameraLeft - mCameraDown` (`mVertexCount
    /// == 4`) or `mCameraDown + mCameraLeft` otherwise.
    pub mCameraLeftPlusUp: vec3_t,
    /// Raven `mCameraLeftMinusUp` — `mCameraLeft + mCameraDown`
    /// (`mVertexCount == 4` only).
    pub mCameraLeftMinusUp: vec3_t,
    /// Raven `mSpawnPlaneNorm` — normalized `force` when `UseSpawnPlane()`.
    pub mSpawnPlaneNorm: vec3_t,
    /// Raven `mSpawnSpeed` — `force`'s pre-normalize length.
    pub mSpawnSpeed: f32,
    /// Raven `mSpawnPlaneRight`/`mSpawnPlaneUp` — `MakeNormalVectors
    /// (mSpawnPlaneNorm)`'s output basis.
    pub mSpawnPlaneRight: vec3_t,
    pub mSpawnPlaneUp: vec3_t,
    /// Raven `mParticleCountRender` — this frame's render-eligible particle
    /// count, written by `Update` and read by `Render`.
    pub mParticleCountRender: i32,
}

impl CWeatherParticleCloud {
    /// Raven `CWeatherParticleCloud::CWeatherParticleCloud` — delegates to
    /// `Reset`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1011-1016`
    pub fn new() -> Self {
        let mut this = Self {
            mImage: None,
            mParticleCount: 0,
            mParticles: Vec::new(),
            mPopulated: false,
            mOrientWithVelocity: false,
            mWaterParticles: false,
            mSpawnPlaneDistance: 0.0,
            mSpawnPlaneSize: 0.0,
            mSpawnRange: SVecRange {
                mMins: [0.0; 3],
                mMaxs: [0.0; 3],
            },
            mGravity: 0.0,
            mColor: [0.0; 4],
            mVertexCount: 0,
            mWidth: 0.0,
            mHeight: 0.0,
            mBlendMode: 0,
            mFilterMode: 0,
            mFade: 0.0,
            mRotation: SFloatRange {
                mMin: 0.0,
                mMax: 0.0,
            },
            mRotationDelta: 0.0,
            mRotationDeltaTarget: 0.0,
            mRotationCurrent: 0.0,
            mRotationChangeNext: 0,
            mRotationChangeTimer: SIntRange { mMin: 0, mMax: 0 },
            mMass: SFloatRange {
                mMin: 0.0,
                mMax: 0.0,
            },
            mFrictionInverse: 0.0,
            mCameraPosition: [0.0; 3],
            mCameraForward: [0.0; 3],
            mCameraLeft: [0.0; 3],
            mCameraDown: [0.0; 3],
            mRange: SVecRange {
                mMins: [0.0; 3],
                mMaxs: [0.0; 3],
            },
            mCameraLeftPlusUp: [0.0; 3],
            mCameraLeftMinusUp: [0.0; 3],
            mSpawnPlaneNorm: [0.0; 3],
            mSpawnSpeed: 0.0,
            mSpawnPlaneRight: [0.0; 3],
            mSpawnPlaneUp: [0.0; 3],
            mParticleCountRender: 0,
        };
        this.Reset();
        this
    }

    /// Raven `CWeatherParticleCloud::Initialize` — "Create Image, Particles,
    /// And Setup All Values".
    ///
    /// Raven declares `int VertexCount=4`; Rust has no default arguments, so
    /// every call site passes it explicitly. `rng` carries `mMass.Pick`'s
    /// msvcrt `rand()` stream, threaded rather than reached (porting-rules
    /// §B4). `view`/`cvars`/`assets`/`models`/`image_state` thread
    /// `R_FindImageFile`/`GL_Bind`'s carriers (wave 4 / wave 0, resolved call
    /// surface LAW) rather than reaching them.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:902-945`
    #[allow(clippy::too_many_arguments)]
    pub fn Initialize(
        &mut self,
        rng: &mut Rng,
        view: &mut EngineHostView,
        cvars: &RendererCvars,
        assets: &mut RenderAssets,
        models: &RenderModels,
        image_state: &mut TrImageState,
        count: i32,
        texture_path: &str,
        vertex_count: i32,
    ) {
        self.Reset();
        debug_assert!(self.mParticleCount == 0 && self.mParticles.is_empty());
        debug_assert!(self.mImage.is_none());

        // Create The Image
        //------------------
        self.mImage = R_FindImageFile(
            view,
            cvars,
            assets,
            models,
            image_state,
            Some(texture_path),
            false,
            false,
            false,
            GL_CLAMP,
        );
        if self.mImage.is_none() {
            com_error(
                errorParm_t::ERR_DROP,
                format!("CWeatherParticleCloud: Could not texture {}", texture_path),
            );
        }

        GL_Bind(self.mImage);

        // Create The Particles
        //----------------------
        self.mParticleCount = count;
        // §19: `new CWeatherParticle[count]` is UB for a negative `count` (the
        // `spacedust` branch's `atoi` can produce one); an empty buffer is the
        // defined behavior picked here — every walk of it runs
        // `0..mParticleCount`, so none of them indexes it.
        self.mParticles = (0..count.max(0))
            .map(|_| CWeatherParticle::zeroed())
            .collect();

        for particle_num in 0..self.mParticleCount {
            // Raven's `part = &(mParticles[particleNum])` is an index here,
            // not a borrow, so `mMass.Pick` can write through it while
            // `mMass` is read (porting-rules §B5).
            let part = particle_num as usize;
            self.mParticles[part].mPosition = [0.0; 3];
            self.mParticles[part].mVelocity = [0.0; 3];
            self.mParticles[part].mAlpha = 0.0;
            self.mMass.Pick(rng, &mut self.mParticles[part].mMass);
        }

        self.mVertexCount = vertex_count;

        // DEFERRED: R4 — `mGLModeEnum = (mVertexCount==3)?GL_TRIANGLES:
        // GL_QUADS` (the `_XBOX` `GL_POINTS` branch is not this build).
        // `mGLModeEnum` is read only by `Render`, itself DEFERRED to R4, and
        // its GL wire constants have no home in this crate (no constant
        // guessing) — so the field is not on this type.
        // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:937-944
    }

    /// Raven `CWeatherParticleCloud::Reset`.
    ///
    /// Raven's body: "// TODO: Free Image?" — an unresolved Raven-side TODO,
    /// preserved verbatim (porting-rules §A2: no speculative behavior).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:951-1006`
    pub fn Reset(&mut self) {
        // TODO: Free Image?
        self.mImage = None;
        if self.mParticleCount != 0 {
            self.mParticles.clear();
        }
        self.mParticleCount = 0;
        self.mParticles = Vec::new();

        self.mPopulated = false;

        // These Are The Default Startup Values For Constant Data
        //========================================================
        self.mOrientWithVelocity = false;
        self.mWaterParticles = false;

        self.mSpawnPlaneDistance = 500.0;
        self.mSpawnPlaneSize = 500.0;
        self.mSpawnRange.mMins = [-(self.mSpawnPlaneDistance * 1.25); 3];
        self.mSpawnRange.mMaxs = [self.mSpawnPlaneDistance * 1.25; 3];

        self.mGravity = 300.0; // Units Per Second

        self.mColor = [1.0; 4];

        self.mVertexCount = 4;
        self.mWidth = 1.0;
        self.mHeight = 1.0;

        self.mBlendMode = 0;
        self.mFilterMode = 0;

        self.mFade = 10.0;

        self.mRotation.Clear();
        self.mRotationDelta = 0.0;
        self.mRotationDeltaTarget = 0.0;
        self.mRotationCurrent = 0.0;
        self.mRotationChangeNext = -1;
        self.mRotation.mMin = -0.7;
        self.mRotation.mMax = 0.7;
        self.mRotationChangeTimer.mMin = 500;
        // Raven writes `mMin` twice (never sets `mMax`) — a real Raven bug,
        // preserved faithfully (porting-rules §A2).
        self.mRotationChangeTimer.mMin = 2000;

        self.mMass.mMin = 5.0;
        self.mMass.mMax = 10.0;

        self.mFrictionInverse = 0.7; // No Friction?
    }

    /// Raven `CWeatherParticleCloud::UseSpawnPlane`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1030-1033`
    pub fn UseSpawnPlane(&self) -> bool {
        self.mGravity != 0.0
    }

    /// Raven `CWeatherParticleCloud::Update`.
    ///
    /// `view_origin` and `view_axis` are `backEnd.viewParms.ori.origin` and `.axis`, threaded rather than reached (porting-rules §B4).
    /// `RE_RenderScene` fills that orientation straight from the scene refdef (`oracle/codemp/renderer/tr_scene.cpp:848-851`), so the refdef gives the identical values.
    /// The `orientationr_t` marker is therefore moot, not stale: the placeholder `ViewParms` still carries no `ori` field, and this fn takes the value from the refdef instead.
    /// `outside` is `mOutside`, `frozen` is `mFrozen`, `wind_velocity` is `mGlobalWindVelocity`, and `seconds_elapsed` is `mSecondsElapsed`.
    /// `CVec3`'s `+=`, `-=`, `*=` and `ScaleAdd` are expanded component-wise, because `vec3_t` is `[f32; 3]` under the interior-safety law.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1039-1306`
    #[allow(clippy::too_many_arguments)]
    pub fn Update(
        &mut self,
        rng: &mut Rng,
        outside: &COutside,
        view_origin: vec3_t,
        view_axis: [vec3_t; 3],
        frozen: bool,
        wind_velocity: vec3_t,
        seconds_elapsed: f32,
    ) {
        // Raven computes this once per cloud per frame, ahead of the freeze gate below.
        // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1050
        let particle_fade = self.mFade * seconds_elapsed;

        // Compute Camera
        //----------------
        self.mCameraPosition = view_origin;
        self.mCameraForward = view_axis[0];
        self.mCameraLeft = view_axis[1];
        self.mCameraDown = view_axis[2];

        if self.mRotationChangeNext != -1 {
            if self.mRotationChangeNext == 0 {
                // The two picks draw from different streams: `mRotation` from the C runtime `rand`, and `mRotationChangeTimer` from `holdrand`.
                self.mRotation.Pick(rng, &mut self.mRotationDeltaTarget);
                // `Reset` writes `mRotationChangeTimer.mMin` twice and never writes `mMax`, so the range is reversed: min 2000, max 0.
                // `Q_irand` increments `max` first, so `((result * -1999) >> 15) + 2000` over `result` in `[0, 32767]` lands uniformly in `[1, 2000]`.
                // The clamp below therefore never fires, and the rotation interval varies.
                // Source: oracle/codemp/game/q_math.c:1464-1467
                self.mRotationChangeNext = self.mRotationChangeTimer.Pick(rng);
                if self.mRotationChangeNext <= 0 {
                    self.mRotationChangeNext = 1;
                }
            }
            self.mRotationChangeNext -= 1;

            let rotation_delta_difference = self.mRotationDeltaTarget - self.mRotationDelta;
            // `0.01` is a double literal, so `fabsf`'s float result promotes
            // and the comparison runs in f64 (ruling 12).
            if rotation_delta_difference.abs() as f64 > 0.01 {
                self.mRotationDelta += rotation_delta_difference; // Blend To New Delta
            }
            self.mRotationCurrent += self.mRotationDelta * seconds_elapsed;
            let s = self.mRotationCurrent.sin();
            let c = self.mRotationCurrent.cos();

            let temp_cam_left = self.mCameraLeft;

            for i in 0..3 {
                self.mCameraLeft[i] *= c * self.mWidth;
            }
            for i in 0..3 {
                self.mCameraLeft[i] += self.mCameraDown[i] * (s * self.mWidth * -1.0);
            }

            for i in 0..3 {
                self.mCameraDown[i] *= c * self.mHeight;
            }
            for i in 0..3 {
                self.mCameraDown[i] += temp_cam_left[i] * (s * self.mHeight);
            }
        } else {
            for i in 0..3 {
                self.mCameraLeft[i] *= self.mWidth;
                self.mCameraDown[i] *= self.mHeight;
            }
        }

        // Compute Global Force
        //----------------------
        let mut force: vec3_t = [0.0, 0.0, -1.0 * self.mGravity];
        for i in 0..3 {
            force[i] += wind_velocity[i];
        }

        // Update Range
        //--------------
        for i in 0..3 {
            self.mRange.mMins[i] = self.mCameraPosition[i] + self.mSpawnRange.mMins[i];
            self.mRange.mMaxs[i] = self.mCameraPosition[i] + self.mSpawnRange.mMaxs[i];
        }

        // If Using A Spawn Plane, Increase The Range Box A Bit To Account For Rotation On The Spawn Plane
        //-------------------------------------------------------------------------------------------------
        if self.UseSpawnPlane() {
            for dim in 0..3 {
                // `±0.01` are double literals, so `force[dim]` promotes and
                // both comparisons run in f64 (ruling 12).
                if force[dim] as f64 > 0.01 {
                    self.mRange.mMins[dim] -= self.mSpawnPlaneDistance / 2.0;
                } else if (force[dim] as f64) < -0.01 {
                    self.mRange.mMaxs[dim] += self.mSpawnPlaneDistance / 2.0;
                }
            }
            self.mSpawnPlaneNorm = force;
            self.mSpawnSpeed = VectorNormalize(&mut self.mSpawnPlaneNorm);
            MakeNormalVectors(
                self.mSpawnPlaneNorm,
                &mut self.mSpawnPlaneRight,
                &mut self.mSpawnPlaneUp,
            );
            if self.mOrientWithVelocity {
                self.mCameraDown = self.mSpawnPlaneNorm;
                for i in 0..3 {
                    self.mCameraDown[i] *= self.mHeight * -1.0;
                }
            }
        }

        // Optimization For Quad Position Calculation
        //--------------------------------------------
        if self.mVertexCount == 4 {
            for i in 0..3 {
                self.mCameraLeftPlusUp[i] = self.mCameraLeft[i] - self.mCameraDown[i];
                self.mCameraLeftMinusUp[i] = self.mCameraLeft[i] + self.mCameraDown[i];
            }
        } else {
            // should really be called mCamera Left + Down
            for i in 0..3 {
                self.mCameraLeftPlusUp[i] = self.mCameraDown[i] + self.mCameraLeft[i];
            }
        }

        // Stop All Additional Processing
        //--------------------------------
        if frozen {
            return;
        }

        // Now Update All Particles
        //--------------------------
        // Raven's `ratl::bits_vs` `get_bit`, `set_bit` and `clear_bit` become mask tests and mask updates on the `u32` bit set.
        let flag_render = 1u32 << CWeatherParticle::FLAG_RENDER;
        let flag_fadein = 1u32 << CWeatherParticle::FLAG_FADEIN;
        let flag_fadeout = 1u32 << CWeatherParticle::FLAG_FADEOUT;

        self.mParticleCountRender = 0;
        for particle_num in 0..self.mParticleCount {
            // Raven's `part = &mParticles[particleNum]` is an index here, not a borrow, so the range picks can write through it (porting-rules §B5).
            let part = particle_num as usize;

            if !self.mPopulated {
                // First Time Spawn Location
                self.mRange.Pick(rng, &mut self.mParticles[part].mPosition);
            }

            // Grab The Force And Apply Non Global Wind
            //------------------------------------------
            let mut part_force = force;
            for i in 0..3 {
                part_force[i] /= self.mParticles[part].mMass;
            }

            // Apply The Force
            //-----------------
            // The force and the friction apply once per frame with no time factor, and only the position step is time-scaled.
            // The result is frame-rate dependent in the oracle too.
            for i in 0..3 {
                self.mParticles[part].mVelocity[i] += part_force[i];
                self.mParticles[part].mVelocity[i] *= self.mFrictionInverse;
            }
            for i in 0..3 {
                self.mParticles[part].mPosition[i] +=
                    self.mParticles[part].mVelocity[i] * seconds_elapsed;
            }

            let part_to_camera: vec3_t = [
                self.mParticles[part].mPosition[0] - self.mCameraPosition[0],
                self.mParticles[part].mPosition[1] - self.mCameraPosition[1],
                self.mParticles[part].mPosition[2] - self.mCameraPosition[2],
            ];
            let mut part_rendering = self.mParticles[part].mFlags & flag_render != 0;
            // Raven calls the three-argument `PointOutside` overload, which never tests `mCacheInit`.
            let part_outside = outside.PointOutsideBounded(
                &self.mParticles[part].mPosition,
                self.mWidth,
                self.mHeight,
            );
            let mut part_in_range = self.mRange.In(&self.mParticles[part].mPosition);
            // This is a half-space test against the camera forward axis, not a frustum test.
            let part_in_view = part_outside
                && part_in_range
                && _DotProduct(part_to_camera, self.mCameraForward) > 0.0;

            // Process Respawn
            //-----------------
            if !part_in_range && !part_rendering {
                self.mParticles[part].mVelocity = [0.0; 3];

                // Reselect A Position On The Spawn Plane
                //----------------------------------------
                if self.UseSpawnPlane() {
                    self.mParticles[part].mPosition = self.mCameraPosition;
                    // `CVec3` has no binary scalar multiply, so the float converts through the broadcast constructor and this line draws nothing.
                    // Source: oracle/codemp/Ravl/CVec.h:570,628
                    for i in 0..3 {
                        self.mParticles[part].mPosition[i] -=
                            self.mSpawnPlaneNorm[i] * self.mSpawnPlaneDistance;
                    }
                    // The same broadcast makes each of the next two lines exactly one `WE_flrand` draw, scaling x, y and z alike.
                    // A per-component draw would take six values instead of two and shift the stream for the rest of the session.
                    // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1216-1217
                    let right_scale = WE_flrand(rng, -self.mSpawnPlaneSize, self.mSpawnPlaneSize);
                    for i in 0..3 {
                        self.mParticles[part].mPosition[i] +=
                            self.mSpawnPlaneRight[i] * right_scale;
                    }
                    let up_scale = WE_flrand(rng, -self.mSpawnPlaneSize, self.mSpawnPlaneSize);
                    for i in 0..3 {
                        self.mParticles[part].mPosition[i] += self.mSpawnPlaneUp[i] * up_scale;
                    }
                }
                // Otherwise, Just Wrap Around To The Other End Of The Range
                //-----------------------------------------------------------
                else {
                    self.mRange
                        .Wrap(&mut self.mParticles[part].mPosition, &mut self.mSpawnRange);
                }
                // Raven's store is dead: the fade machine and the render count below read only `partRendering`, `partInView` and `part->mFlags`.
                // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1226
                #[allow(unused_assignments)]
                {
                    part_in_range = true;
                }
            }

            // Process Fade
            //--------------
            {
                // Start A Fade Out
                //------------------
                if part_rendering && !part_in_view {
                    self.mParticles[part].mFlags &= !flag_fadein;
                    self.mParticles[part].mFlags |= flag_fadeout;
                }
                // Switch From Fade Out To Fade In
                //---------------------------------
                else if part_rendering
                    && part_in_view
                    && self.mParticles[part].mFlags & flag_fadeout != 0
                {
                    self.mParticles[part].mFlags |= flag_fadein;
                    self.mParticles[part].mFlags &= !flag_fadeout;
                }
                // Start A Fade In
                //-----------------
                else if !part_rendering && part_in_view {
                    part_rendering = true;
                    self.mParticles[part].mAlpha = 0.0;
                    self.mParticles[part].mFlags |= flag_render;
                    self.mParticles[part].mFlags |= flag_fadein;
                    self.mParticles[part].mFlags &= !flag_fadeout;
                }

                // Update Fade
                //-------------
                if part_rendering {
                    // Update Fade Out
                    //-----------------
                    if self.mParticles[part].mFlags & flag_fadeout != 0 {
                        self.mParticles[part].mAlpha -= particle_fade;
                        if self.mParticles[part].mAlpha <= 0.0 {
                            self.mParticles[part].mAlpha = 0.0;
                            self.mParticles[part].mFlags &= !flag_fadeout;
                            self.mParticles[part].mFlags &= !flag_fadein;
                            self.mParticles[part].mFlags &= !flag_render;
                            // This store is dead too: the render count below re-reads the flag, and the next particle re-initializes the local.
                            #[allow(unused_assignments)]
                            {
                                part_rendering = false;
                            }
                        }
                    }
                    // Update Fade In
                    //----------------
                    else if self.mParticles[part].mFlags & flag_fadein != 0 {
                        // The alpha ceiling is `mColor[3]`, not 1.0.
                        self.mParticles[part].mAlpha += particle_fade;
                        if self.mParticles[part].mAlpha >= self.mColor[3] {
                            self.mParticles[part].mFlags &= !flag_fadein;
                            self.mParticles[part].mAlpha = self.mColor[3];
                        }
                    }
                }
            }

            // Keep Track Of The Number Of Particles To Render
            //-------------------------------------------------
            // Raven re-reads the flag rather than the local, so a particle that faded out this frame is not counted.
            if self.mParticles[part].mFlags & flag_render != 0 {
                self.mParticleCountRender += 1;
            }
        }
        self.mPopulated = true;
    }

    /// Raven `CWeatherParticleCloud::Render`.
    ///
    /// DEFERRED: R4 — the entire body is immediate-mode GL drawing
    /// (`qglBegin`/`qglColor4f`/`qglVertex3f*`/`qglTexCoord2f`/`qglEnd`/
    /// `qglPushMatrix`/`qglPopMatrix`/`qglPointSize`/`qglPointParameterf*EXT`/
    /// `qglEnable`/`qglMatrixMode`/`qglTexParameterf`), gated on GL wire
    /// constants (`GLS_ALPHA`, `GL_POINTS`, `GL_MODELVIEW`, `GL_TEXTURE_2D`,
    /// `GL_TEXTURE_MIN_FILTER`, `GL_TEXTURE_MAG_FILTER`, `GL_LINEAR`,
    /// `GL_NEAREST`, `GL_POINT_SIZE_MIN_EXT`, `GL_POINT_SIZE_MAX_EXT`,
    /// `GL_DISTANCE_ATTENUATION_EXT`) this packet does not carry — never
    /// guessed (porting-rules: no numeric-constant guessing). The in-module
    /// callees (`GL_State`/`GL_Bind`/`GL_Cull`) are themselves already
    /// DEFERRED no-ops at their own definitions (`tr_backend.rs`, DEC-37
    /// A13.2); R2 leaves the fixed-function GL surface with no R3 home — it
    /// dissolves into R4's wgpu rewrite (the render thread owns the GL
    /// binding cache, DEC-63.4). Only the CPU-only counter effect survives
    /// this wave.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1311-1480`
    pub fn Render(&self, particles_rendered: &mut i32) {
        *particles_rendered += self.mParticleCountRender;
    }
}

impl Default for CWeatherParticleCloud {
    fn default() -> Self {
        Self::new()
    }
}

/// Raven `CWeatherParticleCloud::~CWeatherParticleCloud` — delegates to
/// `Reset`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1021-1024`
impl Drop for CWeatherParticleCloud {
    fn drop(&mut self) {
        self.Reset();
    }
}

impl WorldEffectsState {
    /// Raven `R_AddWeatherZone`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:709-712`
    pub fn R_AddWeatherZone(&mut self, mins: vec3_t, maxs: vec3_t) {
        self.mOutside.AddWeatherZone(mins, maxs);
    }

    /// Raven `R_IsOutside`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:714-717`
    pub fn R_IsOutside(&self, cm: &mut CollisionWorld, pos: &vec3_t) -> bool {
        self.mOutside.PointOutside(cm, pos)
    }

    /// Raven `R_IsShaking`.
    ///
    /// `origin` is Raven's `backEnd.viewParms.ori.origin`, threaded rather
    /// than reached (porting-rules §B4).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:719-722`
    pub fn R_IsShaking(&self, cm: &mut CollisionWorld, origin: &vec3_t) -> bool {
        self.mOutside.mOutsideShake && self.mOutside.PointOutside(cm, origin)
    }

    /// Raven `R_IsOutsideCausingPain`.
    ///
    /// Raven returns `float` from a `&&` expression, so the result is only
    /// ever `0.0`/`1.0` — preserved as written (porting-rules §A2).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:724-727`
    pub fn R_IsOutsideCausingPain(&self, cm: &mut CollisionWorld, pos: &vec3_t) -> f32 {
        (self.mOutside.mOutsidePain != 0.0 && self.mOutside.PointOutside(cm, pos)) as i32 as f32
    }

    /// Raven `R_InitWorldEffects`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1489-1500`
    pub fn R_InitWorldEffects(&mut self, host: &mut EngineHostView) {
        self.rng.srand(Com_Milliseconds(host) as u32);

        for cloud in &mut self.mParticleClouds {
            cloud.Reset();
        }
        self.mParticleClouds.clear();
        self.mWindZones.clear();
        self.mOutside.Reset();
    }

    /// Raven `RB_RenderWorldEffects`.
    ///
    /// `assets` is the sim-published `RenderAssets`, which carries `tr.world`, and `host` carries `mOutside.Cache`'s engine and collision access plus `Com_Printf`'s `Common`.
    /// Both are threaded rather than reached (porting-rules §B4).
    ///
    /// `refdef` is the submitted scene's own refdef, which carries `rdflags` and `frametime`.
    /// Raven reads `RDF_NOWORLDMODEL` off `tr.refdef` and `RDF_SKYBOXPORTAL` off `backEnd.refdef`, two copies that hold different scenes at backend time.
    /// The port has one refdef per scene and reads both bits off it.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1513-1580`
    pub fn RB_RenderWorldEffects(
        &mut self,
        assets: &RenderAssets,
        refdef: &TrRefdef,
        host: &mut EngineHostView,
    ) {
        // Raven: "no world rendering or no world or no particle clouds"
        //
        // Raven's `||` chain short-circuits left to right in C, and so does Rust's, so the four terms keep their oracle order.
        // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1515-1521
        if assets.world.is_none()
            || refdef.rdflags & RDF_NOWORLDMODEL != 0
            || refdef.rdflags & RDF_SKYBOXPORTAL != 0
            || self.mParticleClouds.is_empty()
        {
            return;
        }

        // `SetViewportAndScissor` is retired at this site. The render pass sets the viewport and the scissor from the same view.
        // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1523
        // DEFERRED: R4 — qglMatrixMode(GL_MODELVIEW)/qglLoadMatrixf(backEnd
        // .viewParms.world.modelMatrix): fixed-function GL surface, no R3
        // home (DEC-37 A13.2); `viewParms_t::world.modelMatrix` is also not
        // yet a field on the landed `ViewParms` (`tr_main` R3 wave).
        // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1524-1525

        // Calculate Elapsed Time For Scale Purposes
        //-------------------------------------------
        // The 1 ms floor below means a zero never divides by zero.
        // It makes the weather crawl at one thousandth speed instead.
        // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1530
        self.mMillisecondsElapsed = refdef.frametime as f32;
        if self.mMillisecondsElapsed < 1.0 {
            self.mMillisecondsElapsed = 1.0;
        }
        if self.mMillisecondsElapsed > 1000.0 {
            self.mMillisecondsElapsed = 1000.0;
        }
        self.mSecondsElapsed = self.mMillisecondsElapsed / 1000.0;

        // Make Sure We Are Always Outside Cached
        //----------------------------------------
        if !self.mOutside.Initialized() {
            // Submodel 0 is the worldspawn brush model, so these bounds are the whole map.
            // Raven indexes `bmodels[0]` without a length test, which reads out of bounds for a world with no submodel.
            // §19 picks `None` for that case, which leaves the cache unbuilt rather than panicking.
            // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1546
            let world_bmodel_bounds = assets
                .world
                .as_ref()
                .and_then(|w| w.bmodels.first())
                .map(|b| b.bounds);
            self.mOutside.Cache(host, world_bmodel_bounds);
        } else {
            // Update All Wind Zones
            //-----------------------
            if !self.mFrozen {
                self.wind.global_wind_velocity = [0.0; 3];
                for wz in 0..self.mWindZones.len() {
                    self.mWindZones[wz].Update(&mut self.rng);
                    if self.mWindZones[wz].mGlobal {
                        for i in 0..3 {
                            self.wind.global_wind_velocity[i] +=
                                self.mWindZones[wz].mCurrentVelocity[i];
                        }
                    }
                }
                self.wind.global_wind_direction = self.wind.global_wind_velocity;
                self.wind.global_wind_speed =
                    VectorNormalize(&mut self.wind.global_wind_direction);
            }

            // Update All Particle Clouds
            //----------------------------
            self.mParticlesRendered = 0;
            let frozen = self.mFrozen;
            let seconds_elapsed = self.mSecondsElapsed;
            let wind_velocity = self.wind.global_wind_velocity;
            let view_origin = refdef.view_origin;
            let view_axis = refdef.view_axis;
            for i in 0..self.mParticleClouds.len() {
                self.mParticleClouds[i].Update(
                    &mut self.rng,
                    &self.mOutside,
                    view_origin,
                    view_axis,
                    frozen,
                    wind_velocity,
                    seconds_elapsed,
                );
                self.mParticleClouds[i].Render(&mut self.mParticlesRendered);
            }
            if false {
                com_printf(
                    host.common,
                    &format!("Weather: {} Particles Rendered\n", self.mParticlesRendered),
                );
            }
        }
    }

    /// Raven `R_GetChanceOfSaberFizz`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1990-2007`
    pub fn R_GetChanceOfSaberFizz(&self) -> f32 {
        let mut chance = 0.0f32;
        let mut num_water = 0i32;
        for cloud in &self.mParticleClouds {
            if cloud.mWaterParticles {
                chance += cloud.mGravity / 20000.0;
                num_water += 1;
            }
        }
        if num_water != 0 {
            return chance / num_water as f32;
        }
        0.0
    }

    /// Raven `R_IsRaining`.
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:2009-2012`
    pub fn R_IsRaining(&self) -> bool {
        !self.mParticleClouds.is_empty()
    }

    /// Raven `R_WorldEffectCommand`.
    ///
    /// `qs` is `COM_ParseExt`/`ParseVector`'s parse scratch
    /// (`q_shared.c`'s TU-invisible `com_lines`/`com_parsename` statics,
    /// already relocated off any Rust global — porting-rules §B3/§B4);
    /// `host` carries `Com_Printf`'s `Common` (via `host.common`) for the
    /// same reason. `command` is Raven's `const char *command`, threaded as
    /// a byte-slice cursor mutated by repeated `COM_ParseExt` calls in place
    /// of the `const char **text` out-param idiom (porting-rules §C7).
    ///
    /// `ParseVector` names a shader only to format its two warning strings.
    /// Raven's call site here reads whatever `shader.name` happened to hold
    /// left over from the last shader parse — residual state this fn cannot
    /// reconstruct, and with no bearing on control flow (only the returned
    /// `bool` and the written `v[]` matter). An empty name is passed; the
    /// only externally-visible difference is the text of an unrelated debug
    /// warning, never a return value or written vector component.
    ///
    /// `strcmpi` -> `str::eq_ignore_ascii_case` matches this crate's
    /// established idiom (`tr_shader.rs`'s `NameToAFunc`/`NameToSrcBlendMode`
    /// etc.), not a `native_string` call.
    ///
    /// Every cloud-spawning branch's `mParticleClouds.push_back()` (which
    /// default-constructs in place, then is filled through the returned
    /// reference) becomes a `CWeatherParticleCloud::new()` local pushed once
    /// its fields are written.
    /// `cvars`/`assets`/`models`/`image_state` thread the carriers each
    /// `CWeatherParticleCloud::Initialize` call needs (wave 5 reconciliation:
    /// `R_FindImageFile`/`GL_Bind` are now real calls, not deferrals).
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1593-1986`
    #[allow(clippy::too_many_arguments)]
    pub fn R_WorldEffectCommand(
        &mut self,
        qs: &mut QSharedScratch,
        host: &mut EngineHostView,
        cvars: &RendererCvars,
        assets: &mut RenderAssets,
        models: &RenderModels,
        image_state: &mut TrImageState,
        command: Option<&[u8]>,
    ) {
        if command.is_none() {
            return;
        }
        let mut cursor = command;

        let (token, rest) = COM_ParseExt(qs, cursor, false);
        cursor = rest;
        // The oracle's `if (!token) return;` is dead code — `COM_ParseExt` never
        // returns NULL, only the empty string — so an empty token falls through
        // every `strcmpi` to the trailing help-print `else`, as it does here.

        // Die - clean up the whole weather system -rww
        if token.eq_ignore_ascii_case("die") {
            R_ShutdownWorldEffects(self, host);
            return;
        }
        // Clear - Removes All Particle Clouds And Wind Zones
        //----------------------------------------------------
        else if token.eq_ignore_ascii_case("clear") {
            for cloud in &mut self.mParticleClouds {
                cloud.Reset();
            }
            self.mParticleClouds.clear();
            self.mWindZones.clear();
        }
        // Freeze / UnFreeze - Stops All Particle Motion Updates
        //--------------------------------------------------------
        else if token.eq_ignore_ascii_case("freeze") {
            self.mFrozen = !self.mFrozen;
        }
        // Add a zone
        //---------------
        else if token.eq_ignore_ascii_case("zone") {
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            if ParseVector(qs, &mut cursor, host.common, "", 3, &mut mins)
                && ParseVector(qs, &mut cursor, host.common, "", 3, &mut maxs)
            {
                self.mOutside.AddWeatherZone(mins, maxs);
            }
        }
        // Basic Wind
        //------------
        else if token.eq_ignore_ascii_case("wind") {
            if self.mWindZones.len() >= MAX_WIND_ZONES {
                return;
            }
            let mut n_wind = CWindZone::zeroed();
            n_wind.Initialize();
            self.mWindZones.push(n_wind);
        }
        // Constant Wind
        //---------------
        else if token.eq_ignore_ascii_case("constantwind") {
            if self.mWindZones.len() >= MAX_WIND_ZONES {
                return;
            }
            let mut n_wind = CWindZone::zeroed();
            n_wind.Initialize();
            if !ParseVector(
                qs,
                &mut cursor,
                host.common,
                "",
                3,
                &mut n_wind.mCurrentVelocity,
            ) {
                n_wind.mCurrentVelocity = [0.0; 3];
                n_wind.mCurrentVelocity[1] = 800.0;
            }
            n_wind.mTargetVelocityTimeRemaining = -1;
            self.mWindZones.push(n_wind);
        }
        // Gusting Wind
        //--------------
        else if token.eq_ignore_ascii_case("gustingwind") {
            if self.mWindZones.len() >= MAX_WIND_ZONES {
                return;
            }
            let mut n_wind = CWindZone::zeroed();
            n_wind.Initialize();
            n_wind.mRVelocity.mMins = [-3000.0; 3];
            n_wind.mRVelocity.mMins[2] = -100.0;
            n_wind.mRVelocity.mMaxs = [3000.0; 3];
            n_wind.mRVelocity.mMaxs[2] = 100.0;

            n_wind.mMaxDeltaVelocityPerUpdate = 10.0;

            n_wind.mRDuration.mMin = 1000;
            n_wind.mRDuration.mMax = 3000;

            n_wind.mChanceOfDeadTime = 0.5;
            n_wind.mRDeadTime.mMin = 2000;
            n_wind.mRDeadTime.mMax = 4000;

            self.mWindZones.push(n_wind);
        }
        // Create A Rain Storm
        //---------------------
        else if token.eq_ignore_ascii_case("lightrain") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                500,
                "gfx/world/rain.jpg",
                3,
            );
            n_cloud.mHeight = 80.0;
            n_cloud.mWidth = 1.2;
            n_cloud.mGravity = 2000.0;
            n_cloud.mFilterMode = 1;
            n_cloud.mBlendMode = 1;
            n_cloud.mFade = 100.0;
            n_cloud.mColor = [0.5; 4];
            n_cloud.mOrientWithVelocity = true;
            n_cloud.mWaterParticles = true;
            self.mParticleClouds.push(n_cloud);
        }
        // Create A Rain Storm
        //---------------------
        else if token.eq_ignore_ascii_case("rain") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                1000,
                "gfx/world/rain.jpg",
                3,
            );
            n_cloud.mHeight = 80.0;
            n_cloud.mWidth = 1.2;
            n_cloud.mGravity = 2000.0;
            n_cloud.mFilterMode = 1;
            n_cloud.mBlendMode = 1;
            n_cloud.mFade = 100.0;
            n_cloud.mColor = [0.5; 4];
            n_cloud.mOrientWithVelocity = true;
            n_cloud.mWaterParticles = true;
            self.mParticleClouds.push(n_cloud);
        }
        // Create A Rain Storm
        //---------------------
        else if token.eq_ignore_ascii_case("acidrain") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                1000,
                "gfx/world/rain.jpg",
                3,
            );
            n_cloud.mHeight = 80.0;
            n_cloud.mWidth = 2.0;
            n_cloud.mGravity = 2000.0;
            n_cloud.mFilterMode = 1;
            n_cloud.mBlendMode = 1;
            n_cloud.mFade = 100.0;

            n_cloud.mColor[0] = 0.34;
            n_cloud.mColor[1] = 0.70;
            n_cloud.mColor[2] = 0.34;
            n_cloud.mColor[3] = 0.70;

            n_cloud.mOrientWithVelocity = true;
            n_cloud.mWaterParticles = true;
            self.mParticleClouds.push(n_cloud);

            self.mOutside.mOutsidePain = 0.1;
        }
        // Create A Rain Storm
        //---------------------
        else if token.eq_ignore_ascii_case("heavyrain") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                1000,
                "gfx/world/rain.jpg",
                3,
            );
            n_cloud.mHeight = 80.0;
            n_cloud.mWidth = 1.2;
            n_cloud.mGravity = 2800.0;
            n_cloud.mFilterMode = 1;
            n_cloud.mBlendMode = 1;
            n_cloud.mFade = 15.0;
            n_cloud.mColor = [0.5; 4];
            n_cloud.mOrientWithVelocity = true;
            n_cloud.mWaterParticles = true;
            self.mParticleClouds.push(n_cloud);
        }
        // Create A Snow Storm
        //---------------------
        else if token.eq_ignore_ascii_case("snow") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            // The PC `#else` arm: `Initialize(1000, …)` takes Raven's default
            // `VertexCount=4` (the `_XBOX` arm's `1` and its `mWidth = 0.05f`
            // are not this build).
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                1000,
                "gfx/effects/snowflake1.bmp",
                4,
            );
            n_cloud.mBlendMode = 1;
            n_cloud.mRotationChangeNext = 0;
            n_cloud.mColor = [0.75; 4];
            n_cloud.mWaterParticles = true;
            self.mParticleClouds.push(n_cloud);
        }
        // Create A Some stuff
        //---------------------
        else if token.eq_ignore_ascii_case("spacedust") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            // Raven advances `command` past the count token; this is the last
            // parse in the fn, so the advanced cursor is never read again.
            let (count_token, _) = COM_ParseExt(qs, cursor, false);
            let count = atoi(&count_token);

            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                count,
                "gfx/effects/snowpuff1.tga",
                4,
            );
            n_cloud.mHeight = 1.2;
            n_cloud.mWidth = 1.2;
            n_cloud.mGravity = 0.0;
            n_cloud.mBlendMode = 1;
            n_cloud.mRotationChangeNext = 0;
            n_cloud.mColor = [0.75; 4];
            n_cloud.mWaterParticles = true;
            n_cloud.mMass.mMax = 30.0;
            n_cloud.mMass.mMin = 10.0;
            n_cloud.mSpawnRange.mMins[0] = -1500.0;
            n_cloud.mSpawnRange.mMins[1] = -1500.0;
            n_cloud.mSpawnRange.mMins[2] = -1500.0;
            n_cloud.mSpawnRange.mMaxs[0] = 1500.0;
            n_cloud.mSpawnRange.mMaxs[1] = 1500.0;
            n_cloud.mSpawnRange.mMaxs[2] = 1500.0;
            self.mParticleClouds.push(n_cloud);
        }
        // Create A Sand Storm
        //---------------------
        else if token.eq_ignore_ascii_case("sand") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                400,
                "gfx/effects/alpha_smoke2b.tga",
                4,
            );

            n_cloud.mGravity = 0.0;
            n_cloud.mWidth = 70.0;
            n_cloud.mHeight = 70.0;
            n_cloud.mColor[0] = 0.9;
            n_cloud.mColor[1] = 0.6;
            n_cloud.mColor[2] = 0.0;
            n_cloud.mColor[3] = 0.5;
            n_cloud.mFade = 5.0;
            n_cloud.mMass.mMax = 30.0;
            n_cloud.mMass.mMin = 10.0;
            n_cloud.mSpawnRange.mMins[2] = -150.0;
            n_cloud.mSpawnRange.mMaxs[2] = 150.0;

            n_cloud.mRotationChangeNext = 0;
            self.mParticleClouds.push(n_cloud);
        }
        // Create Blowing Clouds Of Fog
        //------------------------------
        else if token.eq_ignore_ascii_case("fog") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                60,
                "gfx/effects/alpha_smoke2b.tga",
                4,
            );
            n_cloud.mBlendMode = 1;
            n_cloud.mGravity = 0.0;
            n_cloud.mWidth = 70.0;
            n_cloud.mHeight = 70.0;
            n_cloud.mColor = [0.2; 4];
            n_cloud.mFade = 5.0;
            n_cloud.mMass.mMax = 30.0;
            n_cloud.mMass.mMin = 10.0;
            n_cloud.mSpawnRange.mMins[2] = -150.0;
            n_cloud.mSpawnRange.mMaxs[2] = 150.0;

            n_cloud.mRotationChangeNext = 0;
            self.mParticleClouds.push(n_cloud);
        }
        // Create Heavy Rain Particle Cloud
        //-----------------------------------
        else if token.eq_ignore_ascii_case("heavyrainfog") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                70,
                "gfx/effects/alpha_smoke2b.tga",
                4,
            );
            n_cloud.mBlendMode = 1;
            n_cloud.mGravity = 0.0;
            n_cloud.mWidth = 100.0;
            n_cloud.mHeight = 100.0;
            n_cloud.mColor = [0.3; 4];
            n_cloud.mFade = 1.0;
            n_cloud.mMass.mMax = 10.0;
            n_cloud.mMass.mMin = 5.0;

            n_cloud.mSpawnRange.mMins = [-(n_cloud.mSpawnPlaneDistance * 1.25); 3];
            n_cloud.mSpawnRange.mMaxs = [n_cloud.mSpawnPlaneDistance * 1.25; 3];
            n_cloud.mSpawnRange.mMins[2] = -150.0;
            n_cloud.mSpawnRange.mMaxs[2] = 150.0;

            n_cloud.mRotationChangeNext = 0;
            self.mParticleClouds.push(n_cloud);
        }
        // Create Blowing Clouds Of Fog
        //------------------------------
        else if token.eq_ignore_ascii_case("light_fog") {
            if self.mParticleClouds.len() >= MAX_PARTICLE_CLOUDS {
                return;
            }
            let mut n_cloud = CWeatherParticleCloud::new();
            n_cloud.Initialize(
                &mut self.rng,
                host,
                cvars,
                assets,
                models,
                image_state,
                40,
                "gfx/effects/alpha_smoke2b.tga",
                4,
            );
            n_cloud.mBlendMode = 1;
            n_cloud.mGravity = 0.0;
            n_cloud.mWidth = 100.0;
            n_cloud.mHeight = 100.0;
            n_cloud.mColor[0] = 0.19;
            n_cloud.mColor[1] = 0.6;
            n_cloud.mColor[2] = 0.7;
            n_cloud.mColor[3] = 0.12;
            n_cloud.mFade = 0.10;
            n_cloud.mMass.mMax = 30.0;
            n_cloud.mMass.mMin = 10.0;
            n_cloud.mSpawnRange.mMins[2] = -150.0;
            n_cloud.mSpawnRange.mMaxs[2] = 150.0;

            n_cloud.mRotationChangeNext = 0;
            self.mParticleClouds.push(n_cloud);
        } else if token.eq_ignore_ascii_case("outsideshake") {
            self.mOutside.mOutsideShake = !self.mOutside.mOutsideShake;
        } else if token.eq_ignore_ascii_case("outsidepain") {
            // Raven `mOutsidePain = !mOutsidePain;` — a float `!` coerces to
            // bool first, so the result is only ever `0.0`/`1.0`, preserved
            // as written (porting-rules §A2).
            self.mOutside.mOutsidePain = if self.mOutside.mOutsidePain != 0.0 {
                0.0
            } else {
                1.0
            };
        } else {
            com_printf(
                host.common,
                "Weather Effect: Please enter a valid command.\n",
            );
            com_printf(host.common, "\tclear\n");
            com_printf(host.common, "\tfreeze\n");
            com_printf(host.common, "\tzone (mins) (maxs)\n");
            com_printf(host.common, "\twind\n");
            com_printf(host.common, "\tconstantwind (velocity)\n");
            com_printf(host.common, "\tgustingwind\n");
            com_printf(host.common, "\twindzone (mins) (maxs) (velocity)\n");
            com_printf(host.common, "\tlightrain\n");
            com_printf(host.common, "\train\n");
            com_printf(host.common, "\tacidrain\n");
            com_printf(host.common, "\theavyrain\n");
            com_printf(host.common, "\tsnow\n");
            com_printf(host.common, "\tspacedust\n");
            com_printf(host.common, "\tsand\n");
            com_printf(host.common, "\tfog\n");
            com_printf(host.common, "\theavyrainfog\n");
            com_printf(host.common, "\tlight_fog\n");
            com_printf(host.common, "\toutsideshake\n");
            com_printf(host.common, "\toutsidepain\n");
        }
    }
}

/// Raven `R_IsPuffing`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:2014-2017`
pub fn R_IsPuffing() -> bool {
    // Raven: "Eh? Don't want surfacesprites to know this?"
    false
}

/// Raven `R_ShutdownWorldEffects` — delegates to `R_InitWorldEffects`.
///
/// A bare Raven fn wrapping a class-owned method; `state`/`host` threaded in
/// rather than reached (porting-rules §B4), matching
/// `WorldEffectsState::R_InitWorldEffects`'s own signature.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1505-1508`
pub fn R_ShutdownWorldEffects(state: &mut WorldEffectsState, host: &mut EngineHostView) {
    state.R_InitWorldEffects(host);
}

/// Raven `R_WorldEffect_f` — the `worldeffect` console-command handler.
///
/// `state`/`qs`/`host` are threaded in rather than reached (porting-rules
/// §B4), matching `WorldEffectsState::R_WorldEffectCommand`'s own signature
/// (`host.common` supplies `Cvar_VariableIntegerValue`'s `Common`).
///
/// Raven's `char temp[2048]` scratch is [`Cmd_ArgsBuffer`]'s owned return; its
/// `sizeof(temp)` becomes the `buffer_length` cap. `cvars`/`assets`/`models`/
/// `image_state` thread `R_WorldEffectCommand`'s own added carriers
/// through (wave 5 reconciliation), matching its signature.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1583-1591`
#[allow(clippy::too_many_arguments)]
pub fn R_WorldEffect_f(
    state: &mut WorldEffectsState,
    qs: &mut QSharedScratch,
    host: &mut EngineHostView,
    cvars: &RendererCvars,
    assets: &mut RenderAssets,
    models: &RenderModels,
    image_state: &mut TrImageState,
) {
    if Cvar_VariableIntegerValue(host.common, "sv_cheats") != 0 {
        let temp = Cmd_ArgsBuffer(host.common, 2048);
        state.R_WorldEffectCommand(
            qs,
            host,
            cvars,
            assets,
            models,
            image_state,
            Some(temp.as_bytes()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cloud with one particle, at `Reset`'s defaults, with the rotation spin off so the camera block draws nothing.
    fn one_particle_cloud(position: vec3_t) -> CWeatherParticleCloud {
        let mut cloud = CWeatherParticleCloud::new();
        cloud.mParticleCount = 1;
        cloud.mParticles = vec![CWeatherParticle {
            mAlpha: 0.0,
            mFlags: 0,
            mPosition: position,
            mVelocity: [0.0; 3],
            mMass: 5.0,
        }];
        // The first update spawns every particle while this is false, and the spawn draws three values.
        cloud.mPopulated = true;
        cloud
    }

    const VIEW_AXIS: [vec3_t; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    /// Raven's two spawn-plane lines each make one `WE_flrand` draw, and that one value scales x, y and z alike.
    /// A per-component transcription would draw six values instead of two.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1216-1217`
    #[test]
    fn the_spawn_plane_respawn_draws_twice_and_broadcasts() {
        const SEED: u32 = 7;
        let outside = COutside::new();

        // The particle sits far outside the range box and does not render, which is the respawn condition.
        let mut cloud = one_particle_cloud([10000.0, 0.0, 0.0]);
        let size = cloud.mSpawnPlaneSize;

        let mut reference = Rng::default();
        reference.srand(SEED);
        let right_scale = WE_flrand(&mut reference, -size, size);
        let up_scale = WE_flrand(&mut reference, -size, size);
        let after_two_draws = reference.rand();

        let mut rng = Rng::default();
        rng.srand(SEED);
        cloud.Update(&mut rng, &outside, [0.0; 3], VIEW_AXIS, false, [0.0; 3], 0.02);

        assert_eq!(
            rng.rand(),
            after_two_draws,
            "the respawn must take exactly two draws off the C runtime stream",
        );

        let expected: vec3_t = [
            -cloud.mSpawnPlaneNorm[0] * cloud.mSpawnPlaneDistance
                + cloud.mSpawnPlaneRight[0] * right_scale
                + cloud.mSpawnPlaneUp[0] * up_scale,
            -cloud.mSpawnPlaneNorm[1] * cloud.mSpawnPlaneDistance
                + cloud.mSpawnPlaneRight[1] * right_scale
                + cloud.mSpawnPlaneUp[1] * up_scale,
            -cloud.mSpawnPlaneNorm[2] * cloud.mSpawnPlaneDistance
                + cloud.mSpawnPlaneRight[2] * right_scale
                + cloud.mSpawnPlaneUp[2] * up_scale,
        ];
        assert_eq!(cloud.mParticles[0].mPosition, expected);
    }

    /// `SVecRange::Wrap` folds a point back through the opposite face, one axis at a time.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:162-196`
    #[test]
    fn the_range_wraps_each_axis() {
        let range = SVecRange {
            mMins: [-10.0; 3],
            mMaxs: [10.0; 3],
        };
        let mut spawn = SVecRange {
            mMins: [0.0; 3],
            mMaxs: [0.0; 3],
        };

        // Under the minimum on x, over the maximum on y, and inside on z.
        let mut v: vec3_t = [-13.0, 15.0, 4.0];
        range.Wrap(&mut v, &mut spawn);
        assert_eq!(v, [7.0, -5.0, 4.0]);

        // Over the maximum on x and under the minimum on y.
        let mut v: vec3_t = [11.0, -11.0, 0.0];
        range.Wrap(&mut v, &mut spawn);
        assert_eq!(v, [-9.0, 9.0, 0.0]);
    }

    /// The fade machine's four transitions, driven through `Update` in order.
    /// The alpha ceiling is `mColor[3]`, and the render count re-reads the flag rather than the local.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1229-1298`
    #[test]
    fn the_fade_machine_walks_its_four_transitions() {
        let outside = COutside::new();
        let flag_render = 1u32 << CWeatherParticle::FLAG_RENDER;
        let flag_fadein = 1u32 << CWeatherParticle::FLAG_FADEIN;
        let flag_fadeout = 1u32 << CWeatherParticle::FLAG_FADEOUT;

        // The particle sits in front of the camera and inside the range box, so it is in view.
        let mut cloud = one_particle_cloud([10.0, 0.0, 0.0]);
        // No gravity and no wind, so the particle holds its place and the view test does not drift.
        cloud.mGravity = 0.0;
        let mut rng = Rng::default();
        let step = |cloud: &mut CWeatherParticleCloud, rng: &mut Rng| {
            cloud.Update(rng, &outside, [0.0; 3], VIEW_AXIS, false, [0.0; 3], 0.02);
        };
        // `mFade` is 10.0 and the step is 0.02 seconds, so each update moves the alpha by 0.2.
        let fade = cloud.mFade * 0.02;

        // Transition one: not rendering and in view starts a fade in.
        step(&mut cloud, &mut rng);
        assert_eq!(cloud.mParticles[0].mFlags, flag_render | flag_fadein);
        assert_eq!(cloud.mParticles[0].mAlpha, fade);
        assert_eq!(cloud.mParticleCountRender, 1);

        // Transition two: the fade in stops at `mColor[3]`, not at 1.0.
        cloud.mColor[3] = 0.75;
        for _ in 0..8 {
            step(&mut cloud, &mut rng);
        }
        assert_eq!(cloud.mParticles[0].mFlags, flag_render);
        assert_eq!(cloud.mParticles[0].mAlpha, 0.75);

        // Transition three: rendering and out of view starts a fade out.
        // The particle moves behind the camera, which fails the half-space test.
        cloud.mParticles[0].mPosition = [-10.0, 0.0, 0.0];
        step(&mut cloud, &mut rng);
        assert_eq!(cloud.mParticles[0].mFlags, flag_render | flag_fadeout);
        assert_eq!(cloud.mParticles[0].mAlpha, 0.75 - fade);
        assert_eq!(cloud.mParticleCountRender, 1);

        // Transition four: the fade out ends by clearing every flag, and the count re-reads the flag it just cleared.
        for _ in 0..3 {
            step(&mut cloud, &mut rng);
        }
        assert_eq!(cloud.mParticles[0].mFlags, 0);
        assert_eq!(cloud.mParticles[0].mAlpha, 0.0);
        assert_eq!(cloud.mParticleCountRender, 0);
    }
}
