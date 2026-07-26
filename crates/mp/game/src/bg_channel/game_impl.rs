//! Game-tier implementations of the bg channel traits.
//!
//! `BgTraps`/`GameCallbacks` are declared in the (conceptually bg) channel with
//! bg-visible signatures; their concrete implementations belong to the game
//! tier and hold the game's `&Engine` / world handle. Everything currently
//! lives in `mp_game`, so the impls live here beside the traits; when the bg
//! crate splits out, only this file moves to game proper.
//!
//! Exercised in the pmove slice: [`GameBgTraps::pointcontents`] delegates to
//! `crate::trap::PointContents` with a real `Engine` handle — the end-to-end
//! plumbing proof. Every other `BgTraps`/`GameCallbacks` method now delegates the
//! same way (resolve entity nums against the world arena, rebuild `GameContext`,
//! call the ported `trap_*`/`G_*` body). One exception remains:
//! [`GameCallbacksImpl::flyveh_surface_destruction`] is a loud `todo!()`
//! escalation — its bg-visible signature cannot carry the impact `trace_t*`/force
//! flag its game-tier target needs (see the method for the full note).
#![allow(non_snake_case, unused_variables, clippy::too_many_arguments)]

use core::ffi::{c_char, c_int, c_void};

use mp_engine_select::Engine;

use crate::prelude::*;
use crate::FighterNPC::FighterIsLanded;
use crate::g_ICARUScb::Q3_SetParm;
use crate::g_active::{Client_CheckImpactBBrush, G_CheapWeaponFire};
use crate::g_cmds::TryGrapple;
use crate::g_combat::{G_Damage, G_DamageFromKiller};
use crate::g_mem::G_Alloc;
use crate::g_utils::{
    G_AddEvent, G_EffectIndex, G_ModelIndex, G_PlayEffect, G_PlayEffectID, G_SetAnim, G_SoundIndex,
};
use crate::g_vehicles::G_FlyVehicleSurfaceDestruction;
use crate::g_weapon::WP_GetVehicleCamPos;
use crate::npc_c::NPC_SetAnim;
use crate::trap;
use crate::veh_dispatch;
use crate::w_saber::G_CanBeEnemy;
use crate::world::GameWorld;

use super::bg_traps::BgTraps;
use super::game_callbacks::GameCallbacks;

/// The game-side `BgTraps` implementation: holds the `&Engine` from which
/// engine syscalls are issued via `crate::trap` wrappers (Stage 2a: the former
/// null-world placeholder `GameContext` is impossible with a borrowed world —
/// and was never needed; only the engine channel is).
pub struct GameBgTraps<'a> {
    pub engine: &'a Engine,
}

