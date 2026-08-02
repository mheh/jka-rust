//! `cm_terrainmap.cpp` - the automap terrain overlay.
//!
//! Raven keeps one file-scope `CTerrainMap *TerrainMap` pointer (ruling 2);
//! it threads here as `cm.terrain_map` on `CollisionWorld`.
//!
//! Two `CM_TM_*` entry points reach the renderer in Raven and cannot here,
//! because `mp_engine_qcommon` sits below `mp_renderer`:
//! `CM_TM_Create` takes the five already-loaded source images, and
//! `CM_TM_Upload` returns the finished RGBA raster for the caller to upload.
//! `mp_renderer::tr_terrainmap` holds both renderer halves.
//!
//! Raven's `CM_TM_Upload`, `CM_TM_ConvertPosition`, `CM_TM_AddNode`, and
//! `CM_TM_AddNPC` have no caller in either tree. They stay ported because they
//! are the header's declared surface and the automap's only read-back path;
//! the live MP callers are `RM_Mission.cpp`, `RM_Instance.cpp`, and
//! `RM_Manager.cpp`.
//!
//! Source: `oracle/codemp/qcommon/cm_terrainmap.cpp`

use core::ffi::c_int;

use native_math::vector::vec3_t;

use crate::cm::cm_terrainmap_consts::{SIDE_BLUE, SIDE_RED, TM_HEIGHT, TM_WIDTH};
use crate::cm::cpixel32::CPixel32;
use crate::cm::cterrainmap::CTerrainMap;
use crate::cm::terrain_map_images::TerrainMapImages;
use crate::cm::terrain_map_landscape::TerrainMapLandscape;
use crate::collision_world::CollisionWorld;

/// Raven `SideColor` picks the automap symbol color for a team side flag.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:29-44`
#[allow(non_snake_case)]
pub fn SideColor(side: c_int) -> CPixel32 {
    let mut col = CPixel32::new(255, 255, 255, 255);
    match side {
        SIDE_BLUE => col = CPixel32::new(0, 0, 192, 255),
        SIDE_RED => col = CPixel32::new(192, 0, 0, 255),
        _ => {}
    }
    col
}

/// Raven `CM_TM_Free` deletes the active terrain map and clears the pointer.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:407-414`
#[allow(non_snake_case)]
pub fn CM_TM_Free(cm: &mut CollisionWorld) {
    if cm.terrain_map.is_some() {
        cm.terrain_map = None;
    }
}

/// Raven `CM_TM_Create` frees an existing terrain map, then builds a new one
/// from the landscape.
///
/// Raven passes the landscape pointer; the port resolves the one landscape from
/// `cm.land_scape` and does nothing when there is none, because a borrowed
/// landscape beside `&mut CollisionWorld` would alias the field it lives in.
/// `images` carries the five renderer images Raven's constructor loads itself.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:397-405`
#[allow(non_snake_case)]
pub fn CM_TM_Create(cm: &mut CollisionWorld, images: TerrainMapImages) {
    if cm.terrain_map.is_some() {
        CM_TM_Free(cm);
    }

    let Some(landscape) = cm.land_scape.as_ref() else {
        return;
    };
    let map = CTerrainMap::new(TerrainMapLandscape::from(landscape), images);
    cm.terrain_map = Some(Box::new(map));
}

/// Raven `CM_TM_AddNPC` records an NPC marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:440-446`
#[allow(non_snake_case)]
pub fn CM_TM_AddNPC(cm: &mut CollisionWorld, x: c_int, y: c_int, friendly: bool) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddNPC(x, y, friendly);
    }
}

/// Raven `CM_TM_AddNode` records a nav node marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:448-454`
#[allow(non_snake_case)]
pub fn CM_TM_AddNode(cm: &mut CollisionWorld, x: c_int, y: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddNode(x, y);
    }
}

/// Raven `CM_TM_AddStart` records a start marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:416-422`
#[allow(non_snake_case)]
pub fn CM_TM_AddStart(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddStart(x, y, side);
    }
}

/// Raven `CM_TM_AddEnd` records an end marker on the active terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:424-430`
#[allow(non_snake_case)]
pub fn CM_TM_AddEnd(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddEnd(x, y, side);
    }
}

/// Raven `CM_TM_AddObjective` records an objective marker on the active
/// terrain map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:432-438`
#[allow(non_snake_case)]
pub fn CM_TM_AddObjective(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddObjective(x, y, side);
    }
}

/// Raven `CM_TM_AddBuilding` records a building marker on the active terrain
/// map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:456-462`
#[allow(non_snake_case)]
pub fn CM_TM_AddBuilding(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddBuilding(x, y, side);
    }
}

/// Raven `CM_TM_AddWallRect` records a wall rectangle on the active terrain
/// map.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:464-470`
#[allow(non_snake_case)]
pub fn CM_TM_AddWallRect(cm: &mut CollisionWorld, x: c_int, y: c_int, side: c_int) {
    if let Some(terrain_map) = cm.terrain_map.as_mut() {
        terrain_map.AddWallRect(x, y, side);
    }
}

/// Raven `CM_TM_Upload` composes the automap with the player marker and hands
/// the raster to the renderer.
///
/// The port returns the `TM_WIDTH` by `TM_HEIGHT` RGBA bytes and `None` when no
/// terrain map is active. `mp_renderer::tr_terrainmap::R_UploadTerrainAutomap`
/// makes Raven's `R_CreateAutomapImage` call with them.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:472-478`
#[allow(non_snake_case)]
pub fn CM_TM_Upload(
    cm: &mut CollisionWorld,
    player_origin: Option<vec3_t>,
    player_angles: vec3_t,
) -> Option<Vec<u8>> {
    cm.terrain_map
        .as_mut()
        .map(|terrain_map| terrain_map.Upload(player_origin, player_angles))
}

/// Raven `CM_TM_SaveImageToDisk` writes the automap image to disk.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:480-486`
#[allow(non_snake_case)]
pub fn CM_TM_SaveImageToDisk(
    cm: &mut CollisionWorld,
    terrainName: &str,
    missionName: &str,
    seed: &str,
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
#[allow(non_snake_case)]
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
