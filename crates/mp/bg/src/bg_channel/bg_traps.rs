//! `BgTraps` — the bg tier's outbound engine surface.
//!
//! The bg tier (`bg_pmove.c` et al.) cannot see `Engine`/`GameContext`
//! (bg < game). Raven bridged this with the `GAME_HARD_LINKED` `strap_*` shim
//! layer plus the `pmove_t` `trace`/`pointcontents` callbacks. This trait is the
//! Rust form of that surface: bg-visible signatures only (no `Engine`, no
//! `Args`). The game tier implements it over the `crate::trap` wrappers (holding
//! `&Engine`); `PmoveContext`/`BgState` carry a `&dyn BgTraps`.
//!
//! The bg modules currently live inside `mp_game` rather than their own crate,
//! so this trait lives here; the trait boundary — not a crate split — enforces
//! the tier.
#![allow(non_snake_case, clippy::too_many_arguments)]

use core::ffi::{c_char, c_int, c_void};

use crate::prelude::*;

/// The bg-reachable engine surface. Mirrors the `pmove_t` world-test callbacks
/// (`trace`/`pointcontents`) and the `strap_*`/`trap_*` calls bg code makes.
pub trait BgTraps {
    // --- pmove_t world-test callbacks (bg_public.h:484-485 semantics) ---

    /// Mirror of `pmove_t::trace` — `trap_Trace` against all linked entities.
    /// Source: `oracle/codemp/game/bg_public.h:484`
    fn trace(
        &self,
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passEntityNum: c_int,
        contentMask: c_int,
    );

    /// Mirror of `pmove_t::pointcontents` — `trap_PointContents`.
    /// Source: `oracle/codemp/game/bg_public.h:485`
    fn pointcontents(&self, point: *const vec3_t, passEntityNum: c_int) -> c_int;

    // --- filesystem (trap_FS_*; bg saber/anim/vehicle loaders) ---

