//! `G2API` models — the model-instance lifecycle surface: init/remove/clean/
//! copy/duplicate a `CGhoul2Info_v`'s Ghoul2 models, precache-register a model
//! file, and the per-instance lod-bias/skin/shader/flags setters.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_models.rs`, class "G2API
//! models"): Init/Remove/Clean/Copy/Duplicate Ghoul2 models, `PrecacheGhoul2Model`,
//! `SetLodBias`/`Skin`/`Shader`/`Flags`, `SetGhoul2ModelIndexes`,
//! `HaveWeGhoul2Models`, `Ghoul2Size`, `SkinlessModel` (`G2SV-D6`, 1:1 signatures).
//! Listed "Host-consuming" in the doc's per-file host-need survey:
//! "`RE_RegisterModel` loader-register → model accessors; `print`"
//! (`docs/subsystems/ghoul2-server.md:1359`).
//!
//! Every `G2API_*` entry keeps its 1:1 signature (`G2SV-D6`) and threads
//! `g2: &mut Ghoul2System` (ruling 4/11, state threaded not reached);
//! `host: &mut impl EngineHost` is added only where the body needs a host
//! service (model registration/read, or `print`/`error`).
//!
//! **Doc/oracle gap found while enumerating this class (reported to the
//! caller, not fixed here — out of this file's scope):** the frozen
//! `## Seam definition` names `api_models.rs` as consuming "`RE_RegisterModel`
//! loader-register → model accessors", but the quoted, BUILT 15-method
//! `EngineHost` trait (`crates/mp/host-interface/src/engine_host.rs`,
//! `G2SV-D15`) has no method that registers a model **by filename** and
//! returns a `qhandle_t` — `model_mdxm`/`model_mdxa` both take an *existing*
//! `qhandle_t` and only resolve the already-loaded block pointer. Raven's
//! `RE_RegisterModel`/`RE_RegisterServerModel` (the filename → `qhandle_t`
//! register call `G2API_PrecacheGhoul2Model` makes directly,
//! `G2_API.cpp:585-595`, and that `G2_TestModelPointers` makes on every
//! `G2API_InitGhoul2Model`, `:2606-2663`) has no corresponding `EngineHost`
//! method. `G2_TestModelPointers`'s registration branch is additionally gated
//! by `G2_ShouldRegisterServer()` (`:570-583`), which itself reads
//! `currentVM`/`gvm` (VM-slot identity) and the `com_cl_running` cvar plus
//! `Com_TheHunkMarkHasBeenMade()`/`ShaderHashTableExists()` (renderer/hunk
//! state) — none of which map onto any of the 15 frozen methods either. The
//! stubs below still take `host: &mut impl EngineHost` (the nearest frozen
//! parameter shape) so every other host-served line of these bodies
//! transcribes once the gap above is closed, but the registration/should-
//! register calls themselves have no host method to call yet.
//!
//! **This gap is now narrowed and load-bearing facts pinned down (still
//! reported, not fixed — no `mp_host_interface` edit is in this file's
//! scope):**
//! - `G2_TestModelPointers`'s `(com_dedicated && com_dedicated->integer) ||
//!   G2_ShouldRegisterServer()` gate (`G2_API.cpp:2616-2617`) reads a real,
//!   frozen host service: `com_dedicated` is `Cvar_Get("dedicated","2",
//!   CVAR_ROM)` under `-DDEDICATED` (`common.cpp:1290`) — read-only, always
//!   nonzero — so `host.cvar_integer("dedicated")` faithfully reproduces the
//!   left operand and, being truthy in this exact WinDed DEDICATED build,
//!   short-circuits the `||` exactly as the oracle does (never actually
//!   calling `G2_ShouldRegisterServer` from this call site in practice).
//! - `G2_ShouldRegisterServer`'s own `currentVM`/`gvm` check has no host
//!   equivalent, but every call site this crate reaches is inside a
//!   `VM_Call` to the game VM (`## Seam definition`'s `G2API_*` surface is
//!   the `SV_GameSystemCalls` switch target), so `currentVM == gvm` holds
//!   unconditionally here. Its `com_cl_running` operand IS a real cvar
//!   (`Cvar_Get("cl_running","0",CVAR_ROM)`, `common.cpp:1328`, only ever set
//!   nonzero by client startup code that never runs in `-DDEDICATED`) —
//!   `host.cvar_integer("cl_running")` reads it faithfully.
//! - What remains genuinely unresolvable without an `mp_host_interface`
//!   extension is the one filename→`qhandle_t` registration call itself
//!   (`RE_RegisterServerModel`/`RE_RegisterModel`) and, on the one
//!   theoretically-reachable-but-never-actually-taken branch of
//!   `G2_ShouldRegisterServer`, `Com_TheHunkMarkHasBeenMade`/
//!   `ShaderHashTableExists`. Both diverge via the frozen, real
//!   `host.error(...)` service (never invented) rather than fabricating a
//!   handle or a boolean the port cannot actually derive; every other line
//!   of these bodies transcribes fully today.
//! - `assert(...)` calls quoted from the oracle below are **not** ported as
//!   Rust `assert!`/`panic!`: this build defines `-DNDEBUG` (doc's Raven
//!   ground truth, top of `docs/subsystems/ghoul2-server.md`'s "Build
//!   config"), and outside the `Q3_VM` arm (`q_shared.h:70-88`) Raven's
//!   `assert` is the standard `<assert.h>` macro, which `NDEBUG` reduces to
//!   `((void)0)` — every plain `assert(...)` in this file's oracle bodies is
//!   therefore a genuine no-op in the actual shipped binary. Translating them
//!   to `assert!`/`panic!` would introduce panics the real oracle never
//!   raises; they are dropped (with a one-line citation at each site), while
//!   any statement that follows an assert in the same oracle line (e.g. a
//!   `return` beside a dropped `assert(0)`) is preserved, since it is a
//!   separate C statement that always ran regardless of NDEBUG.

