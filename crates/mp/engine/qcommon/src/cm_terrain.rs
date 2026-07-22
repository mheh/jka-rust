//! `CmLandScape` (Raven `CCMLandScape`) — the terrain collision brush + its
//! LIVE construction/registration entry point (Raven `CM_RegisterTerrain`).
//!
//! §F idiomatic reimplementation (porting-rules §17-21); `docs/subsystems/
//! rmg-terrain.md` roster row (class `CCMLandScape`). **RMG-D1** (ruling 25):
//! the whole RMG *generation* path is dead code under the `DEDICATED` build
//! this engine ships, so only the reachable construction/collision surface is
//! ported here; the generation subtree is §20-dropped (Divergences).
//!
//! **Live surface ported in this file:**
//! - The ctor (`cm_terrain.cpp:116-219`): info-string config parse, bounds/
//!   size/patch-size math, heightmap (unpopulated under `DEDICATED`) /
//!   flatten-map (memset-0) allocation, the seeded per-instance LCG
//!   (`holdrand = 0x89abcdef`, `:122`) — **or** `EngineHost::error` when the
//!   config's `heightMap` key is empty (`Com_Error(ERR_FATAL, …)`,
//!   `:190-193`, the 4th live `EngineHost` method).
//! - `LoadTerrainDef` (`:39-110`, unconditional — no `#ifdef DEDICATED`
//!   guard): GP2-parses `ext_data/RMG/<terrainDef>.terrain`, falls back to
//!   `ext_data/arioche/<terrainDef>.terrain`, non-fatal `print` + return on a
//!   double miss (`:48-56`) — reuses the already-ported
//!   `mp_engine_qcommon::gp2::GenericParser2` **intra-crate** (no new pub
//!   seam/edge). Only on a successful parse does it read shader flags
//!   through the SETTLED extern `CollisionWorld::cm_get_shader_info`
//!   (ruling 41/RMG-D5, owned by the `cm` C-track packet, **not** ported
//!   here).
//! - `UpdatePatches`/`CalcRealCoords` (`:898-995`) and the `CmPatch`
//!   collision-patch build over the shared brush arena (RMG-D7/ruling 46:
//!   Raven's single `Z_Malloc(size * GetBlockCount())` buffer
//!   `mPatchBrushData`, `:213-215`, becomes ONE `CmLandScape`-owned
//!   `Vec<u8>` arena; each `CmPatch` — its own file, `cm_patch.rs`, per
//!   ruling 39d/§21 — holds an offset/length range into it, §B5, no raw
//!   pointer).
//! - The per-frame terrain-collision surface (LIVE, ruling 28/RMG-D1):
//!   `PatchCollide`/`WaterCollide` (`:600,836`) + the bounds/water accessors,
//!   reached by the `cm-trace`/`cm-test` C-track packets
//!   (`cm_trace.cpp:283,760,789`, `cm_test.cpp:285-289`). **Per ruling 38**
//!   (the E0502-proven seam repair) these port as `CollisionWorld` methods
//!   (`impl CollisionWorld` below) that resolve `self.land_scape`
//!   internally — *not* `CmLandScape` methods taking `&mut CollisionWorld`
//!   (struck) — and forward to the private `CmLandScape` helpers.
//! - The snapshot/download read (`sv_client.cpp:779-806`): `GetHeightMap`/
//!   `GetFlattenMap`/`GetRealArea`/`get_rand_seed` stay `CmLandScape` `&self`
//!   methods (no `cm` param — the caller resolves the handle via
//!   `RmManager::land()` then reads through the immutable split-borrow).
//! - Three LIVE private-internal (§A1, not pub seam) helpers this doc's
//!   review corrected from "no live caller": `SetShaders` (`:26`, called
//!   from `LoadTerrainDef` at `:83`), `CalcRealCoords` (`:975`, called from
//!   `UpdatePatches` at `:914`), `GetPatch` (`:593`, called from
//!   `PatchCollide` and the `CmPatch` adjacency walk,
//!   `cm_terrain.cpp:256,282,681,823`).
//!
//! **§20-dropped (recorded, not ported — Divergences):** `mRefCount`
//! (renderer-only, DEC-01); the twelve `cm_landscape.h:247-258` area `CM_*`
//! free-fn wrappers and the `CCMLandScape` area/carve methods they forward to
//! (`FlattenArea`/`SaveArea`/`GetWorldHeight`/`AreaCollision`/
//! `GetFirst|NextArea`/`FractionBelowLevel`/`CarveBezierCurve`/
//! `GetFirst|Player|NextObjectiveArea`) — all zero-caller once the
//! generation path (their only callers) is dead; `CArea` (dead-surface,
//! named nowhere live); `CM_TerrainPatchIterate`/`TerrainPatchIterate`
//! (renderer + dead generation-path callers only); `GetTerxelLocalCoords`
//! (`:862`, sole callers commented-out, `:948-950`) and `CarveLine` (`:1133`,
//! sole caller the §20-dropped `CarveBezierCurve`); the `mRandomTerrain`
//! field (RMG-D4e — `CreateRandomTerrain`'s only call site is inside the
//! `#else` of `#ifdef DEDICATED`, `:170-188`, so it is always `0`); `mAreas`/
//! `mAreasIt` (only read/written by the dropped area/carve methods).
//!
//! `CmPatch` (`CCMPatch`) and `CmHeightDetails` (`CCMHeightDetails`) get
//! their own files (`cm_patch.rs`, `cm_height_details.rs`) per ruling 39d
//! (§21 one-Raven-class-per-file) — not repeated here.
//!
//! Class definition source: `oracle/codemp/qcommon/cm_landscape.h:135-243`
//! Method source: `oracle/codemp/qcommon/cm_terrain.cpp`

use core::ffi::c_ulong;
use core::mem::size_of;

use mp_host_interface::EngineHost;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::collision::cplane_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{vec3_t, vec3pair_t};
use native_string::atof::atof;
use native_string::atoi::atoi;
use native_string::Info_ValueForKey;

