//! Game-tier implementations of the bg channel traits (pass-3 rulings 13/16).
//!
//! `BgTraps`/`GameCallbacks` are declared in the (conceptually bg) channel with
//! bg-visible signatures; their concrete implementations belong to the game
//! tier and hold the game's `&Engine` / world handle. Ruling 19 keeps
//! everything in `mp_game` for pass 3, so the impls live here beside the traits;
//! when the bg crate splits out, only this file moves to game proper.
//!
//! Exercised in the pmove slice: [`GameBgTraps::pointcontents`] delegates to
//! `crate::trap::PointContents` with a real `Engine` handle — the end-to-end
//! plumbing proof. Un-exercised methods carry `todo!()` bodies (their target
//! `trap_*`/`G_*` are noted); the wiring, not full coverage, is the gate.
#![allow(non_snake_case, unused_variables, clippy::too_many_arguments)]

use core::ffi::{c_char, c_int, c_void};

use mp_engine_select::Engine;

use crate::prelude::*;

use super::bg_traps::BgTraps;
use super::game_callbacks::GameCallbacks;

/// The game-side `BgTraps` implementation: holds the `GameContext` from which
/// engine syscalls are issued via `crate::trap` wrappers.
pub struct GameBgTraps<'a> {
    pub ctx: GameContext<'a>,
}

impl<'a> GameBgTraps<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        // Create a temporary GameContext just for the engine; world is unreachable
        // from BgTraps methods (ruling 13 seam boundary).
        Self {
            ctx: GameContext {
                world: std::ptr::null_mut(),
                engine,
            },
        }
    }
}