use mp_host_interface::EngineHost;
use mp_qshared::shared::{errorParm_t, qhandle_t};

use crate::ghoul2_system::Ghoul2System;
use crate::mdx::mdxa::MdxaView;
use crate::mdx::mdxm::MdxmView;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

/// Raven `#define GHOUL2_NEWORIGIN 0x008` (`ghoul2_shared.h:232`) — the
/// `mFlags` bit `G2API_Set/GetGhoul2ModelFlags` preserve across a flags
/// overwrite. Duplicated locally (this crate has no shared flags-constants
/// module yet; `MAX_G2_MODELS`/`G2_INDEX_MASK` are already duplicated the
/// same way between `ghoul2_system.rs` and `info_array.rs`, per their own
/// doc comments) rather than invented as a new cross-file dependency.
const GHOUL2_NEWORIGIN: i32 = 0x008;

/// Duplicate a whole `CGhoul2Info` instance (Raven's plain struct-assignment
/// copy, `G2_API.cpp:2315`: `ghoul2To[modelTo] = ghoul2From[modelFrom]`).
///
/// Raven's default copy shares the raw `mBoneCache` pointer between source
/// and destination (a real aliasing quirk, not UB — porting-rules §F19 keeps
/// only genuine UB out of the shared fixtures); copying `bone_cache` by value
/// reproduces that aliasing faithfully rather than "fixing" it.
fn dup_cghoul2_info(src: &CGhoul2Info) -> CGhoul2Info {
    CGhoul2Info {
        slist: src.slist.clone(),
        bltlist: src.bltlist.clone(),
        blist: src.blist.clone(),
        modelindex: src.modelindex,
        custom_shader: src.custom_shader,
        custom_skin: src.custom_skin,
        model_bolt_link: src.model_bolt_link,
        surface_root: src.surface_root,
        lod_bias: src.lod_bias,
        new_origin: src.new_origin,
        gore_set_tag: src.gore_set_tag,
        model: src.model,
        file_name: src.file_name.clone(),
        anim_frame_default: src.anim_frame_default,
        skel_frame_num: src.skel_frame_num,
        mesh_frame_num: src.mesh_frame_num,
        flags: src.flags,
        transformed_verts_array: src.transformed_verts_array.clone(),
        bone_cache: src.bone_cache,
        skin: src.skin,
        valid: src.valid,
        current_model: src.current_model,
        current_model_size: src.current_model_size,
        anim_model: src.anim_model,
        current_anim_model_size: src.current_anim_model_size,
        a_header: src.a_header,
    }
}

/// Raven `qboolean G2_ShouldRegisterServer(void)` — "supreme hackery" gate
/// deciding whether a model registers through the server (`RE_RegisterServerModel`)
/// or client (`RE_RegisterModel`) path: true only when the currently-running VM
/// is the game VM and the client hasn't already marked the hunk with its own
/// asset load. Private helper — its only ghoul2-side callers are
/// `G2API_PrecacheGhoul2Model` (below) and `G2_TestModelPointers`/
/// `G2_SetupModelPointers` (the latter is `misc.rs`'s roster item; this doc
/// module owns the copy this file's own callers need per §F21 colocation).
///
/// See the module doc-comment gap note: `currentVM`/`gvm`/`com_cl_running`/
/// `Com_TheHunkMarkHasBeenMade`/`ShaderHashTableExists` have no `EngineHost`
/// method yet — this body reads what it CAN (`cl_running` is a real cvar) and
/// folds the rest to the fact established there: `currentVM == gvm` always
/// holds at every call site this crate reaches, so absent a `cl_running`
/// surprise this always returns `true`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:568-583`
pub(crate) fn g2_should_register_server(host: &mut impl EngineHost) -> bool {
    if host.cvar_integer("cl_running") != 0 {
        // `Com_TheHunkMarkHasBeenMade`/`ShaderHashTableExists` have no
        // `EngineHost` equivalent (module doc-comment gap note) and this arm
        // is unreachable in the DEDICATED build this crate targets
        // (`cl_running` is `CVAR_ROM` default `0`, only ever set by client
        // startup code that never runs here, `common.cpp:1328`) — but if it
        // somehow fired, silently guessing the answer would be worse than
        // stopping loudly.
        host.error(
            errorParm_t::ERR_DROP,
            "G2_ShouldRegisterServer: cl_running set in a DEDICATED build \u{2014} \
             Com_TheHunkMarkHasBeenMade/ShaderHashTableExists have no EngineHost service",
        );
    }
    true
}

