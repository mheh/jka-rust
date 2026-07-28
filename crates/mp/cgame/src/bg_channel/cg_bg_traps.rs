//! `CgBgTraps` — the cgame-tier `BgTraps` implementation.
//!
//! `mp_bg::bg_channel::BgTraps` is the bg tier's outbound engine surface
//! (`crates/mp/bg/src/bg_channel/bg_traps.rs`); the game tier's implementor is
//! [`mp_game`'s `GameBgTraps`](../../../game/src/bg_channel/game_impl.rs) and
//! the ui's is
//! [`mp_ui`'s `UiBgTraps`](../../../ui/src/bg_channel/ui_bg_traps.rs).
//!
//! cgame is the near-mirror of the game tier here, not of ui. `cg_syscalls.c`
//! carries a real wrapper for every bg-facing trap in the trait — the whole
//! `trap_FS_*` set, `trap_Cvar_Register`, `trap_R_RegisterSkin`, all thirteen
//! `trap_G2API_*`, `trap_SnapVector`, `trap_FX_PlayEffectID`, `trap_Print` and
//! `trap_Error` — so 25 of the 27 methods are straight delegations through
//! [`crate::trap`], the same shape `mp_game` uses. `trap_FX_PlayEffectID` is in
//! fact the *cgame-only* arm: the game tier's implementor panics on it because
//! `bg_slidemove.c` declares and calls it under `#ifndef QAGAME`
//! (`bg_slidemove.c:37-39,550`).
//!
//! The two exceptions are `trace` and `pointcontents`. cgame does not answer
//! those with a syscall at all — `CG_PmoveClientThink` binds
//! `cg_pmove.trace = CG_Trace` / `cg_pmove.pointcontents = CG_PointContents`
//! (`cg_predict.c:1009-1010`, and again for `cg_vehPmove` at 1385-1386), and
//! both walk `cg_solidEntities` before/after the collision-model traps. That is
//! C5 `CgWorld` state this seam cannot reach yet, so both take the C4-precedent
//! `todo!()` + `TODO: Port` marker (DEC-46.1); a neutral trace would be the
//! silent fake porting-rules §14 forbids.
//!
//! State: only the engine transport, matching
//! [`CgGameCallbacks`](crate::bg_channel::CgGameCallbacks). The two blocked
//! methods above are where `CgWorld` will enter this file.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use mp_bg::bg_channel::BgTraps;
use mp_engine_select::Engine;
use mp_qshared::common::mp::qcommon::{
    sharedRagDollParams_t, sharedRagDollUpdateParams_t, sharedSetBoneIKStateParams_t,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{
    fileHandle_t, fsMode_t, mdxaBone_t, qboolean, qhandle_t, sharedIKMoveParams_t, vec3_t, vmCvar_t,
};

use crate::trap;

/// The cgame-side `BgTraps` implementation: holds the `&Engine` every
/// delegating method issues its `crate::trap` calls through. Same shape as
/// `mp_game`'s `GameBgTraps` and `mp_ui`'s `UiBgTraps` — a borrowed engine
/// handle, no other state.
pub struct CgBgTraps<'a> {
    pub engine: &'a Engine,
}

impl<'a> CgBgTraps<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }
}

