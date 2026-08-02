#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use native_math::vector::vec3_t;

use crate::cm_terrain::CmLandScape;

/// Every landscape read `CTerrainMap` makes, gathered as one borrow.
///
/// Raven stores `CCMLandScape *mLandscape` and reaches back through it. The
/// automap lives on the same `CollisionWorld` as the landscape, so a stored
/// borrow would alias its owner; naming the six reads instead keeps the automap
/// free of a back pointer and lets a parity test drive it without a full
/// landscape.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:66,178-180,238-239`
#[derive(Clone, Copy, Debug)]
pub struct TerrainMapLandscape<'a> {
    /// `GetHeightMap()`
    pub height_map: &'a [u8],
    /// `GetRealWidth()`
    pub real_width: c_int,
    /// `GetRealHeight()`
    pub real_height: c_int,
    /// `GetBaseWaterHeight()`
    pub base_water_height: c_int,
    /// `GetMins()`
    pub mins: vec3_t,
    /// `GetSize()`
    pub size: vec3_t,
}

impl<'a> From<&'a CmLandScape> for TerrainMapLandscape<'a> {
    fn from(land: &'a CmLandScape) -> Self {
        TerrainMapLandscape {
            height_map: land.height_map(),
            real_width: land.real_width(),
            real_height: land.real_height(),
            base_water_height: land.base_water_height(),
            mins: land.mins(),
            size: land.size(),
        }
    }
}
