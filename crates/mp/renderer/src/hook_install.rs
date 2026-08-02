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

use crate::renderer_frontend::RendererFrontend;
use crate::tr_model::render_models::RenderModels;

/// Cast the view's type-erased `rm` slot back to the live `RenderModels`. The
/// raw pointer is copied out first (`as_raw`), so the returned borrow is NOT
/// tied to the view — the per-slot rule above governs its use.
///
/// This is the renderer-reach half that serves every `RE_*` receiver named
/// `rm`/`models` (the view's doc records the whole reach).
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.render_models`; the engine is
/// single-threaded and no other cast of this slot is live for the returned
/// borrow's duration.
pub unsafe fn rm_from_view<'a>(view: &mut EngineHostView) -> &'a mut RenderModels {
    &mut *(view.rm.as_raw() as *mut RenderModels)
}

/// Cast the view's type-erased `re` slot back to the live [`RendererFrontend`]
/// carrier bundle — the client's one reach to the `RE_*` receivers (DEC-59.1).
/// A call site splits the returned bundle into the individual receivers its
/// `RE_*` function declares. The DEC-55.2 partition binds module trap arms:
/// `assets` and `frame_data` on a synchronous trap, never `gpu_res`.
///
/// The DEC-60.1 re-audit ran with the gh#22 thread split (2026-08-02) and found
/// no GPU-tier access to re-home: `GpuResources` holds one empty
/// `GlStatePlaceholder`, and no function in this crate reads or writes it. Every
/// real GPU object lives in `mp_renderer_gpu` on the render thread, which the
/// sim thread has no handle to. The 104 `gpu_res` parameters are call shape for
/// the R4 wave that fills the struct; that wave must move it to the render
/// thread rather than keep it in this bundle.
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.re`; the engine is single-threaded and no
/// other cast of this slot is live for the returned borrow's duration. The slot
/// is NULL on dedicated (`Engine.re` is `None`), so a dedicated build must
/// never reach this — the client tier is the only caller.
pub unsafe fn re_from_view<'a>(view: &mut EngineHostView) -> &'a mut RendererFrontend {
    &mut *(view.re.as_raw() as *mut RendererFrontend)
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
/// Returns the `(block, parsed)` pair (DEC-35): the `.glm` block pointer and its
/// parse-once `MdxmParsed` sidecar pointer, both null when absent.
/// Source: `oracle/codemp/renderer/tr_local.h:1128`
fn r_model_mdxm_hook(view: &mut EngineHostView, model: qhandle_t) -> (*mut c_void, *const c_void) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };
    rm.model_mdxm_ptrs(model)
}

/// `EngineHost::model_mdxa` backing — Raven `R_GetModelByHandle(h)->mdxa`.
/// Returns the `(block, parsed)` pair (DEC-35), with the same `animIndex`
/// resolution the raw pointer path used (a GLM handle resolves its GLA through
/// `mdxm->animIndex`). Both null when the resolved loader pointer is NULL.
/// Source: `oracle/codemp/renderer/tr_local.h:1129`
fn r_model_mdxa_hook(view: &mut EngineHostView, model: qhandle_t) -> (*mut c_void, *const c_void) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };
    rm.model_mdxa_ptrs(model)
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
