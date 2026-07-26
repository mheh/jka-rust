//! `UiBgTraps` — the ui-tier `BgTraps` implementation.
//!
//! `mp_bg::bg_channel::BgTraps` is the bg tier's outbound engine surface
//! (`crates/mp/bg/src/bg_channel/bg_traps.rs`); the game tier's implementor is
//! [`mp_game`'s `GameBgTraps`](../../../game/src/bg_channel/game_impl.rs).
//! ui's `&dyn BgTraps` consumers are [`crate::ui_main::UI_SiegeInit`]'s
//! siege-loader call chain (`BG_SiegeLoadClasses`/`BG_SiegeLoadTeams` and the
//! `BG_SiegeParseClassFile`/`BG_SiegeParseTeamFile` they call — verified by
//! reading every `traps.*` call in `crates/mp/bg/src/bg_saga.rs`'s siege-load
//! path: `fs_getfilelist`, `fs_fopen`, `fs_read`, `fs_fclose`, `com_printf`)
//! and [`DisplayContext::UI_ParseAnimationFile`](crate::ui_display_context)'s
//! `BG_ParseAnimationFile` call (`fs_fopen`/`fs_read`/`fs_fclose` again, plus
//! `com_error` on its oversized-file guard). Every other `BgTraps` method is
//! unreachable from ui and panics loudly with its Raven subject
//! (porting-rules §14: no silent no-ops); several also have no `UI_*` syscall
//! at all (`trace`/`pointcontents`/`snap_vector`/`fx_play_effect_id`/
//! `g2api_get_surface_render_status` — ui never traces the world or plays
//! view-model effects).

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use mp_bg::bg_channel::BgTraps;
use mp_engine_select::Engine;
use mp_qshared::common::mp::qcommon::{
    sharedRagDollParams_t, sharedRagDollUpdateParams_t, sharedSetBoneIKStateParams_t,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::cvar::vmCvar_t;
use mp_qshared::shared::{
    fileHandle_t, fsMode_t, mdxaBone_t, qboolean, qhandle_t, sharedIKMoveParams_t, vec3_t,
};

use crate::trap;

/// The ui-side `BgTraps` implementation: holds the `&Engine` the reachable
/// methods issue `crate::trap` calls through. Mirrors `mp_game`'s
/// `GameBgTraps` shape (a borrowed `Engine` handle, no other state).
pub struct UiBgTraps<'a> {
    pub engine: &'a Engine,
}

impl<'a> UiBgTraps<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }
}