/// Raven `RE_RegisterServerModel( fileName )` through the
/// `EngineHost::model_register` seam (the former ghoul2-server.md gap, closed
/// by user ruling 2026-07-12).
/// Source: `oracle/codemp/renderer/tr_model.cpp:588`
pub(crate) fn register_server_model(host: &mut impl EngineHost, file_name: &str) -> qhandle_t {
    host.model_register(file_name)
}

/// `RE_RegisterModel`'s client-path twin of [`register_server_model`]; same
/// gap, same divergence treatment. `context`/`cite` let each call site cite its
/// own oracle location in the `host.error` message.
pub(crate) fn register_model(
    host: &mut impl EngineHost,
    file_name: &str,
    context: &str,
    cite: &str,
) -> qhandle_t {
    host.error(
        errorParm_t::ERR_DROP,
        &format!(
            "{context}: EngineHost has no RE_RegisterModel(\"{file_name}\") equivalent yet \
             (docs/subsystems/ghoul2-server.md gap note, {cite})"
        ),
    )
}

/// Raven `qboolean G2_TestModelPointers(CGhoul2Info *ghlInfo)` — registers
/// `ghlInfo->mFileName` (server or client path per [`g2_should_register_server`]
/// / `com_dedicated->integer`), resolves the model/anim model/`mdxm`/`aHeader`
/// pointers off the returned handle, `Com_Error(ERR_DROP, ...)`s if a
/// previously-loaded model's size changed, and sets `mValid`. Private helper —
/// its sole caller is `G2API_InitGhoul2Model` below (`G2_API.cpp:650`); the
/// sibling `G2_SetupModelPointers` overloads (`misc.rs` roster item) do the
/// equivalent resolve for the already-initialized-model call sites elsewhere.
///
/// `com_dedicated->integer` is unconditionally `2` in this build
/// (`Cvar_Get("dedicated","2",CVAR_ROM)`, `common.cpp:1290`, under
/// `-DDEDICATED`) — read via the real `host.cvar_integer("dedicated")` — so
/// the `||` here short-circuits exactly as the oracle does and never actually
/// calls [`g2_should_register_server`] from this call site in practice.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2606-2663`
fn g2_test_model_pointers(ghl_info: &mut CGhoul2Info, host: &mut impl EngineHost) -> bool {
    ghl_info.valid = false;
    if ghl_info.modelindex != -1 {
        let dedicated = host.cvar_integer("dedicated") != 0 || g2_should_register_server(host);
        ghl_info.model = if dedicated {
            register_server_model(host, &ghl_info.file_name)
        } else {
            register_model(host, &ghl_info.file_name, "G2API models", "G2_API.cpp:593")
        };

        let mdxm = host.model_mdxm(ghl_info.model);
        ghl_info.current_model = mdxm;
        if !mdxm.is_null() {
            // SAFETY: `mdxm` non-null, `EngineHost::model_mdxm`'s contract
            // (`G2SV-D5`).
            let view = unsafe { MdxmView::from_block(mdxm) };
            let ofs_end = view.ofs_end();
            if ghl_info.current_model_size != 0 && ghl_info.current_model_size != ofs_end {
                host.error(
                    errorParm_t::ERR_DROP,
                    "Ghoul2 model was reloaded and has changed, map must be restarted.\n",
                );
            }
            ghl_info.current_model_size = ofs_end;

            let anim_index = view.anim_index();
            let a_header = host.model_mdxa(anim_index);
            ghl_info.anim_model = a_header;
            if !a_header.is_null() {
                // SAFETY: `a_header` non-null, same contract as above.
                let a_ofs_end = unsafe { MdxaView::from_block(a_header) }.ofs_end();
                if ghl_info.current_anim_model_size != 0
                    && ghl_info.current_anim_model_size != a_ofs_end
                {
                    host.error(
                        errorParm_t::ERR_DROP,
                        "Ghoul2 model was reloaded and has changed, map must be restarted.\n",
                    );
                }
                ghl_info.current_anim_model_size = a_ofs_end;
                ghl_info.valid = true;
            }
        }
    }
    if !ghl_info.valid {
        ghl_info.current_model = core::ptr::null();
        ghl_info.current_model_size = 0;
        ghl_info.anim_model = core::ptr::null();
        ghl_info.current_anim_model_size = 0;
        ghl_info.a_header = core::ptr::null();
    }
    ghl_info.valid
}

