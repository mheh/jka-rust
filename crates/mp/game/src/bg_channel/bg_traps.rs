//! `BgTraps` — the bg tier's outbound engine surface (pass-3 ruling 13).
//!
//! The bg tier (`bg_pmove.c` et al.) cannot see `Engine`/`GameContext`
//! (bg < game). Raven bridged this with the `GAME_HARD_LINKED` `strap_*` shim
//! layer plus the `pmove_t` `trace`/`pointcontents` callbacks. This trait is the
//! Rust form of that surface: bg-visible signatures only (no `Engine`, no
//! `Args`). The game tier implements it over the `crate::trap` wrappers (holding
//! `&Engine`); `PmoveContext`/`BgState` carry a `&dyn BgTraps`.
//!
//! Ruling 19 keeps the bg modules inside `mp_game` for pass 3, so this trait
//! lives here; the trait boundary — not a crate split — enforces the tier.
#![allow(non_snake_case, clippy::too_many_arguments)]

use core::ffi::{c_char, c_int, c_void};

use crate::prelude::*;

/// The bg-reachable engine surface. Mirrors the `pmove_t` world-test callbacks
/// (`trace`/`pointcontents`) and the `strap_*`/`trap_*` calls bg code makes.
pub trait BgTraps {
    // --- pmove_t world-test callbacks (bg_public.h:484-485 semantics) ---

    /// Mirror of `pmove_t::trace` — `trap_Trace` against all linked entities.
    /// Source: `oracle/oracle/codemp/game/bg_public.h:484`
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
    /// Source: `oracle/oracle/codemp/game/bg_public.h:485`
    fn pointcontents(&self, point: *const vec3_t, passEntityNum: c_int) -> c_int;

    // --- filesystem (trap_FS_*; bg saber/anim/vehicle loaders) ---

    /// Raven `trap_FS_FOpenFile`. Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn fs_fopen(&self, qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> c_int;
    /// Raven `trap_FS_Read`. Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t);
    /// Raven `trap_FS_Write`. Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn fs_write(&self, buffer: *const c_void, len: c_int, f: fileHandle_t);
    /// Raven `trap_FS_FCloseFile`. Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn fs_fclose(&self, f: fileHandle_t);
    /// Raven `trap_FS_GetFileList`. Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn fs_getfilelist(
        &self,
        path: *const c_char,
        extension: *const c_char,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int;

    // --- ghoul2 straps (the 10 strap_G2API_* wrappers, g_strap.c) ---

    /// Raven `trap_G2API_AddBolt` — the bg-visible mirror of `crate::trap::G2API_AddBolt`
    /// (`G_G2_ADDBOLT` syscall), needed by bg vehicle-loader code (`AttachRidersGeneric`)
    /// that only has `&dyn BgTraps`, not `&Engine`.
    /// Source: `oracle/oracle/codemp/game/g_syscalls.c:1239-1242`
    fn g2api_add_bolt(&self, ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int;

    /// Raven `strap_G2API_GetBoltMatrix`. Source: `oracle/oracle/codemp/game/g_strap.c:6-10`
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
        boneName: *const c_char,
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
        boneName: *const c_char,
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
        boneName: *const c_char,
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
    fn g2api_set_bone_ik_state(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        boneName: *const c_char,
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

    // --- effects / misc ---

    /// Raven `trap_FX_PlayEffectID`. Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn fx_play_effect_id(
        &self,
        fxID: c_int,
        org: *const vec3_t,
        fwd: *const vec3_t,
        vol: c_int,
        rad: c_int,
    );
    /// Raven `trap_SnapVector` — snap a vector to integer coords on the engine.
    /// Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn snap_vector(&self, v: *mut f32);
    /// Raven `trap_Cvar_Register`. Source: `oracle/oracle/codemp/game/g_syscalls.c`
    fn cvar_register(
        &self,
        cvar: *mut vmCvar_t,
        var_name: *const c_char,
        value: *const c_char,
        flags: c_int,
    );
}
