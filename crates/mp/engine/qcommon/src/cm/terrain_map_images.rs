#![allow(non_camel_case_types, non_snake_case)]

use crate::cm::automap_image::AutomapImage;

/// The five images `CTerrainMap`'s constructor loads: the background tile and
/// the four map symbols.
///
/// Raven calls `R_LoadImage` inside the constructor. `mp_engine_qcommon` sits
/// below `mp_renderer` in the crate graph and cannot call it, so the caller
/// loads the five images and hands them in. `mp_renderer`'s
/// `R_LoadTerrainMapImages` is the transcription of Raven's five calls and
/// their qpaths.
///
/// Type definition source: `oracle/codemp/qcommon/cm_terrainmap.cpp:80-96,146`
#[derive(Clone, Default, Debug)]
pub struct TerrainMapImages {
    /// `"gfx\menus\rmg\01_bg"`.
    pub background: AutomapImage,
    /// `"gfx/menus/rmg/start"`.
    pub start: AutomapImage,
    /// `"gfx/menus/rmg/end"`.
    pub end: AutomapImage,
    /// `"gfx/menus/rmg/objective"`.
    pub objective: AutomapImage,
    /// `"gfx/menus/rmg/building"`.
    pub building: AutomapImage,
}