impl BgTraps for UiBgTraps<'_> {
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
        // No `UI_TRACE` syscall exists — the ui module never traces the world.
        // Source: `oracle/codemp/game/bg_public.h:484`
        unreachable!("trap_Trace is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.trace")
    }

    fn pointcontents(&self, _point: *const vec3_t, _passEntityNum: c_int) -> c_int {
        // No `UI_POINT_CONTENTS` syscall exists.
        // Source: `oracle/codemp/game/bg_public.h:485`
        unreachable!("trap_PointContents is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.pointcontents")
    }

    fn fs_fopen(&self, qpath: &str, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
        // Real delegation — `BG_SiegeParseClassFile`/`BG_SiegeParseTeamFile` open
        // every `.scl`/`.team` file through this, and `BG_ParseAnimationFile`
        // opens `animation.cfg` the same way. Raven: `trap_FS_FOpenFile`.
        // SAFETY: callers (the bg siege loader / anim parser) hand a valid,
        // live out-slot.
        trap::FS_FOpenFile(self.engine, qpath, unsafe { &mut *f }, mode)
    }
    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t) {
        // Real delegation — reads the opened `.scl`/`.team`/`animation.cfg`
        // file body. Raven: `trap_FS_Read`.
        // SAFETY: callers hand a buffer at least `len` bytes wide.
        let buf = unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, len as usize) };
        trap::FS_Read(self.engine, buf, f)
    }
    fn fs_write(&self, _buffer: *const c_void, _len: c_int, _f: fileHandle_t) {
        // The siege loader only ever reads class/team files.
        // Source: `oracle/codemp/game/g_syscalls.c`
        unreachable!("trap_FS_Write is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.fs_write")
    }
    fn fs_fclose(&self, f: fileHandle_t) {
        // Real delegation. Raven: `trap_FS_FCloseFile`.
        trap::FS_FCloseFile(self.engine, f)
    }
    fn fs_getfilelist(
        &self,
        path: &str,
        extension: &str,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int {
        // Real delegation — `BG_SiegeLoadClasses`/`BG_SiegeLoadTeams` list every
        // `.scl`/`.team` file this way. Raven: `trap_FS_GetFileList`.
        // SAFETY: callers hand a buffer at least `bufsize` bytes wide.
        let buf = unsafe { core::slice::from_raw_parts_mut(listbuf as *mut u8, bufsize as usize) };
        trap::FS_GetFileList(self.engine, path, extension, buf)
    }

    fn r_register_skin(&self, _name: &str) -> qhandle_t {
        // The siege loader never registers a skin directly (only shaders, via
        // `GameCallbacks::siege_class_ui_portrait`/`siege_class_shader`).
        // Source: `oracle/codemp/game/g_syscalls.c:1179-1182`
        unreachable!("trap_R_RegisterSkin is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.r_register_skin")
    }

    fn g2api_init_ghoul2_model(
        &self,
        _ghoul2Ptr: *mut *mut c_void,
        _fileName: &str,
        _modelIndex: c_int,
        _customSkin: qhandle_t,
        _customShader: qhandle_t,
        _modelFlags: c_int,
        _lodBias: c_int,
    ) -> c_int {
        // Source: `oracle/codemp/game/g_syscalls.c:1223-1227`
        unreachable!("trap_G2API_InitGhoul2Model is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_init_ghoul2_model")
    }
    fn g2api_clean_ghoul2_models(&self, _ghoul2Ptr: *mut *mut c_void) {
        // Source: `oracle/codemp/game/g_syscalls.c:1303-1306`
        unreachable!("trap_G2API_CleanGhoul2Models is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_clean_ghoul2_models")
    }

    fn g2api_add_bolt(&self, _ghoul2: *mut c_void, _modelIndex: c_int, _boneName: &str) -> c_int {
        // Source: `oracle/codemp/game/g_syscalls.c:1239-1242`
        unreachable!("trap_G2API_AddBolt is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_add_bolt")
    }
    fn g2api_get_bolt_matrix(
        &self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _boltIndex: c_int,
        _matrix: *mut mdxaBone_t,
        _angles: *const vec3_t,
        _position: *const vec3_t,
        _frameNum: c_int,
        _modelList: *mut qhandle_t,
        _scale: *const vec3_t,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:6-10`
        unreachable!("strap_G2API_GetBoltMatrix is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_get_bolt_matrix")
    }
    fn g2api_get_bolt_matrix_no_reconstruct(
        &self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _boltIndex: c_int,
        _matrix: *mut mdxaBone_t,
        _angles: *const vec3_t,
        _position: *const vec3_t,
        _frameNum: c_int,
        _modelList: *mut qhandle_t,
        _scale: *const vec3_t,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:12-16`
        unreachable!("strap_G2API_GetBoltMatrix_NoReconstruct is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_get_bolt_matrix_no_reconstruct")
    }
    fn g2api_get_bolt_matrix_no_rec_no_rot(
        &self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _boltIndex: c_int,
        _matrix: *mut mdxaBone_t,
        _angles: *const vec3_t,
        _position: *const vec3_t,
        _frameNum: c_int,
        _modelList: *mut qhandle_t,
        _scale: *const vec3_t,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:18-22`
        unreachable!("strap_G2API_GetBoltMatrix_NoRecNoRot is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_get_bolt_matrix_no_rec_no_rot")
    }
    fn g2api_set_bone_angles(
        &self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _boneName: &str,
        _angles: *const vec3_t,
        _flags: c_int,
        _up: c_int,
        _right: c_int,
        _forward: c_int,
        _modelList: *mut qhandle_t,
        _blendTime: c_int,
        _currentTime: c_int,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:24-29`
        unreachable!("strap_G2API_SetBoneAngles is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_set_bone_angles")
    }
    fn g2api_set_bone_anim(
        &self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _boneName: &str,
        _startFrame: c_int,
        _endFrame: c_int,
        _flags: c_int,
        _animSpeed: f32,
        _currentTime: c_int,
        _setFrame: f32,
        _blendTime: c_int,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:31-35`
        unreachable!("strap_G2API_SetBoneAnim is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_set_bone_anim")
    }
    fn g2api_get_bone_anim(
        &self,
        _ghoul2: *mut c_void,
        _boneName: &str,
        _currentTime: c_int,
        _currentFrame: *mut f32,
        _startFrame: *mut c_int,
        _endFrame: *mut c_int,
        _flags: *mut c_int,
        _animSpeed: *mut f32,
        _modelList: *mut c_int,
        _modelIndex: c_int,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:37-41`
        unreachable!("strap_G2API_GetBoneAnim is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_get_bone_anim")
    }
    fn g2api_set_rag_doll(&self, _ghoul2: *mut c_void, _params: *mut sharedRagDollParams_t) {
        // Source: `oracle/codemp/game/g_strap.c:43-46`
        unreachable!("strap_G2API_SetRagDoll is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_set_rag_doll")
    }
    fn g2api_animate_g2_models(
        &self,
        _ghoul2: *mut c_void,
        _time: c_int,
        _params: *mut sharedRagDollUpdateParams_t,
    ) {
        // Source: `oracle/codemp/game/g_strap.c:48-51`
        unreachable!("strap_G2API_AnimateG2Models is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_animate_g2_models")
    }
    fn g2api_set_bone_ik_state(
        &self,
        _ghoul2: *mut c_void,
        _time: c_int,
        _boneName: Option<&str>,
        _ikState: c_int,
        _params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:53-56`
        unreachable!("strap_G2API_SetBoneIKState is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_set_bone_ik_state")
    }
    fn g2api_ik_move(
        &self,
        _ghoul2: *mut c_void,
        _time: c_int,
        _params: *mut sharedIKMoveParams_t,
    ) -> qboolean {
        // Source: `oracle/codemp/game/g_strap.c:58-61`
        unreachable!("strap_G2API_IKMove is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_ik_move")
    }
    fn g2api_get_surface_render_status(
        &self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _surfaceName: &str,
    ) -> c_int {
        // No `UI_G2_GETSURFACERENDERSTATUS` syscall exists.
        // Source: `oracle/codemp/game/g_syscalls.c:1370`
        unreachable!("trap_G2API_GetSurfaceRenderStatus is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.g2api_get_surface_render_status")
    }

    fn fx_play_effect_id(
        &self,
        _fxID: c_int,
        _org: *const vec3_t,
        _fwd: *const vec3_t,
        _vol: c_int,
        _rad: c_int,
    ) {
        // No `UI_FX_PLAY_EFFECT_ID` syscall exists.
        // Source: `oracle/codemp/game/g_syscalls.c`
        unreachable!("trap_FX_PlayEffectID is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.fx_play_effect_id")
    }
    fn snap_vector(&self, _v: *mut f32) {
        // No `UI_SNAPVECTOR` syscall exists.
        // Source: `oracle/codemp/game/g_syscalls.c`
        unreachable!("trap_SnapVector is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.snap_vector")
    }
    fn cvar_register(&self, _cvar: *mut vmCvar_t, _var_name: &str, _value: &str, _flags: c_int) {
        // The siege loader registers no cvars directly (`_UI_Init` registers
        // `g_siegeTeamSwitch` itself, outside this seam).
        // Source: `oracle/codemp/game/g_syscalls.c`
        unreachable!("trap_Cvar_Register is unreachable from ui: UI_SiegeInit's siege-loader path never calls traps.cvar_register")
    }

    fn com_printf(&self, msg: &str) {
        // Real delegation — `BG_SiegeParseClassFile`'s missing-`class_shader`
        // report. Raven: `Com_Printf` -> `trap_Print`.
        trap::Print(self.engine, msg)
    }
    fn com_error(&self, _error_level: c_int, msg: &str) {
        // Real delegation — `BG_ParseAnimationFile`'s oversized-animation.cfg
        // guard (`DisplayContext::UI_ParseAnimationFile`'s call chain) reaches
        // this; the siege loader itself still never calls it (its fatal paths
        // are the *caller*'s, `UI_SiegeInit`'s, `Com_Error`).
        // Raven: `Com_Error` -> `trap_Error`.
        // Source: `oracle/codemp/game/g_main.c:1208-1217`
        trap::Error(self.engine, msg)
    }
}
