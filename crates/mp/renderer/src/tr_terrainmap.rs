#![allow(non_snake_case)]

//! The renderer halves of `cm_terrainmap.cpp`.
//!
//! Raven links `cm_terrainmap.cpp` into the same binary as the renderer, so
//! `CTerrainMap` calls `R_LoadImage` and `R_CreateAutomapImage` itself.
//! `mp_engine_qcommon` sits below `mp_renderer` in the crate graph, so the two
//! renderer calls live here and the automap raster crosses as plain data:
//! `R_LoadTerrainMapImages` feeds `CM_TM_Create`, and `R_UploadTerrainAutomap`
//! consumes what `CM_TM_Upload` returns.
//!
//! Both functions run at map load, not on a module trap arm.
//! The render thread owns all GPU state (DEC-63.4).
//!
//! Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:80-96,365-387`

use mp_engine_qcommon::cm::automap_image::AutomapImage;
use mp_engine_qcommon::cm::cm_terrainmap_consts::{TM_HEIGHT, TM_WIDTH};
use mp_engine_qcommon::cm::terrain_map_images::TerrainMapImages;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;

use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_image::{R_CreateAutomapImage, R_LoadImage, TrImageState};
use crate::tr_model::render_models::RenderModels;

/// The five `R_LoadImage` calls of `CTerrainMap::CTerrainMap` and
/// `CTerrainMap::ApplyBackground`, with Raven's qpaths verbatim.
///
/// A failed load leaves Raven's `byte*` null and its width and height
/// untouched; [`AutomapImage::none`] is that state.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:91-95,146`
pub fn R_LoadTerrainMapImages(view: &mut EngineHostView) -> TerrainMapImages {
    TerrainMapImages {
        background: load(view, "gfx\\menus\\rmg\\01_bg"),
        start: load(view, "gfx/menus/rmg/start"),
        end: load(view, "gfx/menus/rmg/end"),
        objective: load(view, "gfx/menus/rmg/objective"),
        building: load(view, "gfx/menus/rmg/building"),
    }
}

/// One `R_LoadImage` call, as `CTerrainMap` wants it.
fn load(view: &mut EngineHostView, shortname: &str) -> AutomapImage {
    match R_LoadImage(view, shortname) {
        Some((pic, width, height, _format)) => AutomapImage::from_rgba(&pic, width, height),
        None => AutomapImage::none(),
    }
}

/// Raven `CTerrainMap::Upload`'s renderer call: register the finished automap
/// raster as the `"*automap"` image.
///
/// `pic` is what `CM_TM_Upload` returned, `TM_WIDTH * TM_HEIGHT * 4` bytes.
///
/// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:384`
#[allow(clippy::too_many_arguments)]
pub fn R_UploadTerrainAutomap(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    sim: &mut RenderAssetsSim,
    models: &RenderModels,
    state: &mut TrImageState,
    pic: &[u8],
) {
    R_CreateAutomapImage(
        view, cvars, sim, models, state, "*automap", pic, TM_WIDTH, TM_HEIGHT, false, false,
        true, 0,
    );
}