impl BgTraps for CgBgTraps<'_> {
    // ---------------------------------------------------------------------
    // pmove world tests — cgame answers these in cgame code, not a syscall.
    // ---------------------------------------------------------------------

    fn trace(
        &self,
        _results: *mut trace_t,
        _start: *const vec3_t,
        _mins: *const vec3_t,
        _maxs: *const vec3_t,
        _end: *const vec3_t,
        _passEntityNum: c_int,
        _contentMask: c_int,
    ) {
        //TODO: Port CG_Trace args
        // cgame's `pm->trace` is `CG_Trace`, not a trap. The body is
        // transcribed at `crate::cg_predict::CG_Trace`, but it takes
        // `&mut CgContext` and this seam only carries `&Engine` — same story
        // as `pointcontents` below.
        // Source: `oracle/codemp/cgame/cg_predict.c:359-369`;
        // binding at `oracle/codemp/cgame/cg_predict.c:1009,1385`
        todo!("Port CG_Trace seam — oracle/codemp/cgame/cg_predict.c:359-369 (ported at cg_predict::CG_Trace; this seam lacks CgContext)")
    }

    fn pointcontents(&self, _point: *const vec3_t, _passEntityNum: c_int) -> c_int {
        //TODO: Port CG_PointContents
        // Same story: `CG_PointContents` ORs `trap_CM_PointContents` with a
        // `cg_solidEntities` walk. The body is transcribed at
        // `crate::cg_predict::CG_PointContents`, but it takes `&mut CgContext`
        // and this seam only carries `&Engine`.
        // Source: `oracle/codemp/cgame/cg_predict.c:393-424`;
        // binding at `oracle/codemp/cgame/cg_predict.c:1010,1386`
        todo!("Port CG_PointContents seam — oracle/codemp/cgame/cg_predict.c:393-424 (ported at cg_predict::CG_PointContents; this seam lacks CgContext)")
    }

    // ---------------------------------------------------------------------
    // Filesystem — the bg saber/vehicle/siege/anim loaders all run in cgame.
    // ---------------------------------------------------------------------

    fn fs_fopen(&self, qpath: &str, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
        // Raven: `trap_FS_FOpenFile`. Every `.sab`/`.veh`/`.scl`/`.team` and
        // `animation.cfg` open in the cgame build lands here.
        // SAFETY: the bg loaders hand a live out-slot.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:83-85`
        trap::FS_FOpenFile(self.engine, qpath, unsafe { &mut *f }, mode)
    }
    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t) {
        // Raven: `trap_FS_Read`.
        // SAFETY: callers hand a buffer at least `len` bytes wide.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:87-89`
        let buf = unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, len as usize) };
        trap::FS_Read(self.engine, buf, f)
    }
    fn fs_write(&self, buffer: *const c_void, len: c_int, f: fileHandle_t) {
        // Raven: `trap_FS_Write`. No bg loader writes today, but the cgame
        // wrapper is real, so we delegate rather than fake a disposition.
        // SAFETY: callers hand a buffer at least `len` bytes wide.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:91-93`
        let buf = unsafe { core::slice::from_raw_parts(buffer as *const u8, len as usize) };
        trap::FS_Write(self.engine, buf, f)
    }
    fn fs_fclose(&self, f: fileHandle_t) {
        // Raven: `trap_FS_FCloseFile`.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:95-97`
        trap::FS_FCloseFile(self.engine, f)
    }
    fn fs_getfilelist(
        &self,
        path: &str,
        extension: &str,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int {
        // Raven: `trap_FS_GetFileList` — the saber/vehicle/siege directory
        // sweeps. The engine packs NUL-separated names into the caller buffer.
        // SAFETY: callers hand a buffer at least `bufsize` bytes wide.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:99-101`
        let buf = unsafe { core::slice::from_raw_parts_mut(listbuf as *mut u8, bufsize as usize) };
        trap::FS_GetFileList(self.engine, path, extension, buf)
    }

    // ---------------------------------------------------------------------
    // Media registration + ghoul2.
    // ---------------------------------------------------------------------

    fn r_register_skin(&self, name: &str) -> qhandle_t {
        // Raven: `trap_R_RegisterSkin` — `WP_SaberParseParms`'s `customSkin`
        // and `BG_ModelCache`'s skin arm.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:270-272`
        trap::R_RegisterSkin(self.engine, name)
    }

    fn g2api_init_ghoul2_model(
        &self,
        ghoul2Ptr: *mut *mut c_void,
        fileName: &str,
        modelIndex: c_int,
        customSkin: qhandle_t,
        customShader: qhandle_t,
        modelFlags: c_int,
        lodBias: c_int,
    ) -> c_int {
        // Raven: `trap_G2API_InitGhoul2Model` — `BG_ModelCache`'s precache.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:809-813`
        trap::G2API_InitGhoul2Model(
            self.engine,
            ghoul2Ptr,
            fileName,
            modelIndex,
            customSkin,
            customShader,
            modelFlags,
            lodBias,
        )
    }
    fn g2api_clean_ghoul2_models(&self, ghoul2Ptr: *mut *mut c_void) {
        // Raven: `trap_G2API_CleanGhoul2Models` — `BG_ModelCache`'s counterpart.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:856-859`
        trap::G2API_CleanGhoul2Models(self.engine, ghoul2Ptr)
    }

    fn g2api_add_bolt(&self, ghoul2: *mut c_void, modelIndex: c_int, boneName: &str) -> c_int {
        // Raven: `trap_G2API_AddBolt` — the bg vehicle loader's
        // `AttachRidersGeneric` and `BG_AttachToRancor` reach this.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:940-943`
        trap::G2API_AddBolt(self.engine, ghoul2, modelIndex, boneName)
    }
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
    ) -> qboolean {
        // Raven: `strap_G2API_GetBoltMatrix` -> `trap_G2API_GetBoltMatrix`.
        // SAFETY: bg hands live matrix/angles/position/scale slots; only
        // `modelList` is genuinely nullable (bg_pmove passes NULL).
        // Source: `oracle/codemp/cgame/cg_strap.c:6-10`;
        // `oracle/codemp/cgame/cg_syscalls.c:791-795`
        trap::G2API_GetBoltMatrix(
            self.engine,
            ghoul2,
            modelIndex,
            boltIndex,
            unsafe { &mut *matrix },
            unsafe { &*angles },
            unsafe { &*position },
            frameNum,
            unsafe { modelList.as_mut() },
            unsafe { &*scale },
        ) as qboolean
    }
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
    ) -> qboolean {
        // Raven: `strap_G2API_GetBoltMatrix_NoReconstruct`.
        // SAFETY: as `g2api_get_bolt_matrix`.
        // Source: `oracle/codemp/cgame/cg_strap.c:12-16`;
        // `oracle/codemp/cgame/cg_syscalls.c:797-801`
        trap::G2API_GetBoltMatrix_NoReconstruct(
            self.engine,
            ghoul2,
            modelIndex,
            boltIndex,
            unsafe { &mut *matrix },
            unsafe { &*angles },
            unsafe { &*position },
            frameNum,
            unsafe { modelList.as_mut() },
            unsafe { &*scale },
        ) as qboolean
    }
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
    ) -> qboolean {
        // Raven: `strap_G2API_GetBoltMatrix_NoRecNoRot` — the pmove foot-IK
        // path in `bg_pmove.c` is the live caller.
        // SAFETY: as `g2api_get_bolt_matrix`.
        // Source: `oracle/codemp/cgame/cg_strap.c:18-22`;
        // `oracle/codemp/cgame/cg_syscalls.c:803-807`
        trap::G2API_GetBoltMatrix_NoRecNoRot(
            self.engine,
            ghoul2,
            modelIndex,
            boltIndex,
            unsafe { &mut *matrix },
            unsafe { &*angles },
            unsafe { &*position },
            frameNum,
            unsafe { modelList.as_mut() },
            unsafe { &*scale },
        ) as qboolean
    }
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
    ) -> qboolean {
        // Raven: `strap_G2API_SetBoneAngles`.
        // SAFETY: `angles` is always a live vector; `modelList` is nullable.
        // Source: `oracle/codemp/cgame/cg_strap.c:24-29`;
        // `oracle/codemp/cgame/cg_syscalls.c:861-866`
        trap::G2API_SetBoneAngles(
            self.engine,
            ghoul2,
            modelIndex,
            boneName,
            unsafe { &*angles },
            flags,
            up,
            right,
            forward,
            unsafe { modelList.as_mut() },
            blendTime,
            currentTime,
        ) as qboolean
    }
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
    ) -> qboolean {
        // Raven: `strap_G2API_SetBoneAnim`.
        // Source: `oracle/codemp/cgame/cg_strap.c:31-35`;
        // `oracle/codemp/cgame/cg_syscalls.c:868-872`
        trap::G2API_SetBoneAnim(
            self.engine,
            ghoul2,
            modelIndex,
            boneName,
            startFrame,
            endFrame,
            flags,
            animSpeed,
            currentTime,
            setFrame,
            blendTime,
        ) as qboolean
    }
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
    ) -> qboolean {
        // Raven: `strap_G2API_GetBoneAnim`.
        // SAFETY: the five out-slots are always live locals; `modelList` is
        // the nullable one — `BG_IK_MoveArm` passes NULL for it.
        // Source: `oracle/codemp/cgame/cg_strap.c:37-41`;
        // `oracle/codemp/cgame/cg_syscalls.c:874-878`
        trap::G2API_GetBoneAnim(
            self.engine,
            ghoul2,
            boneName,
            currentTime,
            unsafe { &mut *currentFrame },
            unsafe { &mut *startFrame },
            unsafe { &mut *endFrame },
            unsafe { &mut *flags },
            unsafe { &mut *animSpeed },
            unsafe { modelList.as_mut() },
            modelIndex,
        ) as qboolean
    }
    fn g2api_set_rag_doll(&self, ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) {
        // Raven: `strap_G2API_SetRagDoll`.
        // SAFETY: bg builds the params block on its own stack before calling;
        // a null pointer rides through as the engine's reset arm.
        // Source: `oracle/codemp/cgame/cg_strap.c:43-46`;
        // `oracle/codemp/cgame/cg_syscalls.c:998-1001`
        trap::G2API_SetRagDoll(self.engine, ghoul2, unsafe { params.as_mut() })
    }
    fn g2api_animate_g2_models(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    ) {
        // Raven: `strap_G2API_AnimateG2Models` — `BG_IK_MoveArm`'s update leg.
        // SAFETY: bg builds the params block on its own stack before calling.
        // Source: `oracle/codemp/cgame/cg_strap.c:48-51`;
        // `oracle/codemp/cgame/cg_syscalls.c:1003-1006`
        trap::G2API_AnimateG2Models(self.engine, ghoul2, time, unsafe { &mut *params })
    }
    fn g2api_set_bone_ik_state(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        boneName: Option<&str>,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean {
        // Raven: `strap_G2API_SetBoneIKState`. `None` rides through as a null
        // boneName — the engine's init/reset-IK branch — and `params` is
        // nullable too (`BG_IK_MoveArm`'s halt path passes NULL for both).
        // Source: `oracle/codemp/cgame/cg_strap.c:53-56`;
        // `oracle/codemp/cgame/cg_syscalls.c:1040-1043`
        trap::G2API_SetBoneIKState(self.engine, ghoul2, time, boneName, ikState, unsafe {
            params.as_mut()
        }) as qboolean
    }
    fn g2api_ik_move(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> qboolean {
        // Raven: `strap_G2API_IKMove`.
        // SAFETY: bg builds the params block on its own stack before calling.
        // Source: `oracle/codemp/cgame/cg_strap.c:58-61`;
        // `oracle/codemp/cgame/cg_syscalls.c:1045-1048`
        trap::G2API_IKMove(self.engine, ghoul2, time, unsafe { &mut *params }) as qboolean
    }
    fn g2api_get_surface_render_status(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        surfaceName: &str,
    ) -> c_int {
        // Raven: `trap_G2API_GetSurfaceRenderStatus` — bg's rancor/jaw bolt
        // helpers test surfaces with it.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:976-979`
        trap::G2API_GetSurfaceRenderStatus(self.engine, ghoul2, modelIndex, surfaceName)
    }

    // ---------------------------------------------------------------------
    // Effects / misc.
    // ---------------------------------------------------------------------

    fn fx_play_effect_id(
        &self,
        fxID: c_int,
        org: *const vec3_t,
        fwd: *const vec3_t,
        vol: c_int,
        rad: c_int,
    ) {
        // The cgame-only arm: `bg_slidemove.c` declares and calls
        // `trap_FX_PlayEffectID` under `#ifndef QAGAME`, which is why the game
        // tier's implementor panics here and cgame delegates. `mp_bg` still
        // transcribes only `PM_VehicleImpact`'s QAGAME branch, so nothing calls
        // this yet — the arm is wired for when the `#else` branch lands.
        // SAFETY: the caller's two vectors are live for the syscall.
        // Source: `oracle/codemp/game/bg_slidemove.c:37-39,543-553`;
        // `oracle/codemp/cgame/cg_syscalls.c:637-640`
        trap::FX_PlayEffectID(
            self.engine,
            fxID,
            unsafe { &*org },
            unsafe { &*fwd },
            vol,
            rad,
        )
    }
    fn snap_vector(&self, v: *mut f32) {
        // Raven: `trap_SnapVector`; the `float*` is the caller's 3-float buffer
        // (`*mut f32` head == `*mut [f32;3]`), as `PM_SnapVector` hands it in.
        // Source: `oracle/codemp/cgame/cg_syscalls.c:579-581`
        trap::SnapVector(self.engine, unsafe { &mut *(v as *mut vec3_t) })
    }
    fn cvar_register(&self, cvar: *mut vmCvar_t, var_name: &str, value: &str, flags: c_int) {
        // Raven: `trap_Cvar_Register`. Nothing in bg registers a cvar today,
        // but the cgame wrapper is real, so this delegates like the game tier
        // instead of guessing a disposition. `cvar` is nullable in Raven
        // (register-without-a-handle).
        // Source: `oracle/codemp/cgame/cg_syscalls.c:50-52`
        trap::Cvar_Register(
            self.engine,
            unsafe { cvar.as_mut() },
            var_name,
            value,
            flags,
        )
    }

    // ---------------------------------------------------------------------
    // Console.
    // ---------------------------------------------------------------------

    fn com_printf(&self, msg: &str) {
        // Raven `Com_Printf` -> `trap_Print`, the same route cgame's own
        // `CG_Printf` takes. Every bg parse-error report lands here.
        // Source: `oracle/codemp/cgame/cg_main.c:1209-1218`;
        // `oracle/codemp/cgame/cg_syscalls.c:21-23`
        trap::Print(self.engine, msg)
    }
    fn com_error(&self, error_level: c_int, msg: &str) {
        // Raven `Com_Error` -> `trap_Error`; `error_level` is dropped at the
        // seam exactly as `CG_Error` drops it.
        // Source: `oracle/codemp/cgame/cg_main.c:1220-1229`;
        // `oracle/codemp/cgame/cg_syscalls.c:25-27`
        let _ = error_level;
        trap::Error(self.engine, msg)
    }
}