use crate::cm::cbrush_s::cbrush_t;
use crate::cm::cbrushside_s::cbrushside_t;
use crate::cm::ccmshader::CCMShader;
use crate::cm::cm_local_consts::SURFACE_CLIP_EPSILON;
use crate::cm::trace_work_s::traceWork_s;
// The two per-frame terrain-collision C-track free functions Raven's
// `PatchCollide` calls (`CM_CalcExtents` `cm_trace.cpp:1550`,
// `CM_HandlePatchCollision` `cm_trace.cpp:914`, decls `cm_public.h:56-57`) are
// owned by the `cm`-C-track qcommon packet (they land with the wave-0–4
// clipmap-trace lane, `rmg-terrain.md` Slice hooks). They are NOT ported by
// this doc and are genuinely absent from this tree, so `patch_collide` calls
// them here through their idiomatic ruling-40 renames in the yet-to-land
// `crate::cm::cm_trace` module (see this file's divergence note above / the
// returned `problems` entry — this is a REPORTED missing-sibling binding,
// reconciled when the cm-trace lane lands, not an invented behavior).
use crate::cm::cm_trace::{calc_extents, handle_patch_collision};
use crate::cm_height_details::CmHeightDetails;
use crate::cm_patch::CmPatch;
use crate::collision_world::CollisionWorld;
use crate::gp2::generic_parser2::GenericParser2;
use crate::terrain_handle::TerrainHandle;