impl<'a> GameBgTraps<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
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
        // Mechanical delegation. Raven: `trap_Trace` (`G_TRACE`).
        use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
        trap::Trace(
            self.engine,
            GTraceArgs::new(results, start, mins, maxs, end, passEntityNum, contentMask),
        )
    }

    fn pointcontents(&self, point: *const vec3_t, passEntityNum: c_int) -> c_int {
        // Real delegation — the pmove slice's PM_SetWaterLevel drives this.
        // Raven: `trap_PointContents` (`G_POINT_CONTENTS`).
        use mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs;
        trap::PointContents(self.engine, GPointContentsArgs::new(point, passEntityNum))
    }

    fn fs_fopen(&self, qpath: &str, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
        // Raven: `trap_FS_FOpenFile` (`G_FS_FOPEN_FILE`). The caller guarantees
        // `f` is valid.
        trap::FS_FOpenFile(self.engine, qpath, unsafe { &mut *f }, mode)
    }
    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t) {
        // Mechanical delegation — matches the proven `pointcontents`
        // shape. Raven: `trap_FS_Read` (`G_FS_READ`).
        let buf =
            unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, len as usize) };
        trap::FS_Read(self.engine, buf, f)
    }
    fn fs_write(&self, buffer: *const c_void, len: c_int, f: fileHandle_t) {
        // Raven: `trap_FS_Write` (`G_FS_WRITE`).
        let buf = unsafe { core::slice::from_raw_parts(buffer as *const u8, len as usize) };
        trap::FS_Write(self.engine, buf, f)
    }
    fn fs_fclose(&self, f: fileHandle_t) {
        // Raven: `trap_FS_FCloseFile` (`G_FS_FCLOSE_FILE`).
        trap::FS_FCloseFile(self.engine, f)
    }
    fn fs_getfilelist(
        &self,
        path: &str,
        extension: &str,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int {
        // Raven: `trap_FS_GetFileList` (`G_FS_GETFILELIST`).
        let list =
            unsafe { core::slice::from_raw_parts_mut(listbuf as *mut u8, bufsize as usize) };
        trap::FS_GetFileList(self.engine, path, extension, list)
    }

    fn r_register_skin(&self, name: &str) -> qhandle_t {
        // Raven: `trap_R_RegisterSkin` (`G_R_REGISTERSKIN`).
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
        // Mechanical delegation. Raven: `trap_G2API_InitGhoul2Model`
        // (`G_G2_INITGHOUL2MODEL`).
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
        // Mechanical delegation. Raven: `trap_G2API_CleanGhoul2Models`
        // (`G_G2_CLEANMODELS`).
        trap::G2API_CleanGhoul2Models(
            self.engine,
            mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs::new(ghoul2Ptr),
        )
    }
    fn g2api_add_bolt(&self, ghoul2: *mut c_void, modelIndex: c_int, boneName: &str) -> c_int {
        // Real delegation to the already-wired `trap_G2API_AddBolt` seam
        // (`G_G2_ADDBOLT`); bg-visible callers (e.g. `AttachRidersGeneric`)
        // only carry `&dyn BgTraps`, not `&Engine`.
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
        // Raven: `trap_G2API_GetBoltMatrix` (`G_G2_GETBOLT`).
        use mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs;
        trap::G2API_GetBoltMatrix(
            self.engine,
            GG2GetboltArgs::new(
                ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList, scale,
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
        // Raven: `trap_G2API_GetBoltMatrix_NoReconstruct` (`G_G2_GETBOLT_NOREC`).
        // The syscall `Args` takes `scale` as `*mut vec3_t`; the bg-visible sig is
        // `*const`, so cast at the seam (the engine never mutates it here).
        use mp_abi::game::syscalls::G_G2_GETBOLT_NOREC::GG2GetboltNorecArgs;
        trap::G2API_GetBoltMatrix_NoReconstruct(
            self.engine,
            GG2GetboltNorecArgs::new(
                ghoul2,
                modelIndex,
                boltIndex,
                matrix,
                angles,
                position,
                frameNum,
                modelList,
                scale as *mut vec3_t,
            ),
        )
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
        // Raven: `trap_G2API_GetBoltMatrix_NoRecNoRot` (`G_G2_GETBOLT_NOREC_NOROT`).
        use mp_abi::game::syscalls::G_G2_GETBOLT_NOREC_NOROT::GG2GetboltNorecNorotArgs;
        trap::G2API_GetBoltMatrix_NoRecNoRot(
            self.engine,
            GG2GetboltNorecNorotArgs::new(
                ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList, scale,
            ),
        )
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
        // Raven: `trap_G2API_SetBoneAngles` (`G_G2_ANGLEOVERRIDE`).
        (trap::G2API_SetBoneAngles(
            self.engine,
            ghoul2,
            modelIndex,
            boneName,
            angles,
            flags,
            up,
            right,
            forward,
            modelList,
            blendTime,
            currentTime,
        )) as qboolean
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
        // Raven: `trap_G2API_SetBoneAnim` (`G_G2_PLAYANIM`).
        (trap::G2API_SetBoneAnim(
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
        )) as qboolean
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
        // Raven: `trap_G2API_GetBoneAnim` (`G_G2_GETBONEANIM`).
        (trap::G2API_GetBoneAnim(
            self.engine,
            ghoul2,
            boneName,
            currentTime,
            currentFrame,
            startFrame,
            endFrame,
            flags,
            animSpeed,
            modelList,
            modelIndex,
        )) as qboolean
    }
    fn g2api_set_rag_doll(&self, ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) {
        // Raven: `trap_G2API_SetRagDoll` (`G_G2_SETRAGDOLL`).
        use mp_abi::game::syscalls::G_G2_SETRAGDOLL::GG2SetragdollArgs;
        trap::G2API_SetRagDoll(self.engine, GG2SetragdollArgs::new(ghoul2, params))
    }
    fn g2api_animate_g2_models(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    ) {
        // Raven: `trap_G2API_AnimateG2Models` (`G_G2_ANIMATEG2MODELS`).
        use mp_abi::game::syscalls::G_G2_ANIMATEG2MODELS::GG2Animateg2ModelsArgs;
        trap::G2API_AnimateG2Models(
            self.engine,
            GG2Animateg2ModelsArgs::new(ghoul2, time, params),
        )
    }
    fn g2api_set_bone_ik_state(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        boneName: Option<&str>,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean {
        // Raven: `trap_G2API_SetBoneIKState` (`G_G2_SETBONEIKSTATE`).
        // `None` rides through as a null boneName on the wire — the engine's
        // init/reset-IK branch (`G2_bones.cpp:4674`).
        (trap::G2API_SetBoneIKState(self.engine, ghoul2, time, boneName, ikState, params))
            as qboolean
    }
    fn g2api_ik_move(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> qboolean {
        // Raven: `trap_G2API_IKMove` (`G_G2_IKMOVE`).
        use mp_abi::game::syscalls::G_G2_IKMOVE::GG2IkmoveArgs;
        trap::G2API_IKMove(self.engine, GG2IkmoveArgs::new(ghoul2, time, params))
    }
    fn g2api_get_surface_render_status(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        surfaceName: &str,
    ) -> c_int {
        // Delegates via `crate::trap::G2API_GetSurfaceRenderStatus`
        // (G_G2_GETSURFACERENDERSTATUS).
        trap::G2API_GetSurfaceRenderStatus(self.engine, ghoul2, modelIndex, surfaceName)
    }

    fn fx_play_effect_id(
        &self,
        fxID: c_int,
        org: *const vec3_t,
        fwd: *const vec3_t,
        vol: c_int,
        rad: c_int,
    ) {
        // `trap_FX_PlayEffectID` is cgame-only: in the oracle it is declared and
        // called only under `#ifndef QAGAME` (bg_slidemove.c:38,122,550), while
        // the QAGAME (server/game) build routes the same effect through
        // `G_PlayEffectID` (a `GameCallbacks` upcall). There is no
        // `G_FX_PLAY_EFFECT_ID` in the game syscall table, so this method is dead
        // surface on the game side and must never be reached here.
        // Source: `oracle/codemp/game/bg_slidemove.c:37-39,116-124`
        unreachable!(
            "trap_FX_PlayEffectID is cgame-only (#ifndef QAGAME); QAGAME uses G_PlayEffectID"
        )
    }
    fn snap_vector(&self, v: *mut f32) {
        // Raven: `trap_SnapVector` (`G_SNAPVECTOR`); the `vec3_t*` is the caller's
        // 3-float buffer (`*mut f32` head == `*mut [f32;3]`).
        use mp_abi::game::syscalls::G_SNAPVECTOR::GSnapvectorArgs;
        trap::SnapVector(self.engine, GSnapvectorArgs::new(v as *mut vec3_t))
    }
    fn cvar_register(&self, cvar: *mut vmCvar_t, var_name: &str, value: &str, flags: c_int) {
        // Raven: `trap_Cvar_Register` (`G_CVAR_REGISTER`).
        let cvar = unsafe { cvar.as_mut() };
        trap::Cvar_Register(self.engine, cvar, var_name, value, flags)
    }

    fn com_printf(&self, msg: &str) {
        // Raven `Com_Printf` -> `trap_Print` (`G_PRINT`), the same route the
        // game-tier `Com_Printf` port takes. Source: `g_main.c:1219-1228`.
        trap::Printf(self.engine, msg)
    }
    fn com_error(&self, error_level: c_int, msg: &str) {
        // Raven `Com_Error` -> `trap_Error` (`G_ERROR`); `error_level` is dropped
        // at the seam like the game-tier `Com_Error` port. Source: `g_main.c:1208-1217`.
        let _ = error_level;
        trap::Error(self.engine, msg)
    }
}

/// The game-side `GameCallbacks` implementation. Carries the game handles the
/// `G_*` upcalls need (the world island + the engine); each method resolves the
/// bg-visible entity nums against the world and delegates to the ported `G_*`
/// body. All upcalls now delegate; the sole exception is
/// [`GameCallbacksImpl::flyveh_surface_destruction`], a documented escalation
/// (its signature cannot carry the impact `trace_t*`/force the target requires).
pub struct GameCallbacksImpl<'a> {
    /// The one owned `GameWorld` island (raw so a `&mut dyn GameCallbacks` and a
    /// `&mut BgState` borrowed from the same world can coexist across the seam;
    /// STATE-D6 leaf reborrows discipline applies inside the method bodies).
    pub world: *mut GameWorld,
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
        // Resolves the entity nums against the world arena, rebuilds the module
        // `GameContext`, and delegates to the ported `G_Damage`. `dir`/`point` are
        // bg-visible raw pointers; the ported body takes `dir: Option<&mut vec3_t>`
        // and `point` by value. Every bg caller passes `dir = null` and a non-null
        // `point`. Source: `oracle/codemp/game/g_combat.c` (`G_Damage`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let targ = &mut (*self.world).g_entities[targNum as usize] as *mut gentity_t;
            let inflictor = &mut (*self.world).g_entities[inflictorNum as usize] as *mut gentity_t;
            let attacker = &mut (*self.world).g_entities[attackerNum as usize] as *mut gentity_t;
            let dir = if dir.is_null() {
                None
            } else {
                Some(&mut *(dir as *mut vec3_t))
            };
            let targ_id = ctx.entity_id_of(targ);
            let inflictor_id = ctx.entity_id_of(inflictor);
            let attacker_id = ctx.entity_id_of(attacker);
            G_Damage(
                &mut ctx,
                targ_id,
                inflictor_id,
                attacker_id,
                dir,
                *point,
                damage,
                dflags,
                mod_,
            );
        }
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
        // Delegates to `G_DamageFromKiller(pEnt, pVehEnt, attacker, org, ...)`: the
        // bg-visible `targNum`->`pEnt`, `inflictorNum`->`pVehEnt`, `attackerNum`->
        // `attacker`. `killerNum` is not a body parameter — `G_DamageFromKiller`
        // initializes `killer = attacker` internally (bg passes it equal to
        // `attackerNum`); `dir` is unused (bg passes null). `point`->`org` by value.
        // Source: `oracle/codemp/game/g_combat.c` (`G_DamageFromKiller`).
        let _ = (killerNum, dir);
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let pEnt = &mut (*self.world).g_entities[targNum as usize] as *mut gentity_t;
            let pVehEnt = &mut (*self.world).g_entities[inflictorNum as usize] as *mut gentity_t;
            let attacker = &mut (*self.world).g_entities[attackerNum as usize] as *mut gentity_t;
            let pEnt_id = ctx.entity_id_of(pEnt);
            let pVehEnt_id = ctx.entity_id_of(pVehEnt);
            let attacker_id_2 = ctx.entity_id_of(attacker);
            G_DamageFromKiller(
                &mut ctx,
                pEnt_id,
                pVehEnt_id,
                attacker_id_2,
                *point,
                damage,
                dflags,
                mod_,
            );
        }
    }
    fn add_event(&mut self, entNum: c_int, event: c_int, eventParm: c_int) {
        // `G_AddEvent` is ctx-free and takes a `gentity_t*`; resolve `entNum`.
        // Source: `oracle/codemp/game/g_utils.c` (`G_AddEvent`).
        unsafe {
            let ent = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            G_AddEvent(&mut *(ent), event, eventParm);
        }
    }
    fn entity_legs_anim(&self, entNum: c_int) -> c_int {
        unsafe { (*self.world).g_entities[entNum as usize].s.legsAnim }
    }
    fn entity_torso_anim(&self, entNum: c_int) -> c_int {
        unsafe { (*self.world).g_entities[entNum as usize].s.torsoAnim }
    }
    fn alloc(&mut self, size: c_int) -> *mut c_void {
        // `G_Alloc` bumps the game pool via `ctx.world`; rebuild the ctx from the
        // impl's owned `world`/`engine` (STATE-D6 leaf reborrow).
        // Source: `oracle/codemp/game/g_mem.c` (`G_Alloc`).
        // SAFETY: seam reborrow of the impl's owned world island (STATE-D6);
        // single-threaded module, no live sibling borrow across this call.
        let mut ctx = GameContext {
            world: unsafe { &mut *self.world },
            engine: self.engine,
        };
        G_Alloc(&mut ctx, size)
    }
    fn new_string(&mut self, string: &str) -> *mut c_char {
        // `prefix_string` stores the copy in `ctx.world`'s level-lifetime prefix
        // arena (replacing `G_NewString`'s pool copy) and returns the slot pointer.
        // Source: `oracle/codemp/game/g_spawn.c:724-749` (`G_NewString`).
        // SAFETY: seam reborrow of the impl's owned world island (STATE-D6);
        // single-threaded module, no live sibling borrow across this call.
        let mut ctx = GameContext {
            world: unsafe { &mut *self.world },
            engine: self.engine,
        };
        ctx.prefix_string(string)
    }
    fn play_effect(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) {
        // `G_PlayEffect` is ctx-free and takes `org`/`ang` by value; the spawned
        // temp-entity return is discarded (as at the bg call sites).
        // Source: `oracle/codemp/game/g_utils.c` (`G_PlayEffect`).
        unsafe {
            G_PlayEffect(fxID, *org, *ang);
        }
    }
    fn play_effect_id(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) -> c_int {
        // `G_PlayEffectID` returns the spawned temp-entity; the bg-visible upcall
        // yields its entity number (`ENTITYNUM_NONE` when none was spawned).
        // Source: `oracle/codemp/game/g_utils.c` (`G_PlayEffectID`).
        unsafe {
            let te = G_PlayEffectID(fxID, *org, *ang);
            if te.is_null() {
                ENTITYNUM_NONE
            } else {
                (*te).s.number
            }
        }
    }
    fn sound_index(&mut self, name: &str) -> c_int {
        // Source: `g_utils.c` (`G_SoundIndex`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            G_SoundIndex(&mut ctx, name)
        }
    }
    fn model_index(&mut self, name: &str) -> c_int {
        // Source: `g_utils.c` (`G_ModelIndex`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            G_ModelIndex(&mut ctx, name)
        }
    }
    fn effect_index(&mut self, name: &str) -> c_int {
        // Source: `g_utils.c` (`G_EffectIndex`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            G_EffectIndex(&mut ctx, name)
        }
    }
    fn cheap_weapon_fire(&mut self, entNum: c_int, weapon: c_int) {
        // Raven `G_CheapWeaponFire(entNum, ev)` takes the entity number directly.
        // Source: `oracle/codemp/game/g_active.c` (`G_CheapWeaponFire`).
        // SAFETY: seam reborrow of the impl's owned world island (STATE-D6);
        // single-threaded module, no live sibling borrow across this call.
        let mut ctx = GameContext {
            world: unsafe { &mut *self.world },
            engine: self.engine,
        };
        G_CheapWeaponFire(&mut ctx, entNum, weapon);
    }
    fn client_check_impact_bbrush(&mut self, entNum: c_int, impactNum: c_int) {
        // Raven `Client_CheckImpactBBrush(self, other)`; resolve both nums.
        // Source: `oracle/codemp/game/g_active.c` (`Client_CheckImpactBBrush`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let self_ = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            let other = &mut (*self.world).g_entities[impactNum as usize] as *mut gentity_t;
            let self_id_2 = ctx.entity_id_of(self_);
            let other_id = ctx.entity_id_of(other);
            Client_CheckImpactBBrush(&mut ctx, self_id_2, other_id);
        }
    }
    fn flyveh_surface_destruction(
        &mut self,
        entNum: c_int,
        trace: *mut trace_t,
        magnitude: c_int,
        force: qboolean,
    ) {
        // Resolve `entNum`->vehicle gentity, rebuild `ctx`, and delegate to
        // `G_FlyVehicleSurfaceDestruction` with the bg-supplied impact `trace` and
        // `force` flag. Source: `oracle/codemp/game/g_vehicles.c:3190`;
        // `bg_slidemove.c:472`.
        // SAFETY: seam reborrow of the impl's owned world island (STATE-D6);
        // single-threaded module, no live sibling borrow across this call.
        let mut ctx = GameContext {
            world: unsafe { &mut *self.world },
            engine: self.engine,
        };
        G_FlyVehicleSurfaceDestruction(
            &mut ctx,
            EntityId(entNum as u32),
            trace,
            magnitude,
            force,
        );
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
        // `G_SetAnim` takes a `gentity_t*` + the module `GameContext`; resolve
        // `entNum`, rebuild `ctx`, and pass the bg-owned `ucmd` through.
        // Source: `g_utils.c` (`G_SetAnim`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            let ent_id_2 = ctx.entity_id_of(ent).unwrap();
            G_SetAnim(
                &mut ctx,
                ent_id_2,
                ucmd,
                setAnimParts,
                anim,
                setAnimFlags,
                blendTime,
            );
        }
    }
    fn npc_set_anim(&mut self, entNum: c_int, type_: c_int, anim: c_int, priority: c_int) {
        // Raven `NPC_SetAnim(ent, setAnimParts=type, anim, setFlags=priority)`;
        // resolve `entNum` and rebuild `ctx`. Source: `npc.cpp` (`NPC_SetAnim`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            let ent_id_3 = ctx.entity_id_of(ent).unwrap();
            NPC_SetAnim(&mut ctx, ent_id_3, type_, anim, priority);
        }
    }
    fn wp_get_vehicle_cam_pos(
        &mut self,
        vehEntNum: c_int,
        pilotEntNum: c_int,
        camPos: *mut vec3_t,
    ) {
        // Resolves the vehicle + pilot nums against the world arena, rebuilds the
        // module `GameContext`, and delegates to the ported `WP_GetVehicleCamPos`.
        // Source: `oracle/codemp/game/g_weapon.c:3961-4020`.
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pilot = &mut (*self.world).g_entities[pilotEntNum as usize] as *mut gentity_t;
            let ent_id_4 = ctx.entity_id_of(ent).unwrap();
            let pilot_id = ctx.entity_id_of(pilot).unwrap();
            WP_GetVehicleCamPos(&mut ctx, ent_id_4, pilot_id, &mut *camPos);
        }
    }
    fn can_be_enemy(&mut self, entNum: c_int, otherNum: c_int) -> qboolean {
        // Raven `G_CanBeEnemy(self, enemy)`; resolve both nums.
        // Source: `oracle/codemp/game/w_saber.c` (`G_CanBeEnemy`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let self_ = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            let enemy = &mut (*self.world).g_entities[otherNum as usize] as *mut gentity_t;
            let self_id = ctx.entity_id_of(self_).unwrap();
            let enemy_id = ctx.entity_id_of(enemy).unwrap();
            G_CanBeEnemy(&mut ctx, self_id, enemy_id) as qboolean
        }
    }
    fn get_time(&self) -> c_int {
        // Raven `level.time`. Source: `oracle/codemp/game/g_local.h`.
        unsafe { (*self.world).level.time }
    }
    fn try_grapple(&mut self, entNum: c_int) -> qboolean {
        // Resolves `entNum` against the world arena, rebuilds the module
        // `GameContext` from the handles this impl holds, and delegates to the
        // ported `TryGrapple` body.
        // Source: `oracle/codemp/game/g_cmds.c:3148-3191` (`TryGrapple`).
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            let ent_id = ctx.entity_id_of(ent).unwrap();
            TryGrapple(&mut ctx, ent_id)
        }
    }
    fn q3_set_parm(&mut self, entID: c_int, parmNum: c_int, parmValue: &str) {
        // `Q3_SetParm` takes `entID` as a raw index and resolves it internally; it
        // still takes a `*const c_char`, so re-encode the `&str` for the call.
        // Source: `oracle/codemp/game/g_ICARUScb.c` (`Q3_SetParm`).
        // SAFETY: seam reborrow of the impl's owned world island (STATE-D6);
        // single-threaded module, no live sibling borrow across this call.
        let mut ctx = GameContext {
            world: unsafe { &mut *self.world },
            engine: self.engine,
        };
        Q3_SetParm(&mut ctx, entID, parmNum, cstr(parmValue).as_ptr());
    }
    fn board_vehicle(&mut self, vehEntNum: c_int, entNum: c_int) -> qboolean {
        // Resolves `vehEntNum`->`m_pVehicle` and `entNum`->`bgEntity_t` against
        // the world arena, rebuilds the module `GameContext` from the handles this
        // impl holds, and delegates to `crate::veh_dispatch::board` (now that the
        // dispatch chain threads `ctx`). Source: `oracle/codemp/game/g_vehicles.c:630`.
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle;
            let pEnt =
                &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t as *mut bgEntity_t;
            veh_dispatch::board(&mut ctx, pVeh, pEnt)
        }
    }
    fn update_vehicle(&mut self, vehEntNum: c_int, ucmd: *const usercmd_t) {
        // Resolve `vehEntNum`->`m_pVehicle`, rebuild `ctx`, delegate to the
        // generic-base `Update` dispatch. Source: `bg_pmove.c:10919-10944`.
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle;
            veh_dispatch::update(&mut ctx, pVeh, ucmd);
        }
    }
    fn pm_animate_vehicle(&mut self, vehEntNum: c_int) {
        // Resolve `vehEntNum`->`m_pVehicle`, rebuild `ctx`, delegate to the
        // generic-base `Animate` dispatch. Source: `bg_pmove.c:10921-10945`.
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle;
            veh_dispatch::animate(&mut ctx, pVeh);
        }
    }
    fn update_rider(&mut self, vehEntNum: c_int, riderEntNum: c_int, ucmd: *mut usercmd_t) {
        // Resolve the vehicle + rider. Driver path: bg passed `&pVeh->m_ucmd`.
        // Passenger path: bg passes null, so guard `inuse && client` and use the
        // rider's own `client->pers.cmd` (game-side). Source: `bg_pmove.c:10947-10961`.
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle;
            let rider = &mut (*self.world).g_entities[riderEntNum as usize] as *mut gentity_t;
            let cmd = if ucmd.is_null() {
                if (*rider).inuse == qfalse || (*rider).client.is_null() {
                    return;
                }
                &mut (*((*rider).client)).pers.cmd as *mut usercmd_t
            } else {
                ucmd
            };
            veh_dispatch::update_rider(&mut ctx, pVeh, rider as *mut bgEntity_t, cmd);
        }
    }
    fn attach_riders(&mut self, vehEntNum: c_int) {
        // Resolve `vehEntNum`->`m_pVehicle`, rebuild `ctx`, delegate to the
        // generic-base `AttachRiders` dispatch. Source: `bg_pmove.c:11146-11149`.
        unsafe {
            let mut ctx = GameContext {
                world: &mut *self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle;
            veh_dispatch::attach_riders(&mut ctx, pVeh);
        }
    }
    fn my_saber(&mut self, client_num: c_int, saber_num: c_int) -> *mut saberInfo_t {
        // Reproduces the QAGAME `BG_MySaber` body over the game arena: NULL
        // unless the client is in use, has a `client`, and that saber has a
        // model. Source: `oracle/codemp/game/bg_saber.c:4100-4141`.
        unsafe {
            let ent = &(*self.world).g_entities[client_num as usize] as *const gentity_t;
            if (*ent).inuse != 0 && !(*ent).client.is_null() {
                let saber = &mut (*((*ent).client)).saber[saber_num as usize] as *mut saberInfo_t;
                if (*saber).model[0] == 0 {
                    return core::ptr::null_mut();
                }
                return saber;
            }
            core::ptr::null_mut()
        }
    }
    fn suspended_vehicle_boardable(&self, veh_ent_num: c_int) -> qboolean {
        // `PM_GroundTrace` suspended-vehicle gate. Source: `bg_pmove.c`.
        unsafe {
            let ent = &(*self.world).g_entities[veh_ent_num as usize] as *const gentity_t;
            (!(*ent).client.is_null()
                && (*((*ent).client)).ps.m_iVehicleNum == 0
                && (*ent).spawnflags & 2 != 0) as qboolean
        }
    }
    fn landed_vehicle_boardable(
        &self,
        tr_ent_num: c_int,
        self_num: c_int,
        gametype: c_int,
    ) -> qboolean {
        // `PM_CrashLand` landed-vehicle gate, verbatim over the game arena;
        // `gametype` is read bg-side (`pm->gametype`) and passed in. Source:
        // `oracle/codemp/game/bg_pmove.c` (PM_CrashLand landed-vehicle board).
        unsafe {
            let trEnt = &(*self.world).g_entities[tr_ent_num as usize] as *const gentity_t;
            let veh = (*trEnt).m_pVehicle;
            if (*trEnt).inuse != 0
                && !(*trEnt).client.is_null()
                && (*trEnt).s.eType == entityType_t::ET_NPC as c_int
                && (*trEnt).s.NPC_class == CLASS_VEHICLE as c_int
                && (*((*trEnt).client)).ps.m_iVehicleNum == 0
                && !veh.is_null()
                && (*(*veh).m_pVehicleInfo).r#type as c_int != vehicleType_t::VH_WALKER as c_int
                && (*(*veh).m_pVehicleInfo).r#type as c_int != vehicleType_t::VH_FIGHTER as c_int
            {
                let servEnt = &(*self.world).g_entities[self_num as usize] as *const gentity_t;
                if gametype < GT_TEAM as c_int
                    || (*trEnt).alliedTeam as c_int == 0
                    || (*trEnt).alliedTeam as c_int
                        == (*((*servEnt).client)).sess.sessionTeam as c_int
                {
                    return qtrue;
                }
            }
            qfalse
        }
    }
    fn set_solid_hack(&mut self, ent_num: c_int) {
        // `PM_AdjustBBox` solidHack. Source: `bg_pmove.c` (PM_AdjustBBox).
        unsafe {
            let time = (*self.world).level.time;
            let ent = &(*self.world).g_entities[ent_num as usize] as *const gentity_t;
            if (*ent).inuse != 0 && !(*ent).client.is_null() {
                (*((*ent).client)).solidHack = time + 200;
            }
        }
    }
    fn humanoid_inuse_client(&self, ent_num: c_int) -> qboolean {
        // `PM_Weapon` NPC-no-weapon humanoid test. Source: `bg_pmove.c`.
        unsafe {
            let ent = &(*self.world).g_entities[ent_num as usize] as *const gentity_t;
            ((*ent).inuse != 0 && !(*ent).client.is_null() && (*ent).localAnimIndex == 0)
                as qboolean
        }
    }
    fn fighter_not_suspended(&self, ent_num: c_int) -> qboolean {
        // `PM_VehicleImpact` turn-away suspended gate.
        // Source: `oracle/codemp/game/bg_slidemove.c:313-398`.
        unsafe { ((*self.world).g_entities[ent_num as usize].spawnflags & 2 == 0) as qboolean }
    }
    fn set_other_killer(
        &mut self,
        ent_num: c_int,
        mod_: c_int,
        veh_weapon: c_int,
        weapon_type: c_int,
    ) {
        // `PM_VehicleImpact` knockdown: the three `gclient_t` killer-credit
        // fields (not on `ps`). Source: `oracle/codemp/game/bg_slidemove.c:402-542`.
        unsafe {
            let client = (*self.world).g_entities[ent_num as usize].client;
            (*client).otherKillerMOD = mod_;
            (*client).otherKillerVehWeapon = veh_weapon;
            (*client).otherKillerWeaponType = weapon_type;
        }
    }
    fn entity_inuse(&self, ent_num: c_int) -> qboolean {
        // `PM_VehicleImpact` hit-entity `inuse` read.
        // Source: `oracle/codemp/game/bg_slidemove.c:49-557`.
        unsafe { (*self.world).g_entities[ent_num as usize].inuse }
    }
    fn entity_spawnflags(&self, ent_num: c_int) -> c_int {
        // `PM_VehicleImpact` hit-entity `spawnflags` read.
        // Source: `oracle/codemp/game/bg_slidemove.c:49-557`.
        unsafe { (*self.world).g_entities[ent_num as usize].spawnflags }
    }
    fn entity_takedamage(&self, ent_num: c_int) -> qboolean {
        // `PM_VehicleImpact` hit-entity `takedamage` read.
        // Source: `oracle/codemp/game/bg_slidemove.c:49-557`.
        unsafe { (*self.world).g_entities[ent_num as usize].takedamage }
    }
    fn fighter_is_landed(&self, veh_ent_num: c_int) -> qboolean {
        // Reproduces the inline `FighterIsLanded(hitEnt->m_pVehicle,
        // hitEnt->playerState)` over the game arena; the bg-side
        // `!playerState.is_null()` gate short-circuits before this call.
        // Source: `oracle/codemp/game/bg_slidemove.c:313-398`;
        // `oracle/codemp/game/FighterNPC.c:300-308`.
        unsafe {
            let ent = &(*self.world).g_entities[veh_ent_num as usize] as *const gentity_t;
            FighterIsLanded((*ent).m_pVehicle, (*ent).playerState)
        }
    }

    // ---------------------------------------------------------------------
    // DEC-36 D5 — per-module bg arms, QAGAME side.
    //
    // Each method below reproduces this module's `#ifdef` arm exactly. Where
    // Raven's QAGAME arm is empty — a commented-out call, or an `#ifdef` chain
    // with no QAGAME branch — the body is empty and the destination slot is
    // left untouched, matching the game DLL byte for byte.
    // ---------------------------------------------------------------------

    fn veh_field_model(&mut self, value: &str, dest: *mut c_int) {
        // QAGAME: `*(int *)(b+ofs) = G_ModelIndex( value );`
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:231-237,905-911`
        unsafe { *dest = self.model_index(value) }
    }
    fn veh_field_model_client(&mut self, value: &str, dest: *mut c_int) {
        // QAGAME: the `G_ModelIndex` store is commented out under `#elif QAGAME`
        // — the game module writes nothing.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:238-246,912-920`
    }
    fn veh_field_effect(&mut self, value: &str, dest: *mut c_int) {
        // QAGAME: `*(int *)(b+ofs) = G_EffectIndex( value );`
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:247-253,921-927`
        unsafe { *dest = self.effect_index(value) }
    }
    fn veh_field_effect_client(&mut self, value: &str, dest: *mut c_int) {
        // QAGAME: the `G_EffectIndex` store is commented out under `#elif QAGAME`
        // — the game module writes nothing.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:254-262,928-936`
    }
    fn veh_field_shader(&mut self, value: &str, dest: *mut c_int) {
        // The `#ifdef WE_ARE_IN_THE_UI`/`#elif CGAME` chain has no QAGAME arm —
        // the game module writes nothing.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:263-269,937-943`
    }
    fn veh_field_shader_nomip(&mut self, value: &str, dest: *mut c_int) {
        // Guarded `#ifndef QAGAME` — the game module writes nothing.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:270-274,944-948`
    }
    fn veh_field_sound(&mut self, value: &str, dest: *mut c_int) {
        // QAGAME: `*(int *)(b+ofs) = G_SoundIndex( value );`
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:275-281,949-955`
        unsafe { *dest = self.sound_index(value) }
    }
    fn veh_field_sound_client(&mut self, value: &str, dest: *mut c_int) {
        // QAGAME: the `G_SoundIndex` store is commented out under `#elif QAGAME`
        // — the game module writes nothing.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:282-290,956-964`
    }
    fn veh_weapon_homing_precache(&mut self) {
        // Raven: "Hmm, no need fo have server register this, is there?" — both
        // `G_SoundIndex` calls are commented out; the game registers nothing.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:390-409`
    }
    fn vehicle_skin_precache(&mut self, model: &str, skin: &str) {
        // Guarded `#ifndef QAGAME` — the game registers no vehicle skin.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1293-1299`
    }
    fn vehicle_load_precache(&mut self, hideRider: qboolean) {
        // QAGAME arm; `hideRider` gates cgame-only radar shaders.
        // Source: `oracle/codemp/game/bg_vehicleLoad.c:1336-1359`
        self.effect_index("volumetric/black_smoke");
        self.effect_index("ships/fire");
        self.sound_index("sound/vehicles/common/release.wav");
    }
    fn siege_class_ui_portrait(&mut self, uishader: &str) -> (c_int, String) {
        // QAGAME: `uiPortraitShader = 0` and `memset(uiPortrait, 0, ...)`.
        // Source: `oracle/codemp/game/bg_saga.c:975-988`
        (0, String::new())
    }
    fn siege_class_shader(&mut self, class_shader: &str, class_name: &str) -> (c_int, bool) {
        // QAGAME: `classShader = 0` — the shader is never registered server-side.
        // The `#ifdef QAGAME` arm has no `else` gate, so the class-determination
        // block runs unconditionally.
        // Source: `oracle/codemp/game/bg_saga.c:994-1039`
        (0, true)
    }
}
