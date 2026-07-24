#![allow(non_camel_case_types, non_snake_case)]

//! `G2API` save/load — the server-side save-game (de)serialization entry
//! points for the `CGhoul2Info_v` instance vector, plus the two `.gla`
//! filename getters that share this file per the roster.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_saveload.rs`, class
//! "G2API save/load"): `SaveGhoul2Models`/`LoadGhoul2Models`/`FreeSaveBuffer`/
//! `LoadSaveCodeDestructGhoul2Info`, `GetAnimFileName`(`+Index`), `GetGLAName`.
//!
//! Every `G2API_*` entry keeps its 1:1 signature (`G2SV-D6`) and threads
//! `g2: &mut Ghoul2System`/`&Ghoul2System` (ruling 4/11, state threaded not
//! reached), plus `host: &mut impl EngineHost` wherever the body reaches
//! model-memory (`G2_SetupModelPointers`'s `RE_RegisterModel`/
//! `R_GetModelByHandle` chain, ruling 36 `model_mdxm`) — matching the
//! `api_bones.rs`/`api_collision.rs` sibling convention. `Save`/`Load`
//! themselves are pure in-memory (de)serialization with no host touch in the
//! oracle (the caller does the actual `fs_read_file`/`fs_write_file` around
//! these two), so they thread `g2` only, no `host`.
//!
//! **Doc/oracle mismatch #1, reported (not improvised around):** the `## Seam
//! definition`'s generalized `G2SV-D1` out-param discriminator names
//! `SaveGhoul2Models` alongside `GetRagBonePos`/`GetAnimFileName` as an example
//! of the "write-on-success-only → `Option`" class. Reading `G2_SaveGhoul2Models`
//! (`oracle/codemp/ghoul2/G2_misc.cpp:1719-1798`, the callee
//! `G2API_SaveGhoul2Models` forwards to verbatim) against the discriminator's
//! own rule ("classifies... by reading only its failure path") shows it has
//! **no failure path at all**: every branch — the empty-`ghoul2` short-circuit
//! (`:1721-1727`) and the populated-buffer path (`:1729-1797`) — ends
//! `return qtrue;` after writing both `*buffer`/`*size`. So this function
//! doesn't cleanly fit either named class; `g2api_save_ghoul2_models` below
//! keeps the doc's stated `Option`-returning shape for roster consistency
//! (mirroring how `api_bones.rs`'s `g2api_set_bone_anim` keeps its doc-frozen
//! shape despite a similar sibling inconsistency), but the returned value is
//! always `Some` in a faithful transcription — flagged upstream, not silently
//! reclassified as write-through here.
//!
//! **Doc/oracle mismatch #2, reported (bigger, not improvised around): six of
//! this file's seven rostered functions appear graph-dead in MP.** The MP
//! trap surface (`oracle/codemp/game/g_local.h`'s `trap_G2API_*`
//! declarations + `oracle/codemp/server/sv_game.cpp`'s `G_G2_*` switch,
//! `:1316-1605`) wires **only** `G_G2_GETGLANAME` (`:1387`) from this file's
//! roster. There is no `G_G2_SAVEGHOUL2MODELS`/`LOADGHOUL2MODELS`/
//! `FREESAVEBUFFER`/`LOADSAVECODEDESTRUCTGHOUL2INFO`/`GETANIMFILENAME`/
//! `GETANIMFILENAMEINDEX` syscall arm anywhere in `sv_game.cpp`, and `grep`
//! across all of `oracle/codemp/` finds no caller of those six functions at
//! all outside `ghoul2/G2_API.cpp` itself. Their only real callers live in the
//! **SP** tree (`oracle/code/game/g_savegame.cpp:856,925,1058`), reached
//! through SP's direct `gi.`-function-pointer import table
//! (`oracle/code/server/sv_game.cpp:620-623`) — a calling convention MP's VM
//! syscall dispatch doesn't have, because **`oracle/codemp/` has no
//! `g_savegame.cpp` at all** (MP dedicated servers have no save-game system).
//! This is the same zero-caller shape the doc's own `divergences` list already
//! applies to `G2API_AddSkinGore`/`ResetGoreTag`/`G2_GetGoreRecord` (§20 drop),
//! but the doc rosters these six under "ports fully" instead. Per this task's
//! instructions a wrong/questionable roster entry is reported, not
//! unilaterally dropped, so all seven stubs below are still transcribed
//! (`GetGLAName` is genuinely live; the other six are stubbed per the doc as
//! written, pending a ruling on whether they belong on the §20 list instead).

use mp_host_interface::EngineHost;
use mp_qshared::shared::qhandle_t;

use crate::ghoul2_system::Ghoul2System;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