/// Raven `qhandle_t G2API_PrecacheGhoul2Model(const char *fileName)` — registers
/// `fileName` through the server or client renderer-model path depending on
/// [`g2_should_register_server`].
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:585-595`
pub fn g2api_precache_ghoul2_model(host: &mut impl EngineHost, file_name: &str) -> qhandle_t {
    if g2_should_register_server(host) {
        register_server_model(host, file_name)
    } else {
        register_model(host, file_name, "G2API models", "G2_API.cpp:593")
    }
}

/// Raven `int G2API_InitGhoul2Model(CGhoul2Info_v **ghoul2Ptr, const char
/// *fileName, int modelIndex, qhandle_t customSkin, qhandle_t customShader,
/// int modelFlags, int lodBias)` — initializes the first free model slot (or
/// appends one) with `fileName`, validates it via [`g2_test_model_pointers`],
/// and on success seeds the bone/bolt lists and skin/shader/lod-bias/flags;
/// returns the model's index (or `-1` on an empty `fileName`, asserted in
/// Raven). `ghoul2Ptr`'s allocate-if-null branch (`:614-629`) is not modeled —
/// the caller always holds an already-valid `ghoul2` handle (matching this
/// doc's `## Seam definition`, which freezes this exact signature).
///
/// `modelIndex` is a genuinely unread parameter in the oracle body too (kept
/// for 1:1 signature fidelity, porting-rules §A2 — this file's local `model`
/// loop variable is a different value entirely).
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:601-668`
#[allow(clippy::too_many_arguments)]
pub fn g2api_init_ghoul2_model(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    file_name: &str,
    model_index: i32,
    custom_skin: qhandle_t,
    custom_shader: qhandle_t,
    model_flags: i32,
    lod_bias: i32,
) -> i32 {
    // `modelIndex`/`modelFlags` are both genuinely unread by the oracle body
    // (G2_API.cpp:601-668: `mFlags` is unconditionally set to `0`, never to
    // `modelFlags`) — kept as parameters for 1:1 signature fidelity (§A2).
    let _ = (model_index, model_flags);

    // assert(0) dropped (NDEBUG build, module gap note); the `return -1`
    // beside it is a separate statement and still fires.
    if file_name.is_empty() {
        return -1;
    }

    // find a free spot in the list (mModelindex == -1), else append a fresh one.
    let size = ghoul2.size(g2);
    let mut model = 0;
    while model < size {
        if ghoul2.get(g2, model).modelindex == -1 {
            *ghoul2.get_mut(g2, model) = CGhoul2Info::default();
            break;
        }
        model += 1;
    }
    if model == size {
        // assert(ghoul2.size() < 4) dropped (NDEBUG build).
        ghoul2.push_back(g2, CGhoul2Info::default());
    }

    {
        let info = ghoul2.get_mut(g2, model);
        info.file_name = file_name.to_string();
        info.modelindex = model;
    }

    let valid = g2_test_model_pointers(ghoul2.get_mut(g2, model), host);
    let info = ghoul2.get_mut(g2, model);
    if !valid {
        info.file_name.clear();
        info.modelindex = -1;
    } else {
        crate::bones::g2_init_bone_list(&mut info.blist);
        crate::bolts::g2_init_bolt_list(&mut info.bltlist);
        info.custom_shader = custom_shader;
        info.custom_skin = custom_skin;
        info.lod_bias = lod_bias;
        info.anim_frame_default = 0;
        info.flags = 0; // mFlags = 0 unconditionally (G2_API.cpp:663) — `modelFlags` is unread.
        info.model_bolt_link = -1;
    }

    ghoul2.get(g2, model).modelindex
}

/// Raven `void G2API_CleanGhoul2Models(CGhoul2Info_v **ghoul2Ptr)` — the only
/// function that reads entity states directly (Raven comment, `:495`): clears
/// any attached gore (`G2API_ClearSkinGore`, `_G2_GORE` on) then frees the
/// whole handle (`ghoul2.~CGhoul2Info_v()`, forwarding to `Free`/
/// `Ghoul2System::delete`, `G2SV-D13`(a)) and nulls the caller's pointer. The
/// `#if 0`-disabled refentity crash-diagnostic block (`:502-542`) and the
/// `_FULL_G2_LEAK_CHECKING` alloc-tracking (`:550-561`) are dropped, no parity
/// surface (§F20).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:496-566`
pub fn g2api_clean_ghoul2_models(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v) {
    crate::api_gore::g2api_clear_skin_gore(g2, ghoul2);
    // `ghoul2.~CGhoul2Info_v()` forwards to `Free` (cghoul2_info_v.rs doc
    // comment); the `delete *ghoul2Ptr; *ghoul2Ptr = NULL;` half has no
    // counterpart once `CGhoul2Info_v**` collapses to `&mut CGhoul2Info_v`
    // (mechanical, matching `g2api_remove_ghoul2_model`'s own collapse).
    ghoul2.free(g2);
}

/// Raven `qboolean G2API_SetLodBias(CGhoul2Info *ghlInfo, int lodBias)` —
/// `qfalse` on a null `ghlInfo`, else sets `mLodBias` and returns `qtrue`.
///
/// The null check collapses away: this seam's `ghlInfo` is `&mut CGhoul2Info`,
/// never null, so the `qtrue` arm is the only one reachable (mechanical,
/// matching every other single-instance setter in this file).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:670-678`
pub fn g2api_set_lod_bias(
    g2: &mut Ghoul2System,
    ghl_info: &mut CGhoul2Info,
    lod_bias: i32,
) -> bool {
    let _ = g2; // threaded per ruling 11/4; this setter touches no subsystem state.
    ghl_info.lod_bias = lod_bias;
    true
}

/// Raven `qboolean G2API_SetSkin(CGhoul2Info *ghlInfo, qhandle_t customSkin,
/// qhandle_t renderSkin)` — sets `mCustomSkin`; when `renderSkin` is non-null
/// also runs `G2_SetSurfaceOnOffFromSkin` (`G2_surfaces.cpp:201`, `surfaces.rs`
/// roster item) to set each surface on/off per the skin file, which itself
/// reads the skin via `R_GetSkinByHandle` — served by
/// `EngineHost::skin_surfaces` (user ruling 2026-07-12, server skins
/// name-pool; formerly a frozen-trait gap).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:681-693`
pub fn g2api_set_skin(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    custom_skin: qhandle_t,
    render_skin: qhandle_t,
) -> bool {
    let _ = g2;
    ghl_info.custom_skin = custom_skin;
    if render_skin != 0 {
        crate::surfaces::g2_set_surface_on_off_from_skin(host, ghl_info, render_skin);
    }
    true
}

/// Raven `qboolean G2API_SetShader(CGhoul2Info *ghlInfo, qhandle_t
/// customShader)` — sets `mCustomShader`; `qfalse` on a null `ghlInfo`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:696-704`
pub fn g2api_set_shader(
    g2: &mut Ghoul2System,
    ghl_info: &mut CGhoul2Info,
    custom_shader: qhandle_t,
) -> bool {
    let _ = g2;
    ghl_info.custom_shader = custom_shader;
    true
}

/// Raven `qboolean G2API_HasGhoul2ModelOnIndex(CGhoul2Info_v **ghlRemove,
/// const int modelIndex)` — `qfalse` when the handle's vector is empty, too
/// short for `modelIndex`, or that slot's `mModelindex == -1`; else `qtrue`.
/// Pure read — no mutation of `g2` or `ghoul2`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:783-793`
pub fn g2api_has_ghoul2_model_on_index(
    g2: &Ghoul2System,
    ghoul2: &CGhoul2Info_v,
    model_index: i32,
) -> bool {
    let size = ghoul2.size(g2);
    if size == 0 || size <= model_index || ghoul2.get(g2, model_index).modelindex == -1 {
        return false;
    }
    true
}

/// Trim trailing `-1` (free) model slots off the back of `ghoul2` — the shared
/// tail of [`g2api_remove_ghoul2_model`]/[`g2api_remove_ghoul2_models`].
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:795-875,877-958`
fn trim_trailing_free_slots(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v) {
    let size_now = ghoul2.size(g2);
    let mut new_size = size_now;
    let mut i = size_now - 1;
    while i > -1 {
        if ghoul2.get(g2, i).modelindex == -1 {
            new_size = i;
        } else {
            break;
        }
        i -= 1;
    }
    if new_size != size_now {
        ghoul2.resize(g2, new_size);
    }
}

/// Raven `qboolean G2API_RemoveGhoul2Model(CGhoul2Info_v **ghlRemove, const
/// int modelIndex)` — sanity-asserts the index names a live model, then frees
/// its gore set (`DeleteGoreSet`, `_G2_GORE` on, `G2_misc.cpp:153`) and bone
/// cache (`RemoveBoneCache`, `Ghoul2System.bone_caches`), clears its lists, and
/// marks it `-1`; trims trailing `-1` slots off the back and, if the vector
/// empties out entirely, frees the whole handle (`delete *ghlRemove`,
/// mirrored by `Ghoul2System::delete` per `G2SV-D13`(a)).
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:795-875`
pub fn g2api_remove_ghoul2_model(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
) -> bool {
    let size = ghoul2.size(g2);
    if size == 0 || size <= model_index || ghoul2.get(g2, model_index).modelindex == -1 {
        // assert(0) dropped (NDEBUG build); the `return qfalse` beside it is
        // a separate statement and still fires.
        return false;
    }

    // _G2_GORE on: clean up any gore attached to this model.
    let gore_tag = ghoul2.get(g2, model_index).gore_set_tag;
    if gore_tag != 0 {
        g2.gore.delete_gore_set(gore_tag);
        ghoul2.get_mut(g2, model_index).gore_set_tag = 0;
    }

    if let Some(id) = ghoul2.get_mut(g2, model_index).bone_cache.take() {
        crate::render::bone_cache::remove_bone_cache(g2, id);
    }

    {
        let info = ghoul2.get_mut(g2, model_index);
        info.blist.clear();
        info.bltlist.clear();
        info.slist.clear();
        info.modelindex = -1;
    }

    // trim trailing -1 slots off the back.
    trim_trailing_free_slots(g2, ghoul2);

    // if we are not using any space, just free the ghoul2 vector entirely.
    if ghoul2.size(g2) == 0 {
        ghoul2.free(g2);
    }

    true
}

/// Raven `qboolean G2API_RemoveGhoul2Models(CGhoul2Info_v **ghlRemove)` —
/// "remove 'em ALL!": `qfalse` if the handle's vector is already empty, else
/// frees every live model's gore set/bone cache, clears its lists and marks it
/// `-1`, trims trailing `-1` slots, and frees the whole handle if it empties
/// out (same trailing-trim/whole-handle-free shape as
/// [`g2api_remove_ghoul2_model`], looped over every model instead of one).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:877-958`
pub fn g2api_remove_ghoul2_models(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v) -> bool {
    if ghoul2.size(g2) == 0 {
        return false;
    }

    let mut model_index = 0;
    while model_index < ghoul2.size(g2) {
        if ghoul2.get(g2, model_index).modelindex == -1 {
            model_index += 1;
            continue;
        }

        // _G2_GORE on: clean up any gore attached to this model.
        let gore_tag = ghoul2.get(g2, model_index).gore_set_tag;
        if gore_tag != 0 {
            g2.gore.delete_gore_set(gore_tag);
            ghoul2.get_mut(g2, model_index).gore_set_tag = 0;
        }

        if let Some(id) = ghoul2.get_mut(g2, model_index).bone_cache.take() {
            crate::render::bone_cache::remove_bone_cache(g2, id);
        }

        {
            let info = ghoul2.get_mut(g2, model_index);
            info.blist.clear();
            info.bltlist.clear();
            info.slist.clear();
            info.modelindex = -1;
        }

        model_index += 1;
    }

    // trim trailing -1 slots off the back.
    trim_trailing_free_slots(g2, ghoul2);

    if ghoul2.size(g2) == 0 {
        ghoul2.free(g2);
    }

    true
}

/// Raven `qboolean G2API_HaveWeGhoul2Models(CGhoul2Info_v &ghoul2)` — `qtrue`
/// iff any model instance has `mModelindex != -1`. Pure read.
///
/// Raven's `if ((int)&ghoul2)` address-of-reference check is always true
/// (dropped, mechanical — a reference is never null in valid C++; the same
/// boilerplate several other `G2API_*` functions in this file also discard).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1920-1934`
pub fn g2api_have_we_ghoul2_models(g2: &Ghoul2System, ghoul2: &CGhoul2Info_v) -> bool {
    let size = ghoul2.size(g2);
    (0..size).any(|i| ghoul2.get(g2, i).modelindex != -1)
}

/// Raven `void G2API_SetGhoul2ModelIndexes(CGhoul2Info_v &ghoul2, qhandle_t
/// *modelList, qhandle_t *skinList)` — unconditional `return;` before any of
/// the `#if 0`-disabled body runs (`:1939-1959`): a **compiled no-op** (§C10
/// dead-body fold), not a §20 drop — it is still a live `SV_GameSystemCalls`
/// switch target (`sv_game.cpp:1328`, `G_G2_SETMODELINDEXES`), so the 1:1
/// signature is kept and reached, but the body genuinely does nothing.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1937-1960`
pub fn g2api_set_ghoul2_model_indexes(
    g2: &mut Ghoul2System,
    ghoul2: &mut CGhoul2Info_v,
    model_list: &[qhandle_t],
    skin_list: &[qhandle_t],
) {
    let _ = (g2, ghoul2, model_list, skin_list);
}

/// Raven `qboolean G2API_SetGhoul2ModelFlags(CGhoul2Info *ghlInfo, const int
/// flags)` — on `G2_SetupModelPointers` success, masks `mFlags` down to
/// `GHOUL2_NEWORIGIN` and ORs in `flags`; `qfalse` on setup failure.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2175-2184`
pub fn g2api_set_ghoul2_model_flags(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    flags: i32,
) -> bool {
    let _ = g2; // threaded per ruling 11; `g2_setup_model_pointers`'s single-instance overload doesn't need it.
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        ghl_info.flags &= GHOUL2_NEWORIGIN;
        ghl_info.flags |= flags;
        true
    } else {
        false
    }
}

/// Raven `int G2API_GetGhoul2ModelFlags(CGhoul2Info *ghlInfo)` — on
/// `G2_SetupModelPointers` success, returns `mFlags & ~GHOUL2_NEWORIGIN`; `0`
/// on setup failure (write-on-success-only shape, but the return here is
/// already a plain value per §C7 — no out-param to classify).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2186-2193`
pub fn g2api_get_ghoul2_model_flags(
    g2: &Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
) -> i32 {
    let _ = g2;
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        ghl_info.flags & !GHOUL2_NEWORIGIN
    } else {
        0
    }
}

