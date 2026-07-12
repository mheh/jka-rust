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
        // from BgTraps methods (seam boundary).
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
        // Mechanical delegation. Raven: `trap_Trace` (`G_TRACE`).
        use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
        crate::trap::Trace(
            self.ctx.engine,
            GTraceArgs::new(results, start, mins, maxs, end, passEntityNum, contentMask),
        )
    }

    fn pointcontents(&self, point: *const vec3_t, passEntityNum: c_int) -> c_int {
        // Real delegation — the pmove slice's PM_SetWaterLevel drives this.
        // Raven: `trap_PointContents` (`G_POINT_CONTENTS`).
        use mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs;
        crate::trap::PointContents(
            self.ctx.engine,
            GPointContentsArgs::new(point, passEntityNum),
        )
    }

    fn fs_fopen(&self, qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
        // Raven: `trap_FS_FOpenFile` (`G_FS_FOPEN_FILE`). `Args::new` is `unsafe`
        // (raw out-param `f`); the caller guarantees `f` is valid.
        use mp_abi::game::syscalls::G_FS_FOPEN_FILE::GFsFopenFileArgs;
        let qpath = unsafe { std::ffi::CStr::from_ptr(qpath) }.to_owned();
        crate::trap::FS_FOpenFile(self.ctx.engine, unsafe {
            GFsFopenFileArgs::new(qpath, f, mode)
        })
    }
    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t) {
        // Mechanical delegation — matches the proven `pointcontents`
        // shape. Raven: `trap_FS_Read` (`G_FS_READ`).
        use mp_abi::game::syscalls::G_FS_READ::GFsReadArgs;
        crate::trap::FS_Read(self.ctx.engine, GFsReadArgs::new(buffer as *mut u8, len, f))
    }
    fn fs_write(&self, buffer: *const c_void, len: c_int, f: fileHandle_t) {
        // Raven: `trap_FS_Write` (`G_FS_WRITE`).
        use mp_abi::game::syscalls::G_FS_WRITE::GFsWriteArgs;
        crate::trap::FS_Write(
            self.ctx.engine,
            GFsWriteArgs::new(buffer as *const u8, len, f),
        )
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
        // Raven: `trap_FS_GetFileList` (`G_FS_GETFILELIST`).
        use mp_abi::game::syscalls::G_FS_GETFILELIST::GFsGetfilelistArgs;
        let path = unsafe { std::ffi::CStr::from_ptr(path) }.to_owned();
        let extension = unsafe { std::ffi::CStr::from_ptr(extension) }.to_owned();
        crate::trap::FS_GetFileList(
            self.ctx.engine,
            GFsGetfilelistArgs::new(path, extension, listbuf as *mut u8, bufsize),
        )
    }

    fn r_register_skin(&self, name: *const c_char) -> qhandle_t {
        // Mechanical delegation, matching `g2api_add_bolt`'s
        // CString-conversion shape. Raven: `trap_R_RegisterSkin` (`G_R_REGISTERSKIN`).
        let name = unsafe { std::ffi::CStr::from_ptr(name) }.to_owned();
        crate::trap::R_RegisterSkin(
            self.ctx.engine,
            mp_abi::game::syscalls::G_R_REGISTERSKIN::GRRegisterskinArgs::new(name),
        )
    }

    fn g2api_init_ghoul2_model(
        &self,
        ghoul2Ptr: *mut *mut c_void,
        fileName: *const c_char,
        modelIndex: c_int,
        customSkin: qhandle_t,
        customShader: qhandle_t,
        modelFlags: c_int,
        lodBias: c_int,
    ) -> c_int {
        // Mechanical delegation. Raven: `trap_G2API_InitGhoul2Model`
        // (`G_G2_INITGHOUL2MODEL`).
        let file_name = unsafe { std::ffi::CStr::from_ptr(fileName) }.to_owned();
        crate::trap::G2API_InitGhoul2Model(
            self.ctx.engine,
            mp_abi::game::syscalls::G_G2_INITGHOUL2MODEL::GG2Initghoul2ModelArgs::new(
                ghoul2Ptr,
                file_name,
                modelIndex,
                customSkin,
                customShader,
                modelFlags,
                lodBias,
            ),
        )
    }
    fn g2api_clean_ghoul2_models(&self, ghoul2Ptr: *mut *mut c_void) {
        // Mechanical delegation. Raven: `trap_G2API_CleanGhoul2Models`
        // (`G_G2_CLEANMODELS`).
        crate::trap::G2API_CleanGhoul2Models(
            self.ctx.engine,
            mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs::new(ghoul2Ptr),
        )
    }
    fn g2api_add_bolt(
        &self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: *const c_char,
    ) -> c_int {
        // Real delegation to the already-wired `trap_G2API_AddBolt` seam
        // (`G_G2_ADDBOLT`); bg-visible callers (e.g. `AttachRidersGeneric`)
        // only carry `&dyn BgTraps`, not `&Engine`.
        let bone_name = unsafe { std::ffi::CStr::from_ptr(boneName) }.to_owned();
        crate::trap::G2API_AddBolt(
            self.ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                ghoul2, modelIndex, bone_name,
            ),
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
        crate::trap::G2API_GetBoltMatrix_NoReconstruct(
            self.ctx.engine,
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
        crate::trap::G2API_GetBoltMatrix_NoRecNoRot(
            self.ctx.engine,
            GG2GetboltNorecNorotArgs::new(
                ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList, scale,
            ),
        )
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
        // Raven: `trap_G2API_SetBoneAngles` (`G_G2_ANGLEOVERRIDE`).
        use mp_abi::game::syscalls::G_G2_ANGLEOVERRIDE::GG2AngleoverrideArgs;
        let bone_name = unsafe { std::ffi::CStr::from_ptr(boneName) }.to_owned();
        crate::trap::G2API_SetBoneAngles(
            self.ctx.engine,
            GG2AngleoverrideArgs::new(
                ghoul2,
                modelIndex,
                bone_name,
                angles,
                flags,
                up,
                right,
                forward,
                modelList,
                blendTime,
                currentTime,
            ),
        )
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
        // Raven: `trap_G2API_SetBoneAnim` (`G_G2_PLAYANIM`).
        use mp_abi::game::syscalls::G_G2_PLAYANIM::GG2PlayanimArgs;
        crate::trap::G2API_SetBoneAnim(
            self.ctx.engine,
            GG2PlayanimArgs::new(
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
            ),
        )
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
        // Raven: `trap_G2API_GetBoneAnim` (`G_G2_GETBONEANIM`).
        use mp_abi::game::syscalls::G_G2_GETBONEANIM::GG2GetboneanimArgs;
        let bone_name = unsafe { std::ffi::CStr::from_ptr(boneName) }.to_owned();
        crate::trap::G2API_GetBoneAnim(
            self.ctx.engine,
            GG2GetboneanimArgs::new(
                ghoul2,
                bone_name,
                currentTime,
                currentFrame,
                startFrame,
                endFrame,
                flags,
                animSpeed,
                modelList,
                modelIndex,
            ),
        )
    }
    fn g2api_set_rag_doll(&self, ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) {
        // Raven: `trap_G2API_SetRagDoll` (`G_G2_SETRAGDOLL`).
        use mp_abi::game::syscalls::G_G2_SETRAGDOLL::GG2SetragdollArgs;
        crate::trap::G2API_SetRagDoll(self.ctx.engine, GG2SetragdollArgs::new(ghoul2, params))
    }
    fn g2api_animate_g2_models(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    ) {
        // Raven: `trap_G2API_AnimateG2Models` (`G_G2_ANIMATEG2MODELS`).
        use mp_abi::game::syscalls::G_G2_ANIMATEG2MODELS::GG2Animateg2ModelsArgs;
        crate::trap::G2API_AnimateG2Models(
            self.ctx.engine,
            GG2Animateg2ModelsArgs::new(ghoul2, time, params),
        )
    }
    fn g2api_set_bone_ik_state(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        boneName: *const c_char,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean {
        // Raven: `trap_G2API_SetBoneIKState` (`G_G2_SETBONEIKSTATE`).
        use mp_abi::game::syscalls::G_G2_SETBONEIKSTATE::GG2SetboneikstateArgs;
        let bone_name = unsafe { std::ffi::CStr::from_ptr(boneName) }.to_owned();
        crate::trap::G2API_SetBoneIKState(
            self.ctx.engine,
            GG2SetboneikstateArgs::new(ghoul2, time, bone_name, ikState, params),
        )
    }
    fn g2api_ik_move(
        &self,
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> qboolean {
        // Raven: `trap_G2API_IKMove` (`G_G2_IKMOVE`).
        use mp_abi::game::syscalls::G_G2_IKMOVE::GG2IkmoveArgs;
        crate::trap::G2API_IKMove(self.ctx.engine, GG2IkmoveArgs::new(ghoul2, time, params))
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
        crate::trap::SnapVector(self.ctx.engine, GSnapvectorArgs::new(v as *mut vec3_t))
    }
    fn cvar_register(
        &self,
        cvar: *mut vmCvar_t,
        var_name: *const c_char,
        value: *const c_char,
        flags: c_int,
    ) {
        // Raven: `trap_Cvar_Register` (`G_CVAR_REGISTER`). `Args::new` wants owned
        // `CString`s (`impl Into<CString>`); copy the borrowed C strings.
        use mp_abi::game::syscalls::G_CVAR_REGISTER::GCvarRegisterArgs;
        let var_name = unsafe { std::ffi::CStr::from_ptr(var_name) }.to_owned();
        let value = unsafe { std::ffi::CStr::from_ptr(value) }.to_owned();
        crate::trap::Cvar_Register(
            self.ctx.engine,
            GCvarRegisterArgs::new(cvar, var_name, value, flags),
        )
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
        // Resolves the entity nums against the world arena, rebuilds the module
        // `GameContext`, and delegates to the ported `G_Damage`. `dir`/`point` are
        // bg-visible raw pointers; the ported body takes `dir: Option<&mut vec3_t>`
        // and `point` by value. Every bg caller passes `dir = null` and a non-null
        // `point`. Source: `oracle/codemp/game/g_combat.c` (`G_Damage`).
        unsafe {
            let ctx = GameContext {
                world: self.world,
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
            crate::g_combat::G_Damage(
                ctx,
                ctx.entity_id_of(targ),
                ctx.entity_id_of(inflictor),
                ctx.entity_id_of(attacker),
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
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let pEnt = &mut (*self.world).g_entities[targNum as usize] as *mut gentity_t;
            let pVehEnt = &mut (*self.world).g_entities[inflictorNum as usize] as *mut gentity_t;
            let attacker = &mut (*self.world).g_entities[attackerNum as usize] as *mut gentity_t;
            crate::g_combat::G_DamageFromKiller(
                ctx,
                ctx.entity_id_of(pEnt),
                ctx.entity_id_of(pVehEnt),
                ctx.entity_id_of(attacker),
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
            crate::g_utils::G_AddEvent(ent, event, eventParm);
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
        let ctx = GameContext {
            world: self.world,
            engine: self.engine,
        };
        crate::g_mem::G_Alloc(ctx, size)
    }
    fn new_string(&mut self, string: *const c_char) -> *mut c_char {
        // `G_NewString` copies into the game pool via `ctx.world`.
        // Source: `oracle/codemp/game/g_spawn.c` (`G_NewString`).
        let ctx = GameContext {
            world: self.world,
            engine: self.engine,
        };
        crate::g_spawn::G_NewString(ctx, string)
    }
    fn play_effect(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) {
        // `G_PlayEffect` is ctx-free and takes `org`/`ang` by value; the spawned
        // temp-entity return is discarded (as at the bg call sites).
        // Source: `oracle/codemp/game/g_utils.c` (`G_PlayEffect`).
        unsafe {
            crate::g_utils::G_PlayEffect(fxID, *org, *ang);
        }
    }
    fn play_effect_id(&mut self, fxID: c_int, org: *const vec3_t, ang: *const vec3_t) -> c_int {
        // `G_PlayEffectID` returns the spawned temp-entity; the bg-visible upcall
        // yields its entity number (`ENTITYNUM_NONE` when none was spawned).
        // Source: `oracle/codemp/game/g_utils.c` (`G_PlayEffectID`).
        unsafe {
            let te = crate::g_utils::G_PlayEffectID(fxID, *org, *ang);
            if te.is_null() {
                ENTITYNUM_NONE
            } else {
                (*te).s.number
            }
        }
    }
    fn sound_index(&mut self, name: *const c_char) -> c_int {
        // ctx-free configstring lookup. Source: `g_utils.c` (`G_SoundIndex`).
        crate::g_utils::G_SoundIndex(name)
    }
    fn model_index(&mut self, name: *const c_char) -> c_int {
        // ctx-free configstring lookup. Source: `g_utils.c` (`G_ModelIndex`).
        crate::g_utils::G_ModelIndex(name)
    }
    fn effect_index(&mut self, name: *const c_char) -> c_int {
        // ctx-free configstring lookup. Source: `g_utils.c` (`G_EffectIndex`).
        crate::g_utils::G_EffectIndex(name)
    }
    fn cheap_weapon_fire(&mut self, entNum: c_int, weapon: c_int) {
        // Raven `G_CheapWeaponFire(entNum, ev)` takes the entity number directly.
        // Source: `oracle/codemp/game/g_active.c` (`G_CheapWeaponFire`).
        let ctx = GameContext {
            world: self.world,
            engine: self.engine,
        };
        crate::g_active::G_CheapWeaponFire(ctx, entNum, weapon);
    }
    fn client_check_impact_bbrush(&mut self, entNum: c_int, impactNum: c_int) {
        // Raven `Client_CheckImpactBBrush(self, other)`; resolve both nums.
        // Source: `oracle/codemp/game/g_active.c` (`Client_CheckImpactBBrush`).
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let self_ = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            let other = &mut (*self.world).g_entities[impactNum as usize] as *mut gentity_t;
            crate::g_active::Client_CheckImpactBBrush(
                ctx,
                ctx.entity_id_of(self_),
                ctx.entity_id_of(other),
            );
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
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let veh = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            crate::g_vehicles::G_FlyVehicleSurfaceDestruction(ctx, veh, trace, magnitude, force);
        }
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
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            crate::g_utils::G_SetAnim(ctx, ent, ucmd, setAnimParts, anim, setAnimFlags, blendTime);
        }
    }
    fn npc_set_anim(&mut self, entNum: c_int, type_: c_int, anim: c_int, priority: c_int) {
        // Raven `NPC_SetAnim(ent, setAnimParts=type, anim, setFlags=priority)`;
        // resolve `entNum` and rebuild `ctx`. Source: `npc.cpp` (`NPC_SetAnim`).
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            crate::npc_c::NPC_SetAnim(ctx, ent, type_, anim, priority);
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
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pilot = &mut (*self.world).g_entities[pilotEntNum as usize] as *mut gentity_t;
            crate::g_weapon::WP_GetVehicleCamPos(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                ctx.entity_id_of(pilot).unwrap(),
                &mut *camPos,
            );
        }
    }
    fn can_be_enemy(&mut self, entNum: c_int, otherNum: c_int) -> qboolean {
        // Raven `G_CanBeEnemy(self, enemy)`; resolve both nums.
        // Source: `oracle/codemp/game/w_saber.c` (`G_CanBeEnemy`).
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let self_ = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            let enemy = &mut (*self.world).g_entities[otherNum as usize] as *mut gentity_t;
            crate::w_saber::G_CanBeEnemy(ctx, self_, enemy)
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
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let ent = &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t;
            crate::g_cmds::TryGrapple(ctx, ent)
        }
    }
    fn q3_set_parm(&mut self, entID: c_int, parmNum: c_int, parmValue: *const c_char) {
        // `Q3_SetParm` takes `entID` as a raw index and resolves it internally.
        // Source: `oracle/codemp/game/g_ICARUScb.c` (`Q3_SetParm`).
        let ctx = GameContext {
            world: self.world,
            engine: self.engine,
        };
        crate::g_ICARUScb::Q3_SetParm(ctx, entID, parmNum, parmValue);
    }
    fn board_vehicle(&mut self, vehEntNum: c_int, entNum: c_int) -> qboolean {
        // Resolves `vehEntNum`->`m_pVehicle` and `entNum`->`bgEntity_t` against
        // the world arena, rebuilds the module `GameContext` from the handles this
        // impl holds, and delegates to `crate::veh_dispatch::board` (now that the
        // dispatch chain threads `ctx`). Source: `oracle/codemp/game/g_vehicles.c:630`.
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle as *mut Vehicle_t;
            let pEnt =
                &mut (*self.world).g_entities[entNum as usize] as *mut gentity_t as *mut bgEntity_t;
            crate::veh_dispatch::board(ctx, pVeh, pEnt)
        }
    }
    fn update_vehicle(&mut self, vehEntNum: c_int, ucmd: *const usercmd_t) {
        // Resolve `vehEntNum`->`m_pVehicle`, rebuild `ctx`, delegate to the
        // generic-base `Update` dispatch. Source: `bg_pmove.c:10919-10944`.
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle as *mut Vehicle_t;
            crate::veh_dispatch::update(ctx, pVeh, ucmd);
        }
    }
    fn pm_animate_vehicle(&mut self, vehEntNum: c_int) {
        // Resolve `vehEntNum`->`m_pVehicle`, rebuild `ctx`, delegate to the
        // generic-base `Animate` dispatch. Source: `bg_pmove.c:10921-10945`.
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle as *mut Vehicle_t;
            crate::veh_dispatch::animate(ctx, pVeh);
        }
    }
    fn update_rider(&mut self, vehEntNum: c_int, riderEntNum: c_int, ucmd: *mut usercmd_t) {
        // Resolve the vehicle + rider. Driver path: bg passed `&pVeh->m_ucmd`.
        // Passenger path: bg passes null, so guard `inuse && client` and use the
        // rider's own `client->pers.cmd` (game-side). Source: `bg_pmove.c:10947-10961`.
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle as *mut Vehicle_t;
            let rider = &mut (*self.world).g_entities[riderEntNum as usize] as *mut gentity_t;
            let cmd = if ucmd.is_null() {
                if (*rider).inuse == qfalse || (*rider).client.is_null() {
                    return;
                }
                &mut (*((*rider).client as *mut gclient_t)).pers.cmd as *mut usercmd_t
            } else {
                ucmd
            };
            crate::veh_dispatch::update_rider(ctx, pVeh, rider as *mut bgEntity_t, cmd);
        }
    }
    fn attach_riders(&mut self, vehEntNum: c_int) {
        // Resolve `vehEntNum`->`m_pVehicle`, rebuild `ctx`, delegate to the
        // generic-base `AttachRiders` dispatch. Source: `bg_pmove.c:11146-11149`.
        unsafe {
            let ctx = GameContext {
                world: self.world,
                engine: self.engine,
            };
            let vehEnt = &mut (*self.world).g_entities[vehEntNum as usize] as *mut gentity_t;
            let pVeh = (*vehEnt).m_pVehicle as *mut Vehicle_t;
            crate::veh_dispatch::attach_riders(ctx, pVeh);
        }
    }
}