/// Raven `qboolean G2API_SaveGhoul2Models(CGhoul2Info_v &ghoul2, char **buffer,
/// int *size)` — forwards to `G2_SaveGhoul2Models` (`G2_misc.cpp:1719-1798`,
/// `misc.rs`): serializes every instance's `mModelindex`..`mTransformedVertsArray`
/// block plus its surface/bone/bolt lists into one flat buffer (a 4-byte
/// zero-count buffer when `ghoul2` is empty).
///
/// Out-params `char **buffer`/`int *size` fold into one owned `Vec<u8>`
/// return (`size` is the `Vec`'s length, per §C7); see this file's module-doc
/// note on the `G2SV-D1` discriminator mismatch — the oracle body never
/// returns `qfalse`, so the `Option` is always `Some` in a faithful port.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2472-2475`
pub fn g2api_save_ghoul2_models(g2: &Ghoul2System, ghoul2: &CGhoul2Info_v) -> Option<Vec<u8>> {
    // Raven: `return G2_SaveGhoul2Models(ghoul2, buffer, size);` — forwards
    // verbatim; see this file's module-doc note on the `G2SV-D1`
    // discriminator mismatch (no real failure path, so this is always `Some`).
    crate::misc::g2_save_ghoul2_models(g2, ghoul2)
}

/// Raven `void G2API_LoadGhoul2Models(CGhoul2Info_v &ghoul2, char *buffer)` —
/// forwards to `G2_LoadGhoul2Model` (`G2_misc.cpp:1841-1907`, `misc.rs`):
/// resizes `ghoul2` to the leading instance count (a no-op early return when
/// that count is `0`), then walks `buffer` rebuilding each instance's model
/// index/filename/surface/bone/bolt lists; a valid post-resize instance also
/// re-derives its model pointers via `G2_SetupModelPointers` (`:1873`), which
/// is why `host` is threaded here even though `G2API_LoadGhoul2Models` itself
/// touches no engine service directly.
///
/// `char *buffer` (read-only pointer walk, never written) becomes `&[u8]`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2477-2480`
pub fn g2api_load_ghoul2_models(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    buffer: &[u8],
) {
    // Raven: `G2_LoadGhoul2Model(ghoul2, buffer);` — forwards verbatim.
    crate::misc::g2_load_ghoul2_model(g2, host, ghoul2, buffer)
}

/// Raven `void G2API_FreeSaveBuffer(char *buffer) { Z_Free(buffer); }` — frees
/// the buffer `G2API_SaveGhoul2Models` `Z_Malloc`'d. The Rust `Vec<u8>`
/// `g2api_save_ghoul2_models` returns is already owned, so this is an
/// ownership-consuming drop — no `g2`/`host` thread, matching the sibling
/// pure-ownership default (`EngineHost::fs_free_file`'s `{}` default body,
/// `## Seam definition`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2482-2485`
pub fn g2api_free_save_buffer(buffer: Vec<u8>) {
    // Raven: `Z_Free(buffer);` — the owned `Vec<u8>` is dropped here,
    // matching the ownership-consuming free (module-doc note above).
    drop(buffer);
}

/// Raven `void G2API_LoadSaveCodeDestructGhoul2Info(CGhoul2Info_v &ghoul2)` —
/// clears any per-instance gore (`G2API_ClearSkinGore`, `_G2_GORE` on,
/// `:2493`, `api_gore.rs`), then runs `ghoul2`'s destructor so save-load can
/// overwrite the memory without orphaning the arena slot it owns.
///
/// The destructor call maps to `CGhoul2Info_v::Free` (its documented
/// destructor behavior, `shared/cghoul2_info_v.rs`), which needs `&mut
/// Ghoul2System` to release the arena slot — so `g2` is threaded through
/// (ruling 4/11) even though the Raven signature takes only `ghoul2`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2487-2496`
pub fn g2api_load_save_code_destruct_ghoul2_info(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
) {
    // Raven: `#ifdef _G2_GORE G2API_ClearSkinGore(ghoul2); #endif` — `_G2_GORE`
    // is ON in this build (G2SV-D5), so the call always runs.
    crate::api_gore::g2api_clear_skin_gore(g2, ghoul2);
    // Raven: `ghoul2.~CGhoul2Info_v();` — maps to `Free` (this file's
    // module-doc note), releasing the arena slot so save-load can overwrite
    // the memory without orphaning it.
    ghoul2.free(g2);
}