/// Raven `int G2API_CopyGhoul2Instance(CGhoul2Info_v &g2From, CGhoul2Info_v
/// &g2To, int modelIndex)` — asserts `modelIndex == -1` (bolted-part-subset
/// copying is unsupported), then if `g2From.IsValid()` runs
/// `g2To.DeepCopy(g2From)` and, per live model (`_G2_GORE` on), bumps the
/// `mRefCount` of any gore set the copy now shares (`FindGoreSet`,
/// `G2_misc.cpp:127`). Always returns `-1`. The `_DEBUG` double-copy-onto-
/// valid-instance asserts (`:2247-2256`) are debug-only, dropped (§F20).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2239-2280`
pub fn g2api_copy_ghoul2_instance(
    g2: &mut Ghoul2System,
    g2_from: &mut CGhoul2Info_v,
    g2_to: &mut CGhoul2Info_v,
    model_index: i32,
) -> i32 {
    // Raven's `assert(modelIndex==-1)` is a no-op in this NDEBUG build (see
    // file gap note); the sole in-tree callers already pass -1
    // (`g2api_duplicate_ghoul2_instance` below, `sv_game.cpp` traps), so this
    // is not modeled as a runtime check.
    let _ = model_index;

    if g2_from.is_valid(g2) {
        g2_to.deep_copy(g2, g2_from);

        // _G2_GORE on: bump the refcount of any gore set the copy now shares.
        let size = g2_to.size(g2);
        let tags: Vec<i32> = (0..size)
            .map(|i| g2_to.get(g2, i).gore_set_tag)
            .filter(|&tag| tag != 0)
            .collect();
        for tag in tags {
            if let Some(set) = g2.gore.find_gore_set(tag) {
                set.ref_count += 1;
            }
            // Raven's `assert(gore)` is a no-op here too (NDEBUG); a miss
            // just skips the bump, matching the assert-stripped control flow.
        }
    }

    -1
}

