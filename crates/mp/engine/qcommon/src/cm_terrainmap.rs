//! `cm_terrainmap.cpp` — the automap terrain overlay.
//!
//! Raven keeps one file-scope `CTerrainMap *TerrainMap` pointer (ruling 2);
//! it threads here as `cm.terrain_map` on `CollisionWorld`.
//! Source: `oracle/codemp/qcommon/cm_terrainmap.cpp`

use core::ffi::c_char;
use core::ffi::c_int;

use native_math::vector::vec3_t;

use crate::cm::cm_terrainmap_consts::{SIDE_BLUE, SIDE_RED, TM_HEIGHT, TM_WIDTH};
use crate::cm_terrain::CmLandScape;
use crate::collision_world::CollisionWorld;

//TODO: Port CPixel32
// Source: oracle/codemp/qcommon/cm_draw.h:24-60
//TODO: Port CTerrainMap
// Source: oracle/codemp/qcommon/cm_terrainmap.h:17-60
// Both classes belong to the automap raster lane (gh#29).

/// Raven `SideColor` picks the automap wall color for a terrain side flag.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:29-44`
pub fn SideColor(side: c_int) -> CPixel32 {
    let mut col = CPixel32::new(255, 255, 255);
    match side {
        SIDE_BLUE => col = CPixel32::new(0, 0, 192),
        SIDE_RED => col = CPixel32::new(192, 0, 0),
        _ => {}
    }
    col
}

/// Raven `CM_TM_Free` deletes the active terrain map and clears the pointer.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:407-414`
pub fn CM_TM_Free(cm: &mut CollisionWorld) {
    if cm.terrain_map.is_some() {
        cm.terrain_map = None;
    }
}

/// Raven `CM_TM_Create` frees an existing terrain map, then builds a new one
/// from the landscape.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:397-405`
/// Raven names the parameter type `CCMLandScape`. This crate ports that class
/// as `CmLandScape`, so the parameter takes the ported name.
pub fn CM_TM_Create(cm: &mut CollisionWorld, landscape: *mut CmLandScape) {
    if cm.terrain_map.is_some() {
        CM_TM_Free(cm);
    }

    cm.terrain_map = Some(CTerrainMap::new(landscape));
}

/// Raven `CM_TM_AddNPC` records an NPC marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:440-446`
pub fn CM_TM_AddNPC(cm: &mut CollisionWorld, x: c_int, y: c_int, friendly: bool) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddNPC(x, y, friendly);
    }
}

/// Raven `CM_TM_AddNode` records a nav node marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:448-454`
pub fn CM_TM_AddNode(cm: &mut CollisionWorld, x: c_int, y: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddNode(x, y);
    }
}

/// Raven `CM_TM_AddStart` records a start marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:416-422`
pub fn CM_TM_AddStart(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddStart(x, y, side);
    }
}

/// Raven `CM_TM_AddEnd` records an end marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:424-430`
pub fn CM_TM_AddEnd(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddEnd(x, y, side);
    }
}

/// Raven `CM_TM_AddObjective` records an objective marker on the active
/// terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:432-438`
pub fn CM_TM_AddObjective(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddObjective(x, y, side);
    }
}

/// Raven `CM_TM_AddBuilding` records a building marker on the active terrain
/// map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:456-462`
pub fn CM_TM_AddBuilding(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddBuilding(x, y, side);
    }
}

/// Raven `CM_TM_AddWallRect` records a wall rectangle on the active terrain
/// map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:464-470`
pub fn CM_TM_AddWallRect(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddWallRect(x, y, side);
    }
}

/// Raven `CM_TM_Upload` uploads the player position and angles for the
/// active terrain map to render.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:472-478`
pub fn CM_TM_Upload(cm: &mut CollisionWorld, player_origin: vec3_t, player_angles: vec3_t) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.Upload(player_origin, player_angles);
    }
}

/// Raven `CM_TM_SaveImageToDisk` writes the automap image to disk.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:480-486`
pub fn CM_TM_SaveImageToDisk(
    cm: &mut CollisionWorld,
    terrainName: *const c_char,
    missionName: *const c_char,
    seed: *const c_char,
) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        // write out automap
        terrain_map.SaveImageToDisk(terrainName, missionName, seed);
    }
}

/// Raven `CM_TM_ConvertPosition` maps automap-space coordinates into a
/// caller-given pixel rectangle.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:488-496`
pub fn CM_TM_ConvertPosition(
    cm: &mut CollisionWorld,
    x: &mut c_int,
    y: &mut c_int,
    Width: c_int,
    Height: c_int,
) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.ConvertPos(x, y);
        *x = *x * Width / TM_WIDTH;
        *y = *y * Height / TM_HEIGHT;
    }
}