impl BgTraps for GameBgTraps<'_> {
    fn trace(
        &self,
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passEntityNum: c_int,
        contentMask: c_int,
    ) {
        // Target: crate::trap::Trace (G_TRACE). Args plumbing lands with the
        // pmove-trace slice; the pointcontents path is the proven one here.
        todo!("Port BgTraps::trace delegation — crate::trap::Trace (G_TRACE)")
    }

    fn pointcontents(&self, point: *const vec3_t, passEntityNum: c_int) -> c_int {
        // Real delegation — the pmove slice's PM_SetWaterLevel drives this.
        // Raven: `trap_PointContents` (`G_POINT_CONTENTS`).
        use mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs;
        crate::trap::PointContents(self.ctx.engine, GPointContentsArgs::new(point, passEntityNum))
    }

    fn fs_fopen(&self, qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
        todo!("Port BgTraps::fs_fopen delegation — crate::trap::FS_FOpenFile")
    }
    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t) {
        // Mechanical delegation (ruling 13) — matches the proven `pointcontents`
        // shape. Raven: `trap_FS_Read` (`G_FS_READ`).
        use mp_abi::game::syscalls::G_FS_READ::GFsReadArgs;
        crate::trap::FS_Read(self.ctx.engine, GFsReadArgs::new(buffer as *mut u8, len, f))
    }
    fn fs_write(&self, buffer: *const c_void, len: c_int, f: fileHandle_t) {
        // Raven: `trap_FS_Write` (`G_FS_WRITE`).
        use mp_abi::game::syscalls::G_FS_WRITE::GFsWriteArgs;
        crate::trap::FS_Write(self.ctx.engine, GFsWriteArgs::new(buffer as *const u8, len, f))
    }
    fn fs_fclose(&self, f: fileHandle_t) {
        // Raven: `trap_FS_FCloseFile` (`G_FS_FCLOSE_FILE`).
        use mp_abi::game::syscalls::G_FS_FCLOSE_FILE::GFsFcloseFileArgs;
        crate::trap::FS_FCloseFile(self.ctx.engine, GFsFcloseFileArgs::new(f as c_int))
    }
    fn fs_getfilelist(
        &self,
        path: *const c_char,
        extension: *const c_char,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int {
        todo!("Port BgTraps::fs_getfilelist delegation — crate::trap::FS_GetFileList")
    }

    fn g2api_add_bolt(&self, ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int {
        // Real delegation to the already-wired `trap_G2API_AddBolt` seam
        // (`G_G2_ADDBOLT`); bg-visible callers (e.g. `AttachRidersGeneric`)
        // only carry `&dyn BgTraps`, not `&Engine`.
        let bone_name = unsafe { std::ffi::CStr::from_ptr(boneName) }.to_owned();
        crate::trap::G2API_AddBolt(
            self.ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(ghoul2, modelIndex, bone_name),
        )
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
        // Raven: `trap_G2API_GetBoltMatrix` (`G_G2_GETBOLT`).
        use mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs;
        crate::trap::G2API_GetBoltMatrix(
            self.ctx.engine,
            GG2GetboltArgs::new(
                ghoul2,
                modelIndex,
                boltIndex,
                matrix,
                angles,
                position,
                frameNum,
                modelList,
                scale,
            ),
        )
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
        todo!("Port BgTraps::g2api_get_bolt_matrix_no_reconstruct — strap_G2API_GetBoltMatrix_NoReconstruct")
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
        todo!("Port BgTraps::g2api_get_bolt_matrix_no_rec_no_rot — strap_G2API_GetBoltMatrix_NoRecNoRot")
    }
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
    ) -> qboolean {
        todo!("Port BgTraps::g2api_set_bone_angles — strap_G2API_SetBoneAngles")
    }
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
    ) -> qboolean {
        todo!("Port BgTraps::g2api_set_bone_anim — strap_G2API_SetBoneAnim")
    }
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
    ) -> qboolean {
        todo!("Port BgTraps::g2api_get_bone_anim — strap_G2API_GetBoneAnim")
    }
    fn g2api_set_rag_doll(&self, ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) {
        todo!("Port BgTraps::g2api_set_rag_doll — strap_G2API_SetRagDoll")
    }
    fn g2api_animate_g2_models(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    ) {
        todo!("Port BgTraps::g2api_animate_g2_models — strap_G2API_AnimateG2Models")
    }
    fn g2api_set_bone_ik_state(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        boneName: *const c_char,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean {
        todo!("Port BgTraps::g2api_set_bone_ik_state — strap_G2API_SetBoneIKState")
    }
    fn g2api_ik_move(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> qboolean {
        todo!("Port BgTraps::g2api_ik_move — strap_G2API_IKMove")
    }
    fn g2api_get_surface_render_status(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        surfaceName: *const c_char,
    ) -> c_int {
        use mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs;
        // Delegates via `crate::trap::G2API_GetSurfaceRenderStatus` (G_G2_GETSURFACERENDERSTATUS);
        // args ABI wants an owned `CString`, so the borrowed C string is copied.
        let surface_name = unsafe { core::ffi::CStr::from_ptr(surfaceName) }.to_owned();
        crate::trap::G2API_GetSurfaceRenderStatus(
            self.ctx.engine,
            GG2GetsurfacerenderstatusArgs::new(ghoul2, modelIndex, surface_name),
        )
    }

    fn fx_play_effect_id(
        &self,
        fxID: c_int,
        org: *const vec3_t,
        fwd: *const vec3_t,
        vol: c_int,
        rad: c_int,
    ) {
        todo!("Port BgTraps::fx_play_effect_id — crate::trap::FX_PlayEffectID")
    }
    fn snap_vector(&self, v: *mut f32) {
        // Raven: `trap_SnapVector` (`G_SNAPVECTOR`); the `vec3_t*` is the caller's
        // 3-float buffer (`*mut f32` head == `*mut [f32;3]`).
        use mp_abi::game::syscalls::G_SNAPVECTOR::GSnapvectorArgs;
        crate::trap::SnapVector(self.ctx.engine, GSnapvectorArgs::new(v as *mut vec3_t))
    }
    fn cvar_register(
        &self,
        cvar: *mut vmCvar_t,
        var_name: *const c_char,
        value: *const c_char,
        flags: c_int,
    ) {
        todo!("Port BgTraps::cvar_register — crate::trap::Cvar_Register")
    }
}

/// The game-side `GameCallbacks` implementation. Carries the game handles the
/// `G_*` upcalls need (the world island + the engine); each method resolves the
/// bg-visible entity nums against the world and delegates to the ported `G_*`
/// body. Bodies are `todo!()` until their targets land — the slice does not
/// drive any upcall (`G_Damage` is reachable only through the `PmoveSingle`
/// remainder), so this proves the shape, not the delegation.
pub struct GameCallbacksImpl<'a> {
    /// The one owned `GameWorld` island (raw so a `&mut dyn GameCallbacks` and a
    /// `&mut BgState` borrowed from the same world can coexist across the seam;
    /// STATE-D6 leaf reborrows discipline applies inside the method bodies).
    pub world: *mut crate::world::GameWorld,
    pub engine: &'a Engine,
}