    /// Raven `trap_FS_FOpenFile`. Source: `oracle/codemp/game/g_syscalls.c`
    fn fs_fopen(&self, qpath: &str, f: *mut fileHandle_t, mode: fsMode_t) -> c_int;
    /// Raven `trap_FS_Read`. Source: `oracle/codemp/game/g_syscalls.c`
    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t);
    /// Raven `trap_FS_Write`. Source: `oracle/codemp/game/g_syscalls.c`
    fn fs_write(&self, buffer: *const c_void, len: c_int, f: fileHandle_t);
    /// Raven `trap_FS_FCloseFile`. Source: `oracle/codemp/game/g_syscalls.c`
    fn fs_fclose(&self, f: fileHandle_t);
    /// Raven `trap_FS_GetFileList`. Source: `oracle/codemp/game/g_syscalls.c`
    fn fs_getfilelist(
        &self,
        path: &str,
        extension: &str,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int;

    /// Mirror of `trap_R_RegisterSkin` — the bg-visible surface bg saber-load
    /// code (`WP_SaberParseParms`'s `customSkin` field) needs to register a
    /// skin without holding `&Engine`.
    /// Source: `oracle/codemp/game/g_syscalls.c:1179-1182`
    fn r_register_skin(&self, name: &str) -> qhandle_t;

    /// Mirror of `trap_G2API_InitGhoul2Model` — the bg-visible surface
    /// `BG_ModelCache`'s QAGAME branch needs to precache a ghoul2 model
    /// without holding `&Engine`.
    /// Source: `oracle/codemp/game/g_syscalls.c:1223-1227`
    #[allow(clippy::too_many_arguments)]
    fn g2api_init_ghoul2_model(
        &self,
        ghoul2Ptr: *mut *mut c_void,
        fileName: &str,
        modelIndex: c_int,
        customSkin: qhandle_t,
        customShader: qhandle_t,
        modelFlags: c_int,
        lodBias: c_int,
    ) -> c_int;

    /// Mirror of `trap_G2API_CleanGhoul2Models` — bg-visible counterpart to
    /// `g2api_init_ghoul2_model` (`BG_ModelCache`'s QAGAME branch).
    /// Source: `oracle/codemp/game/g_syscalls.c:1303-1306`
    fn g2api_clean_ghoul2_models(&self, ghoul2Ptr: *mut *mut c_void);

    // --- ghoul2 straps (the 10 strap_G2API_* wrappers, g_strap.c) ---

    /// Raven `trap_G2API_AddBolt` — the bg-visible mirror of `crate::trap::G2API_AddBolt`
    /// (`G_G2_ADDBOLT` syscall), needed by bg vehicle-loader code (`AttachRidersGeneric`)
    /// that only has `&dyn BgTraps`, not `&Engine`.
    /// Source: `oracle/codemp/game/g_syscalls.c:1239-1242`
    fn g2api_add_bolt(&self, ghoul2: *mut c_void, modelIndex: c_int, boneName: &str) -> c_int;

    /// Raven `strap_G2API_GetBoltMatrix`. Source: `oracle/codemp/game/g_strap.c:6-10`
    fn g2api_get_bolt_matrix(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: *const vec3_t,
    ) -> qboolean;
    /// Raven `strap_G2API_GetBoltMatrix_NoReconstruct`. Source: `g_strap.c:12-16`
    fn g2api_get_bolt_matrix_no_reconstruct(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: *const vec3_t,
    ) -> qboolean;
    /// Raven `strap_G2API_GetBoltMatrix_NoRecNoRot`. Source: `g_strap.c:18-22`
    fn g2api_get_bolt_matrix_no_rec_no_rot(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: *const vec3_t,
    ) -> qboolean;
    /// Raven `strap_G2API_SetBoneAngles`. Source: `g_strap.c:24-29`
    fn g2api_set_bone_angles(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: &str,
        angles: *const vec3_t,
        flags: c_int,
        up: c_int,
        right: c_int,
        forward: c_int,
        modelList: *mut qhandle_t,
        blendTime: c_int,
        currentTime: c_int,
    ) -> qboolean;
    /// Raven `strap_G2API_SetBoneAnim`. Source: `g_strap.c:31-35`
    fn g2api_set_bone_anim(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: &str,
        startFrame: c_int,
        endFrame: c_int,
        flags: c_int,
        animSpeed: f32,
        currentTime: c_int,
        setFrame: f32,
        blendTime: c_int,
    ) -> qboolean;
    /// Raven `strap_G2API_GetBoneAnim`. Source: `g_strap.c:37-41`
    fn g2api_get_bone_anim(
        &self,
        ghoul2: *mut c_void,
        boneName: &str,
        currentTime: c_int,
        currentFrame: *mut f32,
        startFrame: *mut c_int,
        endFrame: *mut c_int,
        flags: *mut c_int,
        animSpeed: *mut f32,
        modelList: *mut c_int,
        modelIndex: c_int,
    ) -> qboolean;
    /// Raven `strap_G2API_SetRagDoll`. Source: `g_strap.c:43-46`
    fn g2api_set_rag_doll(&self, ghoul2: *mut c_void, params: *mut sharedRagDollParams_t);
    /// Raven `strap_G2API_AnimateG2Models`. Source: `g_strap.c:48-51`
    fn g2api_animate_g2_models(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    );
    /// Raven `strap_G2API_SetBoneIKState`. Source: `g_strap.c:53-56`
    // `boneName` is genuinely nullable: bg passes NULL to init/reset the IK
    // system on the instance (vs a named bone), so it crosses as `Option<&str>`.
    fn g2api_set_bone_ik_state(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        boneName: Option<&str>,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean;
    /// Raven `strap_G2API_IKMove`. Source: `g_strap.c:58-61`
    fn g2api_ik_move(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> qboolean;
    /// Raven `trap_G2API_GetSurfaceRenderStatus`. Source: `g_syscalls.c:1370`
    fn g2api_get_surface_render_status(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        surfaceName: &str,
    ) -> c_int;

    // --- effects / misc ---

    /// Raven `trap_FX_PlayEffectID`. Source: `oracle/codemp/game/g_syscalls.c`
    fn fx_play_effect_id(
        &self,
        fxID: c_int,
        org: *const vec3_t,
        fwd: *const vec3_t,
        vol: c_int,
        rad: c_int,
    );
    /// Raven `trap_SnapVector` — snap a vector to integer coords on the engine.
    /// Source: `oracle/codemp/game/g_syscalls.c`
    fn snap_vector(&self, v: *mut f32);
    /// Raven `trap_Cvar_Register`. Source: `oracle/codemp/game/g_syscalls.c`
    fn cvar_register(&self, cvar: *mut vmCvar_t, var_name: &str, value: &str, flags: c_int);

    // --- console (Com_Printf/Com_Error map to trap_Print/trap_Error) ---

    /// Raven `Com_Printf`: bg code calls it, and in the module it maps to
    /// `trap_Print` (`G_PRINT`) — mirroring the game-tier `Com_Printf` port.
    /// Source: `oracle/codemp/game/g_main.c:1219-1228`.
    fn com_printf(&self, msg: &str);

    /// Raven `Com_Error`: bg code calls it, and in the module it maps to
    /// `trap_Error` (`G_ERROR`). `error_level` is dropped at the seam, matching
    /// the game-tier `Com_Error` port's `G_Error("%s", text)` (which never
    /// forwards the level). Returns unit, as that port does — call-site control
    /// flow is unchanged.
    /// Source: `oracle/codemp/game/g_main.c:1208-1217`.
    fn com_error(&self, error_level: c_int, msg: &str);
}