/// Raven `void G2API_CopySpecificG2Model(CGhoul2Info_v &ghoul2From, int
/// modelFrom, CGhoul2Info_v &ghoul2To, int modelTo)` — if `ghoul2From` has a
/// model at `modelFrom`, resizes `ghoul2To` to fit `modelTo` if needed, frees
/// any bone cache already at the destination slot (`RemoveBoneCache`,
/// `Ghoul2System.bone_caches`), then overwrites it with the source model
/// (plain struct copy). The dead `forceReconstruct`/`mSkelFrameNum` reset under
/// `#if 0` (`:2284-2286,2301-2304,2317-2324`) is dropped (§F20, never compiled).
///
/// Raven's `((int)&ghoul2From) && ((int)&ghoul2To)` address-of-reference
/// check is always true (dropped, mechanical — see `g2api_have_we_ghoul2_models`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2282-2327`
pub fn g2api_copy_specific_g2_model(
    g2: &mut Ghoul2System,
    ghoul2_from: &mut CGhoul2Info_v,
    model_from: i32,
    ghoul2_to: &mut CGhoul2Info_v,
    model_to: i32,
) {
    if ghoul2_from.size(g2) > model_from {
        if ghoul2_to.size(g2) <= model_to {
            // assert(modelTo < 5) dropped (NDEBUG build).
            ghoul2_to.resize(g2, model_to + 1);
        }

        if ghoul2_to.is_valid(g2) && ghoul2_to.size(g2) >= model_to {
            // remove the bonecache before we stomp over this instance.
            if let Some(id) = ghoul2_to.get_mut(g2, model_to).bone_cache.take() {
                crate::render::bone_cache::remove_bone_cache(g2, id);
            }
        }

        // do the copy (plain struct assignment, `G2_API.cpp:2315`).
        let copied = dup_cghoul2_info(ghoul2_from.get(g2, model_from));
        *ghoul2_to.get_mut(g2, model_to) = copied;
    }
}