impl GameCallbacks for GameCallbacksImpl<'_> {
    fn damage(
        &mut self,
        targNum: c_int,
        inflictorNum: c_int,
        attackerNum: c_int,
        dir: *const vec3_t,
        point: *const vec3_t,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    ) {
        todo!("Port GameCallbacks::damage delegation — G_Damage (g_combat.c)")
    }
    fn damage_from_killer(
        &mut self,
        targNum: c_int,
        inflictorNum: c_int,
        attackerNum: c_int,
        killerNum: c_int,
        dir: *const vec3_t,
        point: *const vec3_t,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    ) {
        todo!("Port GameCallbacks::damage_from_killer delegation — G_Damage")
    }
    fn add_event(&mut self, entNum: c_int, event: c_int, eventParm: c_int) {
        todo!("Port GameCallbacks::add_event delegation — G_AddEvent (g_utils.c)")
    }
    fn alloc(&mut self, size: c_int) -> *mut c_void {
        todo!("Port GameCallbacks::alloc delegation — G_Alloc (g_mem.c)")
    }
    fn new_string(&mut self, string: *const c_char) -> *mut c_char {
        todo!("Port GameCallbacks::new_string delegation — G_NewString (g_spawn.c)")
    }
    fn play_effect(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) {
        todo!("Port GameCallbacks::play_effect delegation — G_PlayEffect")
    }
    fn play_effect_id(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) -> c_int {
        todo!("Port GameCallbacks::play_effect_id delegation — G_PlayEffectID")
    }
    fn sound_index(&mut self, name: *const c_char) -> c_int {
        todo!("Port GameCallbacks::sound_index delegation — G_SoundIndex")
    }
    fn model_index(&mut self, name: *const c_char) -> c_int {
        todo!("Port GameCallbacks::model_index delegation — G_ModelIndex")
    }
    fn effect_index(&mut self, name: *const c_char) -> c_int {
        todo!("Port GameCallbacks::effect_index delegation — G_EffectIndex")
    }
    fn cheap_weapon_fire(&mut self, entNum: c_int, weapon: c_int) {
        todo!("Port GameCallbacks::cheap_weapon_fire delegation — g_weapon.c")
    }
    fn client_check_impact_bbrush(&mut self, entNum: c_int, impactNum: c_int) {
        todo!("Port GameCallbacks::client_check_impact_bbrush delegation — g_active.c")
    }
    fn flyveh_surface_destruction(&mut self, entNum: c_int, trNum: c_int, magnitude: f32) {
        todo!("Port GameCallbacks::flyveh_surface_destruction delegation — g_vehicles.c")
    }
    fn set_anim(
        &mut self,
        entNum: c_int,
        ucmd: *mut usercmd_t,
        setAnimParts: c_int,
        anim: c_int,
        setAnimFlags: c_int,
        blendTime: c_int,
    ) {
        todo!("Port GameCallbacks::set_anim delegation — G_SetAnim")
    }
    fn npc_set_anim(&mut self, entNum: c_int, type_: c_int, anim: c_int, priority: c_int) {
        todo!("Port GameCallbacks::npc_set_anim delegation — NPC_SetAnim")
    }
    fn get_vehicle_cam_pos(&mut self, entNum: c_int, camPos: *mut vec3_t) {
        todo!("Port GameCallbacks::get_vehicle_cam_pos delegation — g_vehicles.c")
    }
    fn can_be_enemy(&mut self, entNum: c_int, otherNum: c_int) -> qboolean {
        todo!("Port GameCallbacks::can_be_enemy delegation — g_combat.c")
    }
    fn get_time(&self) -> c_int {
        todo!("Port GameCallbacks::get_time — level.time accessor")
    }
    fn try_grapple(&mut self, entNum: c_int) -> qboolean {
        todo!("Port GameCallbacks::try_grapple delegation — g_active.c")
    }
    fn q3_set_parm(&mut self, entID: c_int, parmNum: c_int, parmValue: *const c_char) {
        todo!("Port GameCallbacks::q3_set_parm delegation — Q3_SetParm (g_ICARUScb.c)")
    }
}
