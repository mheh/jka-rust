//! Renderer-side `EngineHooks` installation: the model-cache upcalls WinDed
//! links from the REAL `tr_model.cpp` plus the accessor hooks backing
//! qcommon's live `EngineHost` implementation — the casting adapters live here
//! because only this crate can name the real `RenderModels` (host-seam
//! restructure, user ruling 2026-07-11).
//!
//! Slot-cast rule (per-slot): the cast copies the raw pointer out of the
//! view's type-erased slot (`as_raw()`), so the view stays usable — sound as
//! long as nothing called while the cast borrow is live casts the SAME slot
//! again (the `models_level_load_end` host calls are print/cvar reads, which
//! touch only `view.common`).

use core::ffi::c_void;

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::EngineHooks;
use mp_qshared::shared::{qboolean, qfalse, qhandle_t};

use crate::tr_model::render_models::RenderModels;

/// Cast the view's type-erased `rm` slot back to the live `RenderModels`. The
/// raw pointer is copied out first (`as_raw`), so the returned borrow is NOT
/// tied to the view — the per-slot rule above governs its use.
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.render_models`; the engine is
/// single-threaded and no other cast of this slot is live for the returned
/// borrow's duration.
unsafe fn rm_from_view<'a>(view: &mut EngineHostView) -> &'a mut RenderModels {
    &mut *(view.rm.as_raw() as *mut RenderModels)
}

/// Install the renderer-model tier's hook fields.
pub fn install_engine_hooks(hooks: &mut EngineHooks) {
    hooks.RE_RegisterModels_LevelLoadEnd = Some(re_register_models_level_load_end_hook);
    hooks.R_HunkClearCrap = Some(r_hunk_clear_crap_hook);
    hooks.R_ModelMdxm = Some(r_model_mdxm_hook);
    hooks.R_ModelMdxa = Some(r_model_mdxa_hook);
    hooks.R_SkinSurfaces = Some(r_skin_surfaces_hook);
    hooks.R_RegisterServerModel = Some(r_register_server_model_hook);
}

/// Raven `RE_RegisterModels_LevelLoadEnd` — the live eviction path.
/// Source: `oracle/codemp/renderer/tr_model.cpp:337-409`
fn re_register_models_level_load_end_hook(
    view: &mut EngineHostView,
    delete_all_unused: qboolean,
) -> qboolean {
    // SAFETY: view-constructor slot, single-threaded; the eviction's host
    // calls (print/cvar) touch only `view.common`, never this slot.
    let rm = unsafe { rm_from_view(view) };
    rm.models_level_load_end(&mut *view, delete_all_unused != qfalse) as qboolean
}

/// Raven `R_HunkClearCrap`.
/// Source: `oracle/codemp/renderer/tr_model.cpp:1682-1690`
fn r_hunk_clear_crap_hook(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };
    rm.hunk_clear();
}

/// `EngineHost::model_mdxm` backing — Raven `R_GetModelByHandle(h)->mdxm`.
/// Source: `oracle/codemp/renderer/tr_local.h:1128`
fn r_model_mdxm_hook(view: &mut EngineHostView, model: qhandle_t) -> *mut c_void {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };
    rm.get_model(model).mdxm as *mut c_void
}

/// `EngineHost::model_mdxa` backing — Raven `R_GetModelByHandle(h)->mdxa`.
/// Source: `oracle/codemp/renderer/tr_local.h:1129`
fn r_model_mdxa_hook(view: &mut EngineHostView, model: qhandle_t) -> *mut c_void {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };
    rm.get_model(model).mdxa as *mut c_void
}

/// `EngineHost::skin_surfaces` backing — Raven `R_GetSkinByHandle` flattened
/// (server skins name-pool ruling, 2026-07-12).
/// Source: `oracle/codemp/renderer/tr_image.cpp:3342-3347`
fn r_skin_surfaces_hook(view: &mut EngineHostView, h_skin: qhandle_t) -> Vec<(String, String)> {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };
    rm.skin_surfaces(h_skin)
}

/// `EngineHost::model_register` backing — Raven `RE_RegisterServerModel`.
/// Source: `oracle/codemp/renderer/tr_model.cpp:588`
fn r_register_server_model_hook(view: &mut EngineHostView, name: &str) -> qhandle_t {
    // SAFETY: view-constructor slot, single-threaded; register_server_model's
    // host calls (fs/pak/print) never touch this slot (per-slot rule).
    let rm = unsafe { rm_from_view(view) };
    rm.register_server_model(&mut *view, name)
}