/// Raven `BRUSH_SIDES_PER_TERXEL` — the terrain compiles with
/// `_SMOOTH_TERXEL_BRUSH` defined (`cm_terrain.cpp:18`), so this is `8`, not the
/// `#else` `5`. Load-bearing for the shared brush-arena byte size
/// (`cm_terrain.cpp:214-215`, RMG-D7).
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:18-24`
const BRUSH_SIDES_PER_TERXEL: usize = 8;

/// Raven `Round` — `(int)floorf(value + 0.5f)`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:1094-1097`
fn round_f(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

/// Raven `Com_ParseTextFile(file, parser, cleanFirst)` — opens `file`, reads it
/// whole, and drives `parser.Parse`. Ports `FS_FOpenFileByMode`+`FS_Read` as
/// `host.fs_read_file` (`None`/missing or zero length → `false`, mirroring
/// Raven's `if (!f || !length) return false`); the parse result is ignored
/// exactly as Raven ignores `parser.Parse`'s return, and a successful open
/// returns `true`. `cleanFirst` defaults `true` (LoadTerrainDef's 2-arg call).
/// No shared `Com_ParseTextFile` is ported in this crate, so this thin FS
/// wrapper is local (§14 explicit dep).
///
/// Source: `oracle/codemp/qcommon/common.cpp:2179-2202`
fn com_parse_text_file(
    host: &mut impl EngineHost,
    file: &str,
    parser: &mut GenericParser2,
) -> bool {
    match host.fs_read_file(file) {
        Some(data) if !data.is_empty() => {
            let text = String::from_utf8_lossy(&data);
            let _ = parser.parse(&text, true);
            host.fs_free_file(data);
            true
        }
        Some(data) => {
            host.fs_free_file(data);
            false
        }
        None => false,
    }
}

/// Raven `HEIGHT_RESOLUTION` — the size of `mHeightDetails[]`. Already ported
/// for the renderer-side `CTRLandScape`
/// (`crates/mp/renderer/src/tr_landscape/ctrland_scape.rs`), but the renderer
/// is deferred (DEC-01) and out of this crate's dependency graph, so the
/// constant is repeated locally rather than adding a `mp_renderer` edge.
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:13`
const HEIGHT_RESOLUTION: usize = 256;

/// Raven `CCMLandScape`.
///
/// **Frozen field set (RMG-D4h / ruling 46, RMG-D7).** `byte* mHeightMap`/
/// `mFlattenMap` → owned `Vec<u8>` (§B9: manual alloc/free → ownership);
/// `CCMPatch *mPatches` → owned `Vec<CmPatch>`; `holdrand` stays an inline
/// `c_ulong` field (Raven `unsigned long`, seeded `0x89abcdef`,
/// `cm_terrain.cpp:122`; `get_rand_seed` is live-streamed,
/// `sv_client.cpp:806`). The shared per-patch collision-brush buffer
/// `mPatchBrushData` (`cm_landscape.h:151`) becomes ONE `Vec<u8>` arena owned
/// here — Raven's single `Z_Malloc(size * GetBlockCount())` allocation
/// topology preserved; each `CmPatch` (its own file) stores an offset/length
/// range into it rather than a raw `cbrush_s*` slice (§B5, RMG-D7/ruling 46).
///
/// **`mCoords` IS a live field, corrected from an earlier draft's "dead
/// scratch" misreading.** `CalcRealCoords` (`:975-995`) fills it, and while
/// `UpdatePatches` itself never reads it back before freeing it (`:972`),
/// `CCMPatch::CreatePatchPlaneData` (`cm_patch.rs`, `cm_terrain.cpp:302-322,
/// 342-347`) — called from the LIVE `CCMPatch::Init` (`:524-591`, itself
/// called per-patch from `UpdatePatches`, `:925`) — reads it via
/// `owner->GetCoords()` (`:318`) **before** that free, to build every
/// patch's collision planes. So it is a real, `pub(crate)` field (visible to
/// the sibling `cm_patch.rs` module, RMG-D4h's threaded-owner substitute for
/// `owner->…`), not local scratch.
///
/// **Dropped fields** (Divergences, this file's header): `mRefCount`
/// (renderer-only, DEC-01); `mRandomTerrain` (RMG-D4e — always `0`/`NULL`
/// under `DEDICATED`, `GetRandomTerrain()` models as always-`None`, no
/// field/handle exists); `mAreas`/`mAreasIt` (only the dropped area/carve
/// methods touch them).
///
/// **`pub(crate)` fields.** `CCMPatch::Init`/`CreatePatchPlaneData`/
/// `GetAdjacentBrushX`/`GetAdjacentBrushY` (`cm_patch.rs`) read several
/// trivial Raven inline getters (`GetTerxels`/`GetRealWidth`/
/// `GetTerxelSize`/`GetPatchSize`/`GetMins`/`GetCoords`, `cm_landscape.h:
/// 199-239`) through the threaded `owner`/`ls` — these are exposed as
/// `pub(crate)` fields rather than a battery of one-line wrapper methods
/// (the same treatment Seam §C gives the `CollisionWorld` forwarders'
/// "+getters"): `width`, `terxels`, `terxel_size`, `patch_size`, `bounds`,
/// `coords`, `height_details`, `patch_brush_data` (the last for
/// `GetAdjacentBrushX/Y`'s slice into the shared arena, RMG-D7).
///
/// Type definition source: `oracle/codemp/qcommon/cm_landscape.h:135-243`
pub struct CmLandScape {
    /// Raven `mHeightMap` — byte samples, allocated but **unpopulated** under
    /// `DEDICATED` (no image load, no generation).
    /// Source: `cm_landscape.h:140`; alloc `cm_terrain.cpp:157`
    height_map: Vec<u8>,
    /// Raven `mFlattenMap` — memset-0 under `DEDICATED`.
    /// Source: `cm_landscape.h:141`; alloc+memset `cm_terrain.cpp:158,161`
    flatten_map: Vec<u8>,
    /// Raven `mWidth` — heightmap width excluding the 1-pixel edge.
    /// `pub(crate)`: read by `CmPatch::Init`/`CreatePatchPlaneData` via
    /// `owner->GetRealWidth()` (`width + 1`).
    /// Source: `cm_landscape.h:142`
    pub(crate) width: i32,
    /// Raven `mHeight` — heightmap height excluding the 1-pixel edge.
    /// Source: `cm_landscape.h:142`
    height: i32,
    /// Raven `mTerxels` — terxels per patch side. `pub(crate)`: read by every
    /// `CmPatch` live method via `owner->GetTerxels()`.
    /// Source: `cm_landscape.h:143`
    pub(crate) terxels: i32,
    /// Raven `mTerxelSize` — scale from heightmap samples to world coords.
    /// `pub(crate)`: read by `CmPatch::Init` via `owner->GetTerxelSize()`.
    /// Source: `cm_landscape.h:144`
    pub(crate) terxel_size: vec3_t,
    /// Raven `mBounds` — real-world bounds of the terrain brush. `pub(crate)`:
    /// `bounds[0]` (mins) is read by `CmPatch::CreatePatchPlaneData` via
    /// `owner->GetMins()`.
    /// Source: `cm_landscape.h:145`
    pub(crate) bounds: vec3pair_t,
    /// Raven `mSize` — terrain brush size in world coords (excluding 1 patch
    /// edge).
    /// Source: `cm_landscape.h:146`
    // Faithful ctor-set field (`cm_terrain.cpp`); its live reader (Raven
    // `GetSize`) lands with a later terrain-consuming slice, so no in-crate
    // scope reads it back yet.
    #[allow(dead_code)]
    size: vec3_t,
    /// Raven `mPatchSize` — size of each patch in x/y. `pub(crate)`: read by
    /// `CmPatch::Init`/`CreatePatchPlaneData` via `owner->GetPatchSize()`/
    /// `GetPatchWidth()`/`GetPatchHeight()`.
    /// Source: `cm_landscape.h:147`
    pub(crate) patch_size: vec3_t,
    /// Raven `mPatchScalarSize` — horizontal size of the patch.
    /// Source: `cm_landscape.h:148`
    patch_scalar_size: f32,
    /// Raven `mBlockWidth` — heightfield width in blocks.
    /// Source: `cm_landscape.h:149`
    block_width: i32,
    /// Raven `mBlockHeight` — heightfield height in blocks.
    /// Source: `cm_landscape.h:149`
    block_height: i32,
    /// Raven `CCMPatch *mPatches` — one collision patch per block
    /// (`GetBlockCount()` of them), built by [`CmLandScape::update_patches`].
    /// Own file per ruling 39d: `crate::cm_patch::CmPatch`.
    /// Source: `cm_landscape.h:150`
    patches: Vec<CmPatch>,
    /// Raven `mPatchBrushData` — the single shared brush arena (RMG-D7/
    /// ruling 46): Raven's one `Z_Malloc(size * GetBlockCount())` buffer,
    /// range-indexed by each `CmPatch` (no raw pointer, §B5). `pub(crate)`:
    /// `CmPatch::GetAdjacentBrushX/Y` (`cm_patch.rs`) slice directly into it
    /// at the adjacent patch's `brush_offset`/`brush_len`.
    /// Source: `cm_landscape.h:151`; alloc `cm_terrain.cpp:213-215`
    pub(crate) patch_brush_data: Vec<u8>,
    /// Raven `mHasPhysics` — set unless disabled by the config string.
    /// Source: `cm_landscape.h:152`
    // Faithful ctor-set field (`cm_terrain.cpp`); its live reader (Raven
    // `HasPhysics`) lands with a later terrain-consuming slice, so no in-crate
    // scope reads it back yet.
    #[allow(dead_code)]
    has_physics: bool,
    /// Raven `mBaseWaterHeight` — base water height in terxels.
    /// Source: `cm_landscape.h:155`
    base_water_height: i32,
    /// Raven `mWaterHeight` — real-world water height.
    /// Source: `cm_landscape.h:156`
    water_height: f32,
    /// Raven `mWaterContents` — contents flags of the water shader.
    /// Source: `cm_landscape.h:157`
    water_contents: i32,
    /// Raven `mWaterSurfaceFlags` — surface flags of the water shader.
    /// Source: `cm_landscape.h:158`
    water_surface_flags: i32,
    /// Raven `holdrand` — the per-instance LCG seed, seeded `0x89abcdef` in
    /// the ctor and streamed by [`CmLandScape::get_rand_seed`]. Retail-win32
    /// 32-bit width (2026-07-17 ruling; Raven `unsigned long` = 32-bit on the
    /// ship target), not a `Rng`/`QRand` type.
    /// Source: `cm_landscape.h:160`; seeded `cm_terrain.cpp:122`
    holdrand: u32,
    /// Raven `mHeightDetails[HEIGHT_RESOLUTION]` — surface/contents flags per
    /// height band, zeroed by the ctor's `memset` (`cm_terrain.cpp:125`;
    /// `CmHeightDetails` derives `Default` for the equivalent zero-init).
    /// `pub(crate)`: `CmPatch::Init` (`cm_patch.rs`) indexes it via
    /// `owner->GetSurfaceFlags(height)`/`GetContentFlags(height)`
    /// (`cm_landscape.h:225-226`).
    /// Source: `cm_landscape.h:165`
    pub(crate) height_details: [CmHeightDetails; HEIGHT_RESOLUTION],
    /// Raven `mCoords` — scratch real-world coordinate per heightmap sample,
    /// filled by [`CmLandScape::calc_real_coords`] and read live by
    /// `CmPatch::CreatePatchPlaneData` (`cm_patch.rs`) via `owner->GetCoords()`
    /// before `UpdatePatches` frees Raven's copy (`:972` — the Rust `Vec`
    /// instead simply outlives its one construction-time use, §C9). LIVE, not
    /// dead scratch (see the struct doc's correction note). `pub(crate)`.
    /// Source: `cm_landscape.h:166`
    pub(crate) coords: Vec<vec3_t>,
}

impl CmLandScape {
    /// `CCMLandScape::CCMLandScape` — the LIVE construction under
    /// `DEDICATED`: info-string config parse (`heightMap`/`numPatches`/
    /// `terxels`/`physics`/`seed`/`minx..maxz`), bounds/size/block/patch-size
    /// math, the seeded LCG (`holdrand = 0x89abcdef`), heightmap/flatten-map
    /// allocation (heightmap left **unpopulated** — no image load under
    /// `DEDICATED`, `:170-188`), `LoadTerrainDef`, and the patch/brush-arena
    /// build (`UpdatePatches`). **`else` branch**: an empty `heightMap` key
    /// diverges via `host.error(errorParm_t::ERR_FATAL, "Terrain has no
    /// heightmap specified\n")` (`:190-193`) — the 4th live `EngineHost`
    /// method (Seam definition). `server` is accepted for signature fidelity
    /// but unread in the ctor body, exactly as Raven's own parameter is.
    /// `cm` threads `LoadTerrainDef`'s shader lookups
    /// (`CollisionWorld::cm_get_shader_info`, ruling 41/RMG-D5).
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:116-219`
    fn new(
        cm: &mut CollisionWorld,
        host: &mut impl EngineHost,
        configstring: &str,
        server: bool,
    ) -> Self {
        // `server` is accepted for signature fidelity but unread in the ctor
        // body, exactly as Raven's own parameter is.
        let _ = server;

        // Extract the relevant data from the config string.
        let height_map_key = Info_ValueForKey(configstring, "heightMap");
        // Raven `atol`; identical to `atoi` on the 32-bit retail target.
        let num_patches = atoi(&Info_ValueForKey(configstring, "numPatches"));
        let terxels = atoi(&Info_ValueForKey(configstring, "terxels"));
        let has_physics = atoi(&Info_ValueForKey(configstring, "physics")) != 0;
        // Raven parses `seed` with `strtoul` into a local that is never read
        // (`cm_terrain.cpp:137`, dead) — dropped (§C10), no observable effect.

        let bounds: vec3pair_t = [
            [
                atof(&Info_ValueForKey(configstring, "minx")) as f32,
                atof(&Info_ValueForKey(configstring, "miny")) as f32,
                atof(&Info_ValueForKey(configstring, "minz")) as f32,
            ],
            [
                atof(&Info_ValueForKey(configstring, "maxx")) as f32,
                atof(&Info_ValueForKey(configstring, "maxy")) as f32,
                atof(&Info_ValueForKey(configstring, "maxz")) as f32,
            ],
        ];

        // Calculate size of the brush (VectorSubtract, `cm_terrain.cpp:147`).
        let size: vec3_t = [
            bounds[1][0] - bounds[0][0],
            bounds[1][1] - bounds[0][1],
            bounds[1][2] - bounds[0][2],
        ];

        // Work out the dimensions of the brush in blocks — make them as square
        // as possible (`cm_terrain.cpp:150-151`).
        let block_width = round_f((num_patches as f32 * size[0] / size[1]).sqrt());
        let block_height = round_f((num_patches as f32 * size[1] / size[0]).sqrt());

        // ...which lets us get the size of the heightmap (`:154-155`).
        let width = block_width * terxels;
        let height = block_height * terxels;

        let real_area = ((width + 1) * (height + 1)) as usize;
        // `mHeightMap` (`:157`) is allocated but left UNPOPULATED under
        // DEDICATED (no image load, no generation) — Raven's non-zeroing
        // `Z_Malloc` leaves it as UB heap; §F.19 zeroes it here (defined
        // behavior, excluded from the byte-compare goldens). `mFlattenMap` is
        // the memset-0 map (`:158,161`).
        let height_map = vec![0u8; real_area];
        let flatten_map = vec![0u8; real_area];

        // The heightmap-image / random-terrain population (`:163-188`) is dead
        // under DEDICATED (`imageData` forced NULL, `mRandomTerrain` stays 0,
        // RMG-D1) — an empty `heightMap` key is the live `else` branch that
        // fatals (`:190-193`, the 4th live `EngineHost` method).
        if height_map_key.is_empty() {
            host.error(
                errorParm_t::ERR_FATAL,
                "Terrain has no heightmap specified\n",
            );
        }

        // Work out the dimensions of the terxel — almost square (`:196-198`).
        let terxel_size: vec3_t = [
            size[0] / width as f32,
            size[1] / height as f32,
            size[2] / 255.0,
        ];

        // Work out the patchsize (`:201-204`).
        let patch_size: vec3_t = [
            size[0] / block_width as f32,
            size[1] / block_height as f32,
            1.0,
        ];
        // mPatchScalarSize = VectorLength(mPatchSize).
        let patch_scalar_size = (patch_size[0] * patch_size[0]
            + patch_size[1] * patch_size[1]
            + patch_size[2] * patch_size[2])
            .sqrt();

        let block_count = (block_width * block_height) as usize;
        // The single shared brush arena (RMG-D7): Raven's one
        // `Z_Malloc(size * GetBlockCount())` buffer (`:213-215`).
        let num_brushes_per_patch = (terxels * terxels * 2) as usize;
        let brush_size = (num_brushes_per_patch * size_of::<cbrush_t>())
            + (num_brushes_per_patch
                * BRUSH_SIDES_PER_TERXEL
                * 2
                * (size_of::<cbrushside_t>() + size_of::<cplane_t>()));

        let mut ls = CmLandScape {
            height_map,
            flatten_map,
            width,
            height,
            terxels,
            terxel_size,
            bounds,
            size,
            patch_size,
            patch_scalar_size,
            block_width,
            block_height,
            // One collision patch per block, built by `update_patches`.
            patches: (0..block_count).map(|_| CmPatch::default()).collect(),
            patch_brush_data: vec![0u8; brush_size * block_count],
            has_physics,
            base_water_height: 0,
            water_height: 0.0,
            water_contents: 0,
            water_surface_flags: 0,
            holdrand: 0x89abcdef,
            height_details: [CmHeightDetails::default(); HEIGHT_RESOLUTION],
            coords: Vec::new(),
        };

        // Loads in the water height and properties; gets the shader properties
        // for the blended shaders (`:208`).
        ls.load_terrain_def(cm, host, configstring);

        // Initialize all terrain patches (`:218`).
        ls.update_patches();

        ls
    }

    /// `CCMLandScape::LoadTerrainDef` — unconditional (no `#ifdef DEDICATED`
    /// guard). GP2-parses `ext_data/RMG/<terrainDef>.terrain` via a
    /// function-local `mp_engine_qcommon::gp2::GenericParser2`
    /// (`Com_ParseTextFile` → `host.fs_read_file` + `GenericParser2::parse`,
    /// INTRA-CRATE, no new pub seam/edge), falling back to
    /// `ext_data/arioche/<terrainDef>.terrain` on a first miss and printing
    /// `Could not open %s` + returning non-fatally (`host.print`, §C10
    /// control flow) on a double miss — all before any shader read. Only on
    /// a successful parse does it walk `altitudetexture`/`water` groups,
    /// reading shader flags through `cm.cm_get_shader_info` (the SETTLED
    /// extern `cm`-C-track binding, ruling 41/RMG-D5 — NOT ported by this
    /// doc) and calling [`CmLandScape::set_shaders`] / populating
    /// `water_contents`/`water_surface_flags`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:39-110`
    fn load_terrain_def(&mut self, cm: &mut CollisionWorld, host: &mut impl EngineHost, td: &str) {
        let terrain_def = Info_ValueForKey(td, "terrainDef");
        // Com_DPrintf trace print (`:46`) is developer-only, no golden effect —
        // dropped.

        let mut parse = GenericParser2::new();
        let mut path = format!("ext_data/RMG/{terrain_def}.terrain");
        if !com_parse_text_file(host, &path, &mut parse) {
            path = format!("ext_data/arioche/{terrain_def}.terrain");
            if !com_parse_text_file(host, &path, &mut parse) {
                host.print(&format!("Could not open {path}\n"));
                return;
            }
        }

        // The whole file → the root `{ }` struct → its subgroups (`:58-64`).
        let basegroup = parse.top_level();
        for classes in basegroup.subgroups() {
            for items in classes.subgroups() {
                if items.name().eq_ignore_ascii_case("altitudetexture") {
                    // Height must exist — the rest are optional (`:74`).
                    let height = atoi(items.find_pair_value("height").unwrap_or("0"));

                    // Shader for this height (`:77-85`).
                    let shader_name = items.find_pair_value("shader").unwrap_or("");
                    if !shader_name.is_empty() {
                        if let Some(shader) = cm.cm_get_shader_info(shader_name) {
                            self.set_shaders(height, shader);
                        }
                    }
                } else if items.name().eq_ignore_ascii_case("water") {
                    // Grab the height of the water (`:93-94`).
                    self.base_water_height = atoi(items.find_pair_value("height").unwrap_or("0"));
                    // SetRealWaterHeight (`:94`, `cm_landscape.h:231`):
                    // mWaterHeight = height * mTerxelSize[2].
                    self.water_height = self.base_water_height as f32 * self.terxel_size[2];

                    // Grab the material of the water (`:97-103`).
                    let shader_name = items.find_pair_value("shader").unwrap_or("");
                    if let Some(shader) = cm.cm_get_shader_info(shader_name) {
                        self.water_contents = shader.contentFlags;
                        self.water_surface_flags = shader.surfaceFlags;
                    }
                }
            }
        }
        // Com_ParseTextFileDestroy (`:109`) — the arena drops with `parse`.
    }

    /// `CCMLandScape::SetShaders` — LIVE private-internal (§A1) helper called
    /// from [`CmLandScape::load_terrain_def`]'s `altitudetexture` case
    /// (`cm_terrain.cpp:83`) whenever that case's shader lookup hits. Fills
    /// `height_details[height..HEIGHT_RESOLUTION]` with the shader's
    /// content/surface flags, stopping at the first band that already has
    /// surface flags set. The call site already null-checks the shader
    /// (`if(shader) { SetShaders(…); }`, `:81-84`), so this takes `&CCMShader`
    /// rather than Raven's nullable `CCMShader*`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:26-37`
    fn set_shaders(&mut self, height: i32, shader: &CCMShader) {
        let mut i = height;
        while i < HEIGHT_RESOLUTION as i32 {
            let idx = i as usize;
            if self.height_details[idx].get_surface_flags() == 0 {
                self.height_details[idx].set_flags(shader.contentFlags, shader.surfaceFlags);
            }
            i += 1;
        }
    }

    /// `CCMLandScape::UpdatePatches` — LIVE, called once from the ctor
    /// (`:218`) after `LoadTerrainDef`. Calls
    /// [`CmLandScape::calc_real_coords`] (fills `self.coords`), then builds
    /// each `CmPatch` (its own file, `cm_patch.rs`) by slicing this
    /// landscape's shared `patch_brush_data` arena per block (RMG-D7/
    /// ruling 46: Raven's `mPatchBrushData + (size * (ix + iy *
    /// mBlockWidth))` pointer offset, `cm_terrain.cpp:925`, becomes range
    /// arithmetic over the owned `Vec`) — each `CmPatch::Init` call reads
    /// `self.coords` live via `CreatePatchPlaneData` before this method
    /// returns.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:898-973`
    fn update_patches(&mut self) {
        // Calculate real world coordinates from the heightmap (`:914`).
        self.calc_real_coords();

        let num_brushes_per_patch = (self.terxels * self.terxels * 2) as usize;
        let size = (num_brushes_per_patch * size_of::<cbrush_t>())
            + (num_brushes_per_patch
                * BRUSH_SIDES_PER_TERXEL
                * 2
                * (size_of::<cbrushside_t>() + size_of::<cplane_t>()));

        // `CmPatch::init` takes the whole height map by `&[u8]` AND the owning
        // `&mut CmLandScape` (frozen sibling signature) — both would borrow
        // `self`. Clone the height map once (read-only in `init`) so the shared
        // borrow does not alias the `&mut self`; the patch is `mem::take`n out
        // of `self.patches` and restored, so `init` can take `&mut self` while
        // mutating its own patch (the shared-arena slices/adjacency reads on
        // `ls` never touch the patch's own slot).
        let height_map = self.height_map.clone();

        let mut patch_index = 0usize;
        let mut iy = 0;
        let mut y = 0;
        while y < self.height {
            let mut ix = 0;
            let mut x = 0;
            while x < self.width {
                let world: vec3_t = [
                    self.bounds[0][0] + (x as f32 * self.terxel_size[0]),
                    self.bounds[0][1] + (y as f32 * self.terxel_size[1]),
                    self.bounds[0][2],
                ];
                // Raven: mPatchBrushData + (size * (ix + iy * mBlockWidth)),
                // range arithmetic over the shared arena (RMG-D7, `:925`).
                let brush_offset = size * (ix + iy * self.block_width) as usize;

                let mut patch = core::mem::take(&mut self.patches[patch_index]);
                patch.init(self, x, y, world, &height_map, brush_offset);
                self.patches[patch_index] = patch;

                patch_index += 1;
                ix += 1;
                x += self.terxels;
            }
            iy += 1;
            y += self.terxels;
        }

        // The dead `#if 0`-style smoothing block (`:929-969`) and its
        // `GetTerxelLocalCoords` callers are §20 (commented out in Raven) — not
        // ported.

        // Cleanup coord array (Z_Free(mCoords), `:972`) — the owned `Vec`
        // simply outlives its one construction-time use (§C9).
        self.coords = Vec::new();
    }

    /// `CCMLandScape::CalcRealCoords` — LIVE private-internal (§A1) helper
    /// called from [`CmLandScape::update_patches`] (`:914`). Fills
    /// `self.coords` with the real-world coordinate of every heightmap
    /// sample. **Corrects an earlier draft's "dead scratch" misreading**: the
    /// output IS read live, by `CmPatch::CreatePatchPlaneData`
    /// (`cm_patch.rs`) via `owner->GetCoords()` (`cm_terrain.cpp:318`) during
    /// the per-patch build `UpdatePatches` drives — before Raven's copy is
    /// freed, unread-again, at `:972` (the struct doc's `mCoords` note).
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:975-995`
    fn calc_real_coords(&mut self) {
        let real_width = self.width + 1; // GetRealWidth
        let real_height = self.height + 1; // GetRealHeight
        self.coords = vec![[0.0f32; 3]; (real_width * real_height) as usize];

        let mins = self.bounds[0]; // GetMins
        let ts = self.terxel_size; // GetTerxelSize

        // Work out the real world coordinates of each heightmap entry.
        for y in 0..real_height {
            for x in 0..real_width {
                let offset = ((y * real_width) + x) as usize;
                // VectorSet(icoords, x, y, mHeightMap[offset]).
                let icoords: [f32; 3] = [x as f32, y as f32, self.height_map[offset] as f32];
                // VectorScaleVectorAdd(GetMins(), icoords, GetTerxelSize(),
                // mCoords[offset]): out = mins + icoords .* terxelSize.
                self.coords[offset] = [
                    mins[0] + icoords[0] * ts[0],
                    mins[1] + icoords[1] * ts[1],
                    mins[2] + icoords[2] * ts[2],
                ];
            }
        }
    }

    /// `CCMLandScape::GetPatch` — LIVE private-internal (§A1) helper: indexes
    /// `mPatches + ((y * mBlockWidth) + x)`. Called from
    /// [`CmLandScape::patch_collide`] (`:681,768,823`) and from `CmPatch`'s
    /// `GetAdjacentBrushX`/`GetAdjacentBrushY` (`cm_patch.rs`,
    /// `cm_terrain.cpp:256,282`) via the owning `CmLandScape` threaded in
    /// place of Raven's dropped `owner` back-pointer (§B3/RMG-D4h). `pub(crate)`
    /// (not module-private) because that adjacency walk lives in the sibling
    /// file `cm_patch.rs`. Non-`const` in Raven (returns a mutable pointer),
    /// so `&mut self` / `&mut CmPatch` here.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:593-596`
    pub(crate) fn get_patch(&mut self, x: i32, y: i32) -> &mut CmPatch {
        // mPatches + ((y * mBlockWidth) + x).
        let index = ((y * self.block_width) + x) as usize;
        &mut self.patches[index]
    }

    /// `CCMLandScape::PatchCollide` — LIVE (ruling 28/RMG-D1): the per-frame
    /// terrain patch-collision sweep, reached only through
    /// [`CollisionWorld::terrain_patch_collide`] below (never a pub seam
    /// method itself — ruling 38's receiver-shape repair). The `checkcount`
    /// writes into this landscape's owned brush data are legal `&mut self`
    /// mutation. `tw`/`trace_t` are C-track types (`crate::cm`/
    /// `mp_qshared::common::mp::trace_t`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:600-834`
    fn patch_collide(
        &mut self,
        tw: &mut traceWork_s,
        trace: &mut trace_t,
        start: vec3_t,
        end: vec3_t,
        checkcount: i32,
    ) {
        // Convert to valid bounding box.
        let mut t_bounds: vec3pair_t = [[0.0; 3]; 2];
        calc_extents(start, end, tw, &mut t_bounds);

        // Raven guards on `if (1)` (the `com_newtrace->integer` read is
        // commented out, `:607-608`); the slope-walk branch is always taken and
        // the box-collide `else` (`:799-833`) is dead — dropped per §C10.
        let mut patch_direction: f32 = 1.0;
        let mut check_direction: f32 = 1.0;
        let fraction = trace.fraction;

        if (end[0] - start[0]).abs() >= (end[1] - start[1]).abs() {
            // x travels more than y — calculate line slope and offset.
            let slope = if end[0] - start[0] != 0.0 {
                (end[1] - start[1]) / (end[0] - start[0])
            } else {
                0.0
            };
            let offset = start[1] - (start[0] * slope);

            // Find the starting patch.
            let mut start_patch_loc = ((start[0] - self.bounds[0][0]) / self.patch_size[0]).floor();
            let mut end_patch_loc = ((end[0] - self.bounds[0][0]) / self.patch_size[0]).floor();

            let mut count_patches;
            if start_patch_loc <= end_patch_loc {
                // moving along slope in a positive direction
                end_patch_loc += 1.0;
                start_patch_loc -= 1.0;
                count_patches = (end_patch_loc - start_patch_loc + 1.0) as i32;
            } else {
                // moving along slope in a negative direction
                end_patch_loc -= 1.0;
                start_patch_loc += 1.0;
                patch_direction = -1.0;
                count_patches = (start_patch_loc - end_patch_loc + 1.0) as i32;
            }
            if slope < 0.0 {
                check_direction = -1.0;
            }

            // Real world location, then back into patch coords.
            let mut start_pos =
                ((start_patch_loc * self.patch_size[0] + self.bounds[0][0]) * slope) + offset;
            start_pos =
                ((start_pos - self.bounds[0][1] + tw.size[0][1]) / self.patch_size[1]).floor();
            loop {
                if start_patch_loc >= 0.0 && start_patch_loc < self.block_width as f32 {
                    // valid location
                    let mut end_pos = (((start_patch_loc + patch_direction) * self.patch_size[0]
                        + self.bounds[0][0])
                        * slope)
                        + offset;
                    end_pos = ((end_pos - self.bounds[0][1] + tw.size[1][1]) / self.patch_size[1])
                        .floor();

                    if check_direction < 0.0 {
                        start_pos += 1.0;
                        end_pos -= 1.0;
                    } else {
                        start_pos -= 1.0;
                        end_pos += 1.0;
                    }
                    let mut count = ((end_pos - start_pos).abs() + 1.0) as i32;
                    while count != 0 {
                        if start_pos >= 0.0 && start_pos < self.block_height as f32 {
                            // valid location
                            {
                                let patch =
                                    self.get_patch(start_patch_loc as i32, start_pos as i32);
                                // Collide with every patch to find the minimum
                                // fraction.
                                handle_patch_collision(
                                    tw,
                                    trace,
                                    t_bounds[0],
                                    t_bounds[1],
                                    patch,
                                    checkcount,
                                );
                            }
                            if trace.fraction <= 0.0 {
                                return;
                            }
                        }
                        start_pos += check_direction;
                        count -= 1;
                    }

                    if trace.fraction < fraction {
                        return;
                    }
                }
                // Move to the next spot (staying one behind, to get the opposite
                // edge of the terrain patch).
                start_pos =
                    ((start_patch_loc * self.patch_size[0] + self.bounds[0][0]) * slope) + offset;
                start_patch_loc += patch_direction;
                start_pos =
                    ((start_pos - self.bounds[0][1] + tw.size[0][1]) / self.patch_size[1]).floor();
                count_patches -= 1;
                if count_patches == 0 {
                    break;
                }
            }
        } else {
            // y travels more than x — no zero-guard on the denominator (the
            // branch condition guarantees `end[1] - start[1] != 0`).
            let slope = (end[0] - start[0]) / (end[1] - start[1]);
            let offset = start[0] - (start[1] * slope);

            let mut start_patch_loc = ((start[1] - self.bounds[0][1]) / self.patch_size[1]).floor();
            let mut end_patch_loc = ((end[1] - self.bounds[0][1]) / self.patch_size[1]).floor();

            let mut count_patches;
            if start_patch_loc <= end_patch_loc {
                end_patch_loc += 1.0;
                start_patch_loc -= 1.0;
                count_patches = (end_patch_loc - start_patch_loc + 1.0) as i32;
            } else {
                end_patch_loc -= 1.0;
                start_patch_loc += 1.0;
                patch_direction = -1.0;
                count_patches = (start_patch_loc - end_patch_loc + 1.0) as i32;
            }
            if slope < 0.0 {
                check_direction = -1.0;
            }

            let mut start_pos =
                ((start_patch_loc * self.patch_size[1] + self.bounds[0][1]) * slope) + offset;
            start_pos =
                ((start_pos - self.bounds[0][0] + tw.size[0][0]) / self.patch_size[0]).floor();
            loop {
                if start_patch_loc >= 0.0 && start_patch_loc < self.block_height as f32 {
                    let mut end_pos = (((start_patch_loc + patch_direction) * self.patch_size[1]
                        + self.bounds[0][1])
                        * slope)
                        + offset;
                    end_pos = ((end_pos - self.bounds[0][0] + tw.size[1][0]) / self.patch_size[0])
                        .floor();

                    if check_direction < 0.0 {
                        start_pos += 1.0;
                        end_pos -= 1.0;
                    } else {
                        start_pos -= 1.0;
                        end_pos += 1.0;
                    }

                    let mut count = ((end_pos - start_pos).abs() + 1.0) as i32;
                    while count != 0 {
                        if start_pos >= 0.0 && start_pos < self.block_width as f32 {
                            {
                                let patch =
                                    self.get_patch(start_pos as i32, start_patch_loc as i32);
                                handle_patch_collision(
                                    tw,
                                    trace,
                                    t_bounds[0],
                                    t_bounds[1],
                                    patch,
                                    checkcount,
                                );
                            }
                            if trace.fraction <= 0.0 {
                                return;
                            }
                        }
                        start_pos += check_direction;
                        count -= 1;
                    }

                    if trace.fraction < fraction {
                        return;
                    }
                }
                start_pos =
                    ((start_patch_loc * self.patch_size[1] + self.bounds[0][1]) * slope) + offset;
                start_patch_loc += patch_direction;
                start_pos =
                    ((start_pos - self.bounds[0][0] + tw.size[0][0]) / self.patch_size[0]).floor();
                count_patches -= 1;
                if count_patches == 0 {
                    break;
                }
            }
        }
    }

    /// `CCMLandScape::WaterCollide` — LIVE (ruling 28/RMG-D1), `const` in
    /// Raven → `&self`. Reached only through
    /// [`CollisionWorld::terrain_water_collide`] below.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrain.cpp:836-860`
    fn water_collide(&self, begin: vec3_t, end: vec3_t, mut fraction: f32) -> f32 {
        // Completely above water.
        if (begin[2] > self.water_height) && (end[2] > self.water_height) {
            return fraction;
        }
        // Completely below water.
        if (begin[2] < self.water_height) && (end[2] < self.water_height) {
            return fraction;
        }
        // Starting in water and leaving.
        if begin[2] < self.water_height - SURFACE_CLIP_EPSILON {
            fraction =
                ((self.water_height - SURFACE_CLIP_EPSILON) - begin[2]) / (end[2] - begin[2]);
            return fraction;
        }
        // Now the trace must be entering the water.
        if begin[2] > self.water_height + SURFACE_CLIP_EPSILON {
            fraction =
                (begin[2] - (self.water_height + SURFACE_CLIP_EPSILON)) / (begin[2] - end[2]);
        }
        fraction
    }

    // --- Snapshot/download read (Seam §C item 1, sv_client.cpp:779-806).
    //     These stay `&self` methods — the caller resolves the handle via
    //     `RmManager::land()` then reads through the immutable split-borrow
    //     `if let Some(land) = &cm.land_scape { land.height_map() }`. ---

    /// `CCMLandScape::GetHeightMap` — `byte*` → `&[u8]` (ruling 28). §F.19-UB:
    /// unpopulated under `DEDICATED` (no image load) — excluded from
    /// goldens.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:218`
    pub fn height_map(&self) -> &[u8] {
        &self.height_map
    }

    /// `CCMLandScape::GetFlattenMap` — `byte*` → `&[u8]` (ruling 28).
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:219`
    pub fn flatten_map(&self) -> &[u8] {
        &self.flatten_map
    }

    /// `CCMLandScape::GetRealArea` — `(mWidth + 1) * (mHeight + 1)`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:211`
    pub fn real_area(&self) -> i32 {
        (self.width + 1) * (self.height + 1)
    }

    /// `CCMLandScape::get_rand_seed` — Raven `unsigned long`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:239`
    pub fn get_rand_seed(&self) -> c_ulong {
        self.holdrand as c_ulong
    }
}

// --- Per-frame terrain collision (Seam §C item 2). RULING 38: these are
//     methods ON `CollisionWorld` that resolve `self.land_scape` internally
//     (no double borrow) — NOT `CmLandScape` methods taking
//     `&mut CollisionWorld` (that shape is struck, E0502). Signatures are
//     Raven-faithful; the caller preserves Raven's `cmg.landScape != NULL`
//     gate as a `self.land_scape.is_some()` check before calling in, so
//     `land_scape` is `Some` at entry (§19: the `None` branch mirrors Raven's
//     NULL-deref-avoidance, unreachable by caller contract). Reached by the
//     `cm-trace`/`cm-test` C-track packets (`cm_trace.cpp:283,760,789,997,
//     1374`, `cm_test.cpp:285-289`); owned by *this* subsystem (RMG-D4a: the
//     `cm` C-track packets exclude `CCMLandScape`). ---
impl CollisionWorld {
    /// Raven `cmg.landScape->PatchCollide(tw, trace, start, end, checkcount)`
    /// — decl `cm_landscape.h:175`, def `cm_terrain.cpp:600`. `&mut self`:
    /// the checkcount writes into landscape-owned brush data are legal
    /// mutation (ruling 38); `trace_t &trace` out-param → `&mut trace_t`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:175`
    pub fn terrain_patch_collide(
        &mut self,
        tw: &mut traceWork_s,
        trace: &mut trace_t,
        start: vec3_t,
        end: vec3_t,
        checkcount: i32,
    ) {
        // Raven's caller gates on `cmg.landScape != NULL`; the None branch
        // mirrors that NULL-deref-avoidance (unreachable by caller contract,
        // §19).
        let land = self
            .land_scape
            .as_mut()
            .expect("terrain_patch_collide: land_scape must be Some (caller gates on cmg.landScape != NULL)");
        land.patch_collide(tw, trace, start, end, checkcount);
    }

    /// Raven `cmg.landScape->WaterCollide(begin, end, fraction)` —
    /// `cm_landscape.h:178` / `cm_terrain.cpp:836`. `const` in Raven → `&self`
    /// read.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:178`
    pub fn terrain_water_collide(&self, begin: vec3_t, end: vec3_t, fraction: f32) -> f32 {
        match self.land_scape.as_ref() {
            Some(land) => land.water_collide(begin, end, fraction),
            // Caller gates on `cmg.landScape != NULL`; the None branch returns
            // the fraction unchanged (no water interaction), §19.
            None => fraction,
        }
    }

    /// `CCMLandScape::GetBounds` — `cm_landscape.h:199` (`const` → `&self`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:199`
    pub fn terrain_bounds(&self) -> &vec3pair_t {
        &self
            .land_scape
            .as_ref()
            .expect(
                "terrain_bounds: land_scape must be Some (caller gates on cmg.landScape != NULL)",
            )
            .bounds
    }

    /// `CCMLandScape::GetPatchScalarSize` — `cm_landscape.h:207`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:207`
    pub fn terrain_patch_scalar_size(&self) -> f32 {
        self.land_scape
            .as_ref()
            .expect("terrain_patch_scalar_size: land_scape must be Some")
            .patch_scalar_size
    }

    /// `CCMLandScape::GetWaterHeight` — `cm_landscape.h:232`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:232`
    pub fn terrain_water_height(&self) -> f32 {
        self.land_scape
            .as_ref()
            .expect("terrain_water_height: land_scape must be Some")
            .water_height
    }

    /// `CCMLandScape::GetWaterContents` — `cm_landscape.h:233`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:233`
    pub fn terrain_water_contents(&self) -> i32 {
        self.land_scape
            .as_ref()
            .expect("terrain_water_contents: land_scape must be Some")
            .water_contents
    }

    /// `CCMLandScape::GetWaterSurfaceFlags` — `cm_landscape.h:234`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:234`
    pub fn terrain_water_surface_flags(&self) -> i32 {
        self.land_scape
            .as_ref()
            .expect("terrain_water_surface_flags: land_scape must be Some")
            .water_surface_flags
    }
}

/// `CM_RegisterTerrain` — constructs (or, on repeat registration,
/// get-or-creates) the `CmLandScape` under `DEDICATED`. Folds Raven's
/// `CM_InitTerrain` (`cm_terrain.cpp:1618-1626`: `new CCMLandScape(…)` +
/// `SetTerrainId(terrainId)`, always called with `terrainId = 0`,
/// `cm_load.cpp:1048`) directly into construction — no separate
/// `CmLandScape::set_terrain_id`/`get_terrain_id` methods exist; the
/// returned `TerrainHandle` *is* the id. The random-terrain arm
/// (`cm_terrain.cpp:178`) is never taken (RMG-D1). Repeat registration
/// (`cmg.landScape` already `Some`) returns the existing handle rather than
/// `IncreaseRefCount()`ing (`mRefCount` is §20-dropped, DEC-01/RMG-D4c) —
/// `cm_load.cpp:1040-1044`.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:1036-1057`
pub fn register_terrain(
    cm: &mut CollisionWorld,
    host: &mut impl EngineHost,
    config: &str,
    server: bool,
) -> TerrainHandle {
    if cm.land_scape.is_some() {
        // Already spawned — just return the existing terrain id
        // (`cm_load.cpp:1040-1044`). Raven `IncreaseRefCount()`s here, but
        // `mRefCount` is §20-dropped (renderer-only, DEC-01/RMG-D4c); the
        // observable seam behavior is the returned handle. `GetTerrainId()` is
        // always `0` (`SetTerrainId(0)`, `CM_InitTerrain`, `cm_load.cpp:1048`).
        return TerrainHandle(0);
    }

    // Doesn't exist so create and link in — `CM_InitTerrain(config, 0, server)`
    // folds `new CCMLandScape(...)` + `SetTerrainId(0)` (`cm_terrain.cpp:1618-1626`).
    let ls = CmLandScape::new(cm, host, config, server);
    cm.land_scape = Some(ls);
    // The returned `TerrainHandle` *is* the id, always `0`.
    TerrainHandle(0)
}