/// Raven `void G2API_DuplicateGhoul2Instance(CGhoul2Info_v &g2From,
/// CGhoul2Info_v **g2To)` — "automatically copy everything about this model,
/// and make a new one if necessary": asserts `*g2To` is null going in
/// (Raven `assert(0); return;` on a non-null destination — a bad-caller path,
/// not a value this fn ever chooses to overwrite), allocates a fresh
/// destination handle, and forwards into
/// [`g2api_copy_ghoul2_instance`]`(g2From, *g2To, -1)`. `g2To`'s
/// allocate-if-null out-param collapses to `&mut CGhoul2Info_v` per the same
/// mechanical `CGhoul2Info_v **` → `&mut CGhoul2Info_v` rule
/// [`g2api_init_ghoul2_model`] and [`g2api_remove_ghoul2_model`] already apply
/// (both doc-frozen), not a fresh judgment call here. The
/// `_FULL_G2_LEAK_CHECKING` alloc tracking (`:2341-2352`) is dropped (§F20).
///
/// Raven's `assert(0)` on the bad-caller path is a no-op in this NDEBUG
/// build (file gap note); the `return` beside it is a separate statement and
/// still fires — modeled here as an `is_valid` check standing in for "is
/// `*g2To` non-null" (a fresh, unallocated `CGhoul2Info_v` is never valid).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2330-2358`
pub fn g2api_duplicate_ghoul2_instance(
    g2: &mut Ghoul2System,
    g2_from: &mut CGhoul2Info_v,
    g2_to: &mut CGhoul2Info_v,
) {
    if g2_to.is_valid(g2) {
        return;
    }
    let _ = g2api_copy_ghoul2_instance(g2, g2_from, g2_to, -1);
}

