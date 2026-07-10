#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    clippy::too_many_arguments
)]

//! `cm_terrain.cpp` — the qcommon terrain twins (`CCMLandScape`/`CArea`/
//! `CRandomTerrain` free-function seam): flatten/smooth/average height-map
//! callbacks, the area-query C wrappers, and the RMG init/collision surface.
//!
//! Source: `oracle/codemp/qcommon/cm_terrain.cpp`
//!
//! PORT-NOTE(rmg-terrain): `CCMLandScape`/`CArea`/`CRandomTerrain` are the
//! rmg-terrain.md §F design's classes (porting-rules §F, ruling 16 — these
//! qcommon terrain twins fold into that doc). Referenced opaquely here (raw
//! pointer only, per the frozen §F seam) exactly as the packets resolve them;
//! reported as missing symbols for the finisher to replace with the real
//! imports once that crate lands. `RmManager` is likewise the state-receiver
//! type pinned by the engine-fork-discovery preamble's receiver order
//! (`cm_load.rs`/`vm_fns.rs` precedent).

use core::ffi::{c_char, c_int};

use mp_game::prelude::byte;
use native_math::vector::{vec3_t, vec3pair_t};
use native_types::thandle_t;

use mp_host_interface::engine_host::EngineHost;

// PORT-NOTE(rm-types): see module doc.
#[allow(dead_code)]
pub struct RmManager;
// PORT-NOTE(rmg-terrain): see module doc.
#[allow(dead_code)]
pub struct CCMLandScape;
#[allow(dead_code)]
pub struct CArea;
#[allow(dead_code)]
pub struct CRandomTerrain;

