//! Raven `tr_WorldEffects.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_WorldEffects.cpp`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cm_test::CM_PointContents;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::{com_error, com_printf};
use mp_engine_qcommon::common_fns::Com_Milliseconds;
use mp_qshared::shared::vec3_t;
use mp_qshared::shared::{
    errorParm_t, CONTENTS_INSIDE, CONTENTS_OUTSIDE, CONTENTS_SOLID, CONTENTS_WATER,
};
use native_math::rng::{Rng, RAND_MAX};

use crate::render_state::image_asset::ImageHandle;
use crate::tr_worldeffects::sparticle::SParticle;

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
    /// The C runtime's `holdrand` (`srand`/`rand`, seeded by
    /// `R_InitWorldEffects`) plus `q_math.c`'s `holdrand` behind `Q_irand` —
    /// both TU-invisible globals in Raven, owned here as one field on the
    /// subsystem carrier (DEC-37 A13.3) and threaded into `WE_flrand`/
    /// `FloatRand`/`SIntRange::Pick`.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1491`
    pub rng: Rng,
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
    // PORT-NOTE: Raven's `mParticles` is a `delete[]`-owned heap array; owned
    // `Vec<SParticle>` (`sparticle.rs`) replaces the manual alloc/free
    // (porting-rules §C9).
    pub mParticles: Vec<SParticle>,
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
        };
        this.Reset();
        this
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
}

/// Raven `R_IsPuffing`.
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:2014-2017`
pub fn R_IsPuffing() -> bool {
    // Raven: "Eh? Don't want surfacesprites to know this?"
    false
}