/// Raven `char *G2API_GetAnimFileName(CGhoul2Info *ghlInfo, char **filename)`
/// — write-on-success-only (`G2SV-D1` discriminator): `return qfalse` on
/// `G2_SetupModelPointers` failure **before** touching `filename`, else
/// forwards to `G2_GetAnimFileName(ghlInfo->mFileName, filename)`
/// (`G2_misc.cpp:356-367`, `misc.rs`), which itself only writes `*filename` on
/// its own success path. Both failure paths agree: `filename` untouched on
/// `None`.
///
/// Takes the single `CGhoul2Info *ghlInfo` directly (not the `CGhoul2Info_v`
/// wrapper), matching the `g2api_get_bone_anim`/`g2api_get_anim_range`
/// (`api_bones.rs`) precedent for this shape; `g2` is threaded per ruling 11
/// though this body never reaches `Ghoul2System` state, only `ghl_info` +
/// `host` (model registration/read via `G2_SetupModelPointers`,
/// `G2_GetAnimFileName`'s own `RE_RegisterModel`/`R_GetModelByHandle` chain).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1979-1986`
pub fn g2api_get_anim_file_name(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
) -> Option<String> {
    // `g2` is threaded per ruling 11 but never reached — only `ghl_info` and
    // `host` (see module comment above).
    let _ = g2;
    // Raven: `if (G2_SetupModelPointers(ghlInfo)) { return
    // G2_GetAnimFileName(ghlInfo->mFileName, filename); } return qfalse;`
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return None;
    }
    crate::misc::g2_get_anim_file_name(host, &ghl_info.file_name)
}

/// Raven `char *G2API_GetAnimFileNameIndex(qhandle_t modelIndex) { model_t
/// *mod_m = R_GetModelByHandle(modelIndex); return mod_m->mdxm->animName; }` —
/// unlike its sibling above, takes a bare `qhandle_t` (no `CGhoul2Info`/
/// `CGhoul2Info_v` at all) and has **no guard**: an invalid `modelIndex`
/// null-derefs `mod_m` in the oracle.
///
/// Divergence (§19, Raven UB site): `mod_m`/`mod_m->mdxm` null is undefined
/// behavior in the oracle; the port picks the one defined behavior and
/// returns `None` instead of dereferencing a null model.
///
/// `g2` is threaded per ruling 11 though unused (mirrors
/// `g2api_override_server_with_client_data`, `api_collision.rs`); `host`
/// serves the `model_mdxm` read (ruling 36, `G2SV-D15`) this body needs to
/// reach `animName` without naming the `mp_renderer`-owned `mdxm` struct here
/// (`G2SV-D5`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1962-1966`
pub fn g2api_get_anim_file_name_index(
    g2: &Ghoul2System,
    host: &mut impl EngineHost,
    model_index: qhandle_t,
) -> Option<String> {
    // `g2` is threaded per ruling 11 but never reached (module comment above).
    let _ = g2;

    let Some(mdxm) = host.model_mdxm(model_index) else {
        // Divergence (§19, Raven UB site, per this fn's doc comment above):
        // `mod_m`/`mod_m->mdxm` null is UB in the oracle; the port returns
        // `None` instead of dereferencing a null model.
        return None;
    };
    // `mdxmHeader_t->animName` off the live loader block — `MdxmView` owns the
    // byte offset (`G2SV-D5`, this crate never names the `mdxm*` types).
    Some(mdxm.anim_name())
}

/// Raven `char *G2API_GetGLAName(CGhoul2Info_v &ghoul2, int modelIndex)` —
/// write-on-success-only: `NULL` when `G2_SetupModelPointers(ghoul2)` fails or
/// `modelIndex` is out of range (`:2416`'s bounds check), else
/// `ghoul2[modelIndex].currentModel->mdxm->animName`.
///
/// `g2` is `&mut` because `G2_SetupModelPointers(CGhoul2Info_v&)` re-derives
/// model pointers on each arena-held instance (mutating through the handle);
/// `host` serves that same model-registration/read chain (matching
/// `g2api_get_gla_name`'s sibling `G2_SetupModelPointers` callers throughout
/// this crate).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2412-2426`
pub fn g2api_get_gla_name(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &CGhoul2Info_v,
    model_index: i32,
) -> Option<String> {
    // Raven: `if (G2_SetupModelPointers(ghoul2)) { ... } return NULL;`
    if !crate::misc::g2_setup_model_pointers_v(g2, host, ghoul2) {
        return None;
    }
    // Raven: `(int)&ghoul2 && (ghoul2.size() > modelIndex)` — the
    // address-of-reference half is never zero (same idiom as
    // `api_surfaces.rs`'s `g2api_set_surface_on_off`), collapsing to a plain
    // bounds check.
    if ghoul2.size(g2) <= model_index {
        return None;
    }

    // Raven: `assert(ghoul2[modelIndex].currentModel &&
    // ghoul2[modelIndex].currentModel->mdxm);` dropped (NDEBUG build,
    // module-doc convention, `api_models.rs`); the `return` below is the
    // sole live statement past it.
    let info = ghoul2.get(g2, model_index);

    // `mdxmHeader_t->animName` off the `currentModel->mdxm` block reached
    // through `ghlInfo->model`'s registered handle — `MdxmView` owns the byte
    // offset (`G2SV-D5`). Raven's `assert(currentModel && mdxm)` was dropped
    // above (NDEBUG); a null block is Raven UB (§19), so no guard is added here.
    let mdxm = host.model_mdxm(info.model).unwrap();
    Some(mdxm.anim_name())
}
