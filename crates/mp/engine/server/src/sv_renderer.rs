//! Dedicated-server renderer entry points the `jamp` server links out of the
//! WinDed DEDICATED renderer subset (`tr_model.cpp`/`tr_shader.cpp`), reached
//! directly from `sv_init.cpp`'s `SV_SpawnServer`.
//!
//! These are thin server-crate glue: the real model-registry logic is the
//! FROZEN `tr-model.md` reimplementation, owned by `Engine.render_models` and
//! ported as `impl RenderModels` methods in `mp_renderer` (real-definitions
//! LAW — reused, never re-transcribed). The server threads the model registry
//! as qcommon's type-erased `cm_load::RenderModels` slot (opaque-slot ruling);
//! each entry casts it back to the real `RenderModels` (`rm_from_slot`) and
//! forwards to the method.
//!
//! Source: `oracle/codemp/renderer/tr_model.cpp` (WinDed DEDICATED link set,
//! `docs/plans/2026-07-08-mp-engine-build-out.md`).

use mp_qshared::shared::force_reload::ForceReload_e;
use mp_qshared::shared::{qboolean, qhandle_t};

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_renderer::tr_model::render_models::RenderModels as RealRenderModels;

use crate::server_host::rm_from_slot;

/// Raven `R_SVModelInit` — the dedicated-server model-init entry (a bare wrapper
/// over `R_ModelInit`; the `#endif // !DEDICATED` sits directly above it, so it
/// is the always-compiled server entry). Forwards to the FROZEN `tr-model.md`
/// `RenderModels::model_init` (resets `tr.numModels`/`mhHashTable`, reserves
/// `models[0]` as the `MOD_BAD` NULL model).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1655-1657` (`R_SVModelInit`);
/// body `:1662-1679` (`R_ModelInit`).
pub fn R_SVModelInit(view: &mut EngineHostView) {
    // SAFETY: `view.rm` is the `cm_load::RenderModels` slot armed by the caller
    // from the live `Engine.render_models`; `rm_from_slot`'s contract holds.
    let rm = unsafe { rm_from_slot(&mut view.rm) };
    rm.model_init();
}

/// Raven `RE_RegisterMedia_LevelLoadBegin` — the level-load media bracket the
/// server opens at the top of `SV_SpawnServer`. Forwards to the FROZEN
/// `tr-model.md` `RenderModels::media_level_load_begin` (force-reload eviction /
/// `sv_pure` dump, `tr.numBSPModels = 0`, and the level-counter bump when the
/// map name changes). The `#ifndef DEDICATED` `R_Images_DeleteLightMaps()` tail
/// is folded dead in that method (§C10, dedicated build).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:522-566`.
pub fn RE_RegisterMedia_LevelLoadBegin(
    view: &mut EngineHostView,
    ps_map_name: &str,
    e_force_reload: ForceReload_e,
) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of
    // this slot for the borrow's duration (`view.rm` casts back to the live
    // `Engine.render_models`).
    let rm = unsafe { &mut *(view.rm.as_raw() as *mut RealRenderModels) };
    rm.media_level_load_begin(view, ps_map_name, e_force_reload);
}

/// Raven `R_InitShaders(qboolean server)`. On the WinDed DEDICATED build the
/// body reduces to `Com_Memset(hashTable, 0)` + `deferLoad = qfalse` — both are
/// `tr_shader.cpp` file statics; the `#ifndef DEDICATED` load branch
/// (`CreateInternalShaders`/`ScanAndLoadShaderFiles`/`CreateExternalShaders`)
/// compiles out entirely. The hash table's server-skin rows live on as the
/// `RenderModels` server-shader name pool (user ruling 2026-07-12, server
/// skins name-pool), reset by `init_skins`/`hunk_clear` — `SV_SpawnServer`
/// always runs those and this back-to-back — so nothing is left for this
/// entry; the rest of `tr_shader.cpp` stays §20-dropped.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4265-4280`.
pub fn R_InitShaders(_view: &mut EngineHostView, _server: qboolean) {
    // `Com_Memset(hashTable, 0)` — covered by `init_skins`'s pool reset (see
    // doc comment); `deferLoad = qfalse` is a §20-dropped client static.
}

/// Raven `R_InitSkins` — reset the skin registry to the single default skin.
/// Forwards to `RenderModels::init_skins`, the server-skins name-pool home
/// (user ruling 2026-07-12, amending `tr-model.md`).
///
/// Source: `oracle/codemp/renderer/tr_image.cpp:3324-3334`.
pub fn R_InitSkins(view: &mut EngineHostView) {
    // SAFETY: as `R_SVModelInit`.
    let rm = unsafe { rm_from_slot(&mut view.rm) };
    rm.init_skins();
}

/// Raven `RE_RegisterServerSkin` — the `G_R_REGISTERSKIN` vmcall target
/// ("Mangled version of the above function to load .skin files on the
/// server"). Forwards to `RenderModels::register_server_skin` (user ruling
/// 2026-07-12, server skins name-pool).
///
/// Source: `oracle/codemp/renderer/tr_image.cpp:3301-3318`.
pub fn RE_RegisterServerSkin(view: &mut EngineHostView, name: &str) -> qhandle_t {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of
    // this slot for the borrow's duration.
    let rm = unsafe { &mut *(view.rm.as_raw() as *mut RealRenderModels) };
    rm.register_server_skin(view, name)
}
