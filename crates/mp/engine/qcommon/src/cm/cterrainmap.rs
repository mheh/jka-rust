#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_int;

use native_math::vector::vec3_t;

use crate::cm_terrain::CmLandScape;

//TODO: Port CTerrainMap
// Source: oracle/codemp/qcommon/cm_terrainmap.h:17-60
// The automap raster lane (gh#29, DEC-55.4) owns the image buffers, the symbol
// bitmaps, and every method body below.
// The `cm_terrainmap.rs` forwarders are transcribed, so this type declares the
// surface they call and each method panics until the lane lands.

/// Raven `CTerrainMap` — the automap image for the current landscape.
///
/// Type definition source: `oracle/codemp/qcommon/cm_terrainmap.h:17-60`
pub struct CTerrainMap {}

impl CTerrainMap {
    /// Raven `CTerrainMap::CTerrainMap` builds the automap image from the
    /// landscape.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:46-97`
    pub fn new(_landscape: *mut CmLandScape) -> Self {
        todo!("Port CTerrainMap::CTerrainMap — oracle/codemp/qcommon/cm_terrainmap.cpp:46-97")
    }

    /// Raven `CTerrainMap::ConvertPos` maps a world position into automap
    /// pixel space.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:236-247`
    pub fn ConvertPos(&mut self, _x: &mut c_int, _y: &mut c_int) {
        todo!("Port CTerrainMap::ConvertPos — oracle/codemp/qcommon/cm_terrainmap.cpp:236-247")
    }

    /// Raven `CTerrainMap::AddBuilding` draws a building symbol.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:276-283`
    pub fn AddBuilding(&mut self, _x: c_int, _y: c_int, _side: c_int) {
        todo!("Port CTerrainMap::AddBuilding — oracle/codemp/qcommon/cm_terrainmap.cpp:276-283")
    }

    /// Raven `CTerrainMap::AddStart` draws a start symbol.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:249-256`
    pub fn AddStart(&mut self, _x: c_int, _y: c_int, _side: c_int) {
        todo!("Port CTerrainMap::AddStart — oracle/codemp/qcommon/cm_terrainmap.cpp:249-256")
    }

    /// Raven `CTerrainMap::AddEnd` draws an end symbol.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:258-265`
    pub fn AddEnd(&mut self, _x: c_int, _y: c_int, _side: c_int) {
        todo!("Port CTerrainMap::AddEnd — oracle/codemp/qcommon/cm_terrainmap.cpp:258-265")
    }

    /// Raven `CTerrainMap::AddObjective` draws an objective symbol.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:267-274`
    pub fn AddObjective(&mut self, _x: c_int, _y: c_int, _side: c_int) {
        todo!("Port CTerrainMap::AddObjective — oracle/codemp/qcommon/cm_terrainmap.cpp:267-274")
    }

    /// Raven `CTerrainMap::AddNPC` draws an NPC marker.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:285-294`
    pub fn AddNPC(&mut self, _x: c_int, _y: c_int, _friendly: bool) {
        todo!("Port CTerrainMap::AddNPC — oracle/codemp/qcommon/cm_terrainmap.cpp:285-294")
    }

    /// Raven `CTerrainMap::AddWallRect` draws a wall rectangle.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:304-321`
    pub fn AddWallRect(&mut self, _x: c_int, _y: c_int, _side: c_int) {
        todo!("Port CTerrainMap::AddWallRect — oracle/codemp/qcommon/cm_terrainmap.cpp:304-321")
    }

    /// Raven `CTerrainMap::AddNode` draws a nav node marker.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:296-302`
    pub fn AddNode(&mut self, _x: c_int, _y: c_int) {
        todo!("Port CTerrainMap::AddNode — oracle/codemp/qcommon/cm_terrainmap.cpp:296-302")
    }

    /// Raven `CTerrainMap::Upload` draws the player and sends the image to the
    /// renderer.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:365-387`
    pub fn Upload(&mut self, _player_origin: vec3_t, _player_angles: vec3_t) {
        todo!("Port CTerrainMap::Upload — oracle/codemp/qcommon/cm_terrainmap.cpp:365-387")
    }

    /// Raven `CTerrainMap::SaveImageToDisk` writes the automap image as a file.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:389-395`
    pub fn SaveImageToDisk(
        &mut self,
        _terrainName: *const c_char,
        _missionName: *const c_char,
        _seed: *const c_char,
    ) {
        todo!("Port CTerrainMap::SaveImageToDisk — oracle/codemp/qcommon/cm_terrainmap.cpp:389-395")
    }
}