/// Raven `qboolean G2API_SkinlessModel(CGhoul2Info *g2)` — "see if surfs have
/// any shader info": on `G2_SetupModelPointers` success, walks the model's
/// `mdxm` surface-hierarchy table and returns `qfalse` at the first surface
/// with a non-empty shader name; `qtrue` if none is found (or setup fails).
/// Reads model memory (`mod->mdxm`), served by `EngineHost::model_mdxm`
/// (ruling 36) once `G2_SetupModelPointers`/registration resolve the handle —
/// see the module doc-comment's `RE_RegisterModel` gap note.
///
/// Raven: for each surface entry (`MdxmView::hierarchy_iter`), a non-empty
/// `shader` name means "found a surface with a shader, ok" (`qfalse` = not
/// skinless); reaching the end with none found is `qtrue`. `surf->shader` is a
/// `char[64]` array, never null itself, so Raven's `if (surf->shader &&
/// surf->shader[0])` collapses to the first-byte check alone.
///
/// Re-derives the `mdxm` block via `host.model_mdxm(ghl_info.model)` directly
/// rather than trusting `ghl_info.current_model`'s stored value: that field's
/// exact representation is `misc.rs`'s `g2_setup_model_pointers` sibling's own
/// internal choice (only its null-ness is an externally observable contract,
/// per its own doc comment), so this function stays self-contained instead of
/// depending on that convention.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2499-2527`
pub fn g2api_skinless_model(
    g2: &Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
) -> bool {
    let _ = g2; // threaded per ruling 11; unused by this particular body.
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        let mdxm = host.model_mdxm(ghl_info.model);
        if !mdxm.is_null() {
            // SAFETY: `mdxm` non-null, contract per `EngineHost::model_mdxm`.
            return unsafe { MdxmView::from_block(mdxm) }
                .hierarchy_iter()
                .all(|s| s.shader_first_byte() == 0);
        }
    }
    // found nothing.
    true
}

/// Raven `int G2API_Ghoul2Size ( CGhoul2Info_v &ghoul2)` — `ghoul2.size()`,
/// `_G2_GORE` on (this fn lives in the `_G2_GORE` block alongside the gore
/// surface, `G2_API.cpp:2530-2567`, but does no gore work itself). Pure read.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2563-2566`
pub fn g2api_ghoul2_size(g2: &Ghoul2System, ghoul2: &CGhoul2Info_v) -> i32 {
    ghoul2.size(g2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::bone_info_t::boneInfo_t;
    use mp_host_interface::mock::MockHost;

    #[test]
    fn dup_cghoul2_info_duplicates_lists_independently() {
        let mut src = CGhoul2Info::default();
        src.modelindex = 3;
        src.file_name = "models/x.glm".to_string();
        src.blist.push(boneInfo_t {
            boneNumber: 7,
            ..unsafe { core::mem::zeroed() }
        });
        src.gore_set_tag = 99;

        let mut copy = dup_cghoul2_info(&src);
        assert_eq!(copy.modelindex, 3);
        assert_eq!(copy.file_name, "models/x.glm");
        assert_eq!(copy.blist.len(), 1);
        assert_eq!(copy.blist[0].boneNumber, 7);
        assert_eq!(copy.gore_set_tag, 99);

        // Independent storage: mutating the copy must not touch `src`.
        copy.blist[0].boneNumber = 55;
        copy.file_name.push('!');
        assert_eq!(src.blist[0].boneNumber, 7);
        assert_eq!(src.file_name, "models/x.glm");
    }

    #[test]
    fn g2_should_register_server_is_true_when_no_client_is_running() {
        let mut host = MockHost::new();
        // `cl_running` unregistered reads 0, matching a DEDICATED build
        // (common.cpp:1328) — `currentVM == gvm` is assumed to hold (module
        // gap note), so this returns true.
        assert!(g2_should_register_server(&mut host));
    }

    #[test]
    fn g2_should_register_server_errors_when_cl_running_is_set() {
        let mut host = MockHost::new();
        host.set_cvar("cl_running", "1");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            g2_should_register_server(&mut host)
        }));
        assert!(result.is_err());
        assert_eq!(host.errors.len(), 1);
    }

    #[test]
    fn register_server_model_routes_through_the_host_seam() {
        // The former gap divergence (host.error) closed 2026-07-12:
        // registration flows through EngineHost::model_register and returns
        // the resolved handle (MockHost: 1-based, dedup by name).
        let mut host = MockHost::new();
        assert_eq!(register_server_model(&mut host, "models/x.glm"), 1);
        assert_eq!(register_server_model(&mut host, "models/y.glm"), 2);
        assert_eq!(register_server_model(&mut host, "models/x.glm"), 1);
        assert_eq!(host.model_registers, ["models/x.glm", "models/y.glm"]);
    }

    #[test]
    fn g2api_set_lod_bias_always_succeeds() {
        let mut g2 = Ghoul2System::default();
        let mut info = CGhoul2Info::default();
        assert!(g2api_set_lod_bias(&mut g2, &mut info, 3));
        assert_eq!(info.lod_bias, 3);
    }

    #[test]
    fn g2api_set_shader_always_succeeds() {
        let mut g2 = Ghoul2System::default();
        let mut info = CGhoul2Info::default();
        assert!(g2api_set_shader(&mut g2, &mut info, 42));
        assert_eq!(info.custom_shader, 42);
    }

    #[test]
    fn g2api_set_ghoul2_model_indexes_is_a_pure_no_op() {
        let mut g2 = Ghoul2System::default();
        let mut ghoul2 = CGhoul2Info_v { mItem: 0 };
        let model_list = [1i32, 2, 3];
        let skin_list = [4i32, 5, 6];
        // Must not panic and must not require any arena state at all (the
        // handle is deliberately left unallocated/invalid).
        g2api_set_ghoul2_model_indexes(&mut g2, &mut ghoul2, &model_list, &skin_list);
    }
}