// ---------------------------------------------------------------------
// Externally-ported callees this file reaches whose bodies are not linked
// into this crate yet — forward-declared with the faithful shape inferred
// from the Raven call sites (receivers per the packets' RESOLVED CALL
// SURFACE tables), matching the established `extern "Rust"` forward-declare
// convention used elsewhere in this crate (`cm_load.rs`/`cm_polylib.rs`).
// PORT-NOTE(callee-signatures): reported in missing_symbols.
//
// `CCMLandScape`/`CArea`/`CRandomTerrain` methods are referenced opaquely by
// their exact Raven member names via C-style free-fn shims (`Class_Method`)
// pending the rmg-terrain.md crate landing — same convention as
// `cm_load.rs`'s `CCMLandScape_DecreaseRefCount` etc.
// ---------------------------------------------------------------------
extern "Rust" {
    fn CCMLandScape_TerrainPatchIterate(
        landscape: *const CCMLandScape,
        IterateFunc: Option<extern "C" fn(*mut (), *mut ())>,
        userdata: *mut (),
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );
    fn CCMLandScape_GetWorldHeight(
        landscape: *const CCMLandScape,
        origin: vec3_t,
        bounds: vec3pair_t,
        aboveGround: bool,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> f32;
    fn CCMLandScape_SaveArea(
        landscape: *mut CCMLandScape,
        area: *mut CArea,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );
    fn CCMLandScape_GetFirstArea(
        landscape: *mut CCMLandScape,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> *mut CArea;
    fn CCMLandScape_GetFirstObjectiveArea(
        landscape: *mut CCMLandScape,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> *mut CArea;
    fn CCMLandScape_GetPlayerArea(
        landscape: *mut CCMLandScape,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> *mut CArea;
    fn CCMLandScape_GetNextArea(
        landscape: *mut CCMLandScape,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> *mut CArea;
    fn CCMLandScape_GetNextObjectiveArea(
        landscape: *mut CCMLandScape,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> *mut CArea;
    fn CCMLandScape_FlattenArea(
        landscape: *mut CCMLandScape,
        area: *mut CArea,
        height: c_int,
        save: bool,
        forceHeight: bool,
        smooth: bool,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );
    fn CCMLandScape_FractionBelowLevel(
        landscape: *mut CCMLandScape,
        area: *mut CArea,
        height: c_int,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> f32;
    fn CCMLandScape_AreaCollision(
        landscape: *mut CCMLandScape,
        area: *mut CArea,
        areaTypes: *mut c_int,
        areaTypeCount: c_int,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> bool;
    fn CCMLandScape_CarveBezierCurve(
        landscape: *mut CCMLandScape,
        numCtls: c_int,
        ctls: *mut vec3_t,
        steps: c_int,
        depth: c_int,
        size: c_int,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );
    fn CCMLandScape_rand_seed(
        landscape: *mut CCMLandScape,
        seed: u32,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );
    fn CCMLandScape_new(
        configstring: *const c_char,
        server: bool,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    ) -> *mut CCMLandScape;
    fn CCMLandScape_SetTerrainId(
        landscape: *mut CCMLandScape,
        terrainId: thandle_t,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );
    fn CRandomTerrain_new(rmg: &mut RmManager, host: &mut dyn EngineHost) -> *mut CRandomTerrain;
    fn CRandomTerrain_Init(
        rt: *mut CRandomTerrain,
        landscape: *mut CCMLandScape,
        heightmap: *mut byte,
        width: c_int,
        height: c_int,
        rmg: &mut RmManager,
        host: &mut dyn EngineHost,
    );

    // PORT-NOTE(q_math-reach): `Info_ValueForKey` (q_shared primitive) is
    // ported in `mp_game`, a tier above this crate's dependency graph
    // (`cm_polylib.rs`/`cm_load.rs` precedent) — not reachable here.
    // Referenced by its exact Raven name; reported as a missing symbol.
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
}

/// Raven `CM_CircularIterate`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1067-1092`
pub fn CM_CircularIterate(
    data: *mut byte,
    width: c_int,
    height: c_int,
    xo: c_int,
    yo: c_int,
    insideRadius: c_int,
    outsideRadius: c_int,
    user: *mut c_int,
    callback: Option<extern "C" fn(*mut byte, f32, *mut c_int)>,
) {
    unsafe {
        let mut y = -outsideRadius;
        while y < outsideRadius + 1 {
            if y + yo >= 0 && y + yo < height {
                let offset = ((outsideRadius * outsideRadius - y * y) as f32).sqrt() as c_int;
                let mut x = -offset;
                while x < offset + 1 {
                    if x + xo >= 0 && x + xo < width {
                        let radius = ((x * x + y * y) as f32).sqrt();

                        if radius >= insideRadius as f32 {
                            let work = data.offset((x + xo) as isize + ((y + yo) * width) as isize);
                            if let Some(cb) = callback {
                                cb(
                                    work,
                                    (radius - insideRadius as f32)
                                        / (outsideRadius - insideRadius) as f32,
                                    user,
                                );
                            }
                        }
                    }
                    x += 1;
                }
            }
            y += 1;
        }
    }
}

/// Raven `CM_ForceHeight`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1094-1097`
pub fn CM_ForceHeight(work: *mut byte, lerp: f32, user: *mut c_int) {
    unsafe {
        *work = (*user).clamp(0, 255) as byte;
    }
}

/// Raven `CM_GetAverage`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1100-1104`
pub fn CM_GetAverage(work: *mut byte, lerp: f32, user: *mut c_int) {
    unsafe {
        *user += *work as c_int;
        *user.offset(1) += 1;
    }
}

/// Raven `CM_Smooth`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1106-1112`
pub fn CM_Smooth(work: *mut byte, lerp: f32, user: *mut c_int) {
    unsafe {
        let smooth =
            (std::f32::consts::PI / 2.0 * 3.0 + (1.0 - lerp) * (std::f32::consts::PI / 2.0)).sin()
                + 1.0;
        // Raven: float smooth = (1.0f - lerp); // commented out in source
        *work = (*work as f32 + (*user as f32 - *work as f32) * smooth) as byte;
    }
}

/// Raven `CM_MakeAverage`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1114-1126`
pub fn CM_MakeAverage(work: *mut byte, lerp: f32, user: *mut c_int) {
    unsafe {
        let height = *work as c_int;
        let mut diff = *user - height;
        if diff.abs() > 3 {
            diff >>= 2;
        }
        let height = height + diff;
        *work = height.clamp(0, 255) as byte;
    }
}

/// Raven `CM_BelowLevel`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1370-1377`
pub fn CM_BelowLevel(data: *mut byte, lerp: f32, info: *mut c_int) {
    unsafe {
        *info.offset(1) += 1;
        if (*data as c_int) < *info.offset(2) {
            *info += 1;
        }
    }
}

/// Raven `CM_TerrainPatchIterate`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1628-1631`
pub fn CM_TerrainPatchIterate(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *const CCMLandScape,
    IterateFunc: Option<extern "C" fn(*mut (), *mut ())>,
    userdata: *mut (),
) {
    unsafe {
        CCMLandScape_TerrainPatchIterate(landscape, IterateFunc, userdata, rmg, host);
    }
}

/// Raven `CM_GetWorldHeight`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1633-1636`
pub fn CM_GetWorldHeight(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *const CCMLandScape,
    origin: vec3_t,
    bounds: vec3pair_t,
    aboveGround: bool,
) -> f32 {
    unsafe { CCMLandScape_GetWorldHeight(landscape, origin, bounds, aboveGround, rmg, host) }
}

/// Raven `CM_SaveArea`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1648-1651`
pub fn CM_SaveArea(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
    area: *mut CArea,
) {
    unsafe {
        CCMLandScape_SaveArea(landscape, area, rmg, host);
    }
}

/// Raven `CM_GetFirstArea`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1663-1666`
pub fn CM_GetFirstArea(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
) -> *mut CArea {
    unsafe { CCMLandScape_GetFirstArea(landscape, rmg, host) }
}

/// Raven `CM_GetFirstObjectiveArea`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1668-1671`
pub fn CM_GetFirstObjectiveArea(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
) -> *mut CArea {
    unsafe { CCMLandScape_GetFirstObjectiveArea(landscape, rmg, host) }
}

/// Raven `CM_GetPlayerArea`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1673-1676`
pub fn CM_GetPlayerArea(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
) -> *mut CArea {
    unsafe { CCMLandScape_GetPlayerArea(landscape, rmg, host) }
}

/// Raven `CM_GetNextArea`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1678-1681`
pub fn CM_GetNextArea(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
) -> *mut CArea {
    unsafe { CCMLandScape_GetNextArea(landscape, rmg, host) }
}

/// Raven `CM_GetNextObjectiveArea`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1683-1686`
pub fn CM_GetNextObjectiveArea(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
) -> *mut CArea {
    unsafe { CCMLandScape_GetNextObjectiveArea(landscape, rmg, host) }
}

/// Raven `CM_FlattenArea`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1638-1641`
pub fn CM_FlattenArea(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
    area: *mut CArea,
    height: c_int,
    save: bool,
    forceHeight: bool,
    smooth: bool,
) {
    unsafe {
        CCMLandScape_FlattenArea(
            landscape,
            area,
            height,
            save,
            forceHeight,
            smooth,
            rmg,
            host,
        );
    }
}

/// Raven `CM_FractionBelowLevel`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1653-1656`
pub fn CM_FractionBelowLevel(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
    area: *mut CArea,
    height: c_int,
) -> f32 {
    unsafe { CCMLandScape_FractionBelowLevel(landscape, area, height, rmg, host) }
}

/// Raven `CM_AreaCollision`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1658-1661`
pub fn CM_AreaCollision(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
    area: *mut CArea,
    areaTypes: *mut c_int,
    areaTypeCount: c_int,
) -> bool {
    unsafe { CCMLandScape_AreaCollision(landscape, area, areaTypes, areaTypeCount, rmg, host) }
}

/// Raven `CM_CarveBezierCurve`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1643-1646`
pub fn CM_CarveBezierCurve(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    landscape: *mut CCMLandScape,
    numCtls: c_int,
    ctls: *mut vec3_t,
    steps: c_int,
    depth: c_int,
    size: c_int,
) {
    unsafe {
        CCMLandScape_CarveBezierCurve(landscape, numCtls, ctls, steps, depth, size, rmg, host);
    }
}

/// Raven `CreateRandomTerrain`.
///
/// Raven guards the live body in `#ifndef PRE_RELEASE_DEMO`; per the
/// project's "FINAL_BUILD/PRE_RELEASE_DEMO undefined" convention (retail
/// build) that guard is always taken. The `CreatePath`/`Generate` calls below
/// it are commented out in Raven's own source (dead in the oracle itself,
/// independent of ruling 25's §20-drop of the RMG generation path).
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1688-1713`
pub fn CreateRandomTerrain(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    config: *const c_char,
    landscape: *mut CCMLandScape,
    heightmap: *mut byte,
    width: c_int,
    height: c_int,
) -> *mut CRandomTerrain {
    unsafe {
        let mut random_terrain: *mut CRandomTerrain = core::ptr::null_mut();

        let seed_str = Info_ValueForKey(config, b"seed\0".as_ptr() as *const c_char);
        let seed = libc::strtoul(seed_str, core::ptr::null_mut(), 10) as u32;

        CCMLandScape_rand_seed(landscape, seed, rmg, host);

        random_terrain = CRandomTerrain_new(rmg, host);
        CRandomTerrain_Init(
            random_terrain,
            landscape,
            heightmap,
            width,
            height,
            rmg,
            host,
        );

        random_terrain
    }
}

/// Raven `CM_InitTerrain`.
///
/// Source: `oracle/codemp/qcommon/cm_terrain.cpp:1618-1626`
pub fn CM_InitTerrain(
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    configstring: *const c_char,
    terrainId: thandle_t,
    server: bool,
) -> *mut CCMLandScape {
    unsafe {
        let ls = CCMLandScape_new(configstring, server, rmg, host);
        CCMLandScape_SetTerrainId(ls, terrainId, rmg, host);
        ls
    }
}
