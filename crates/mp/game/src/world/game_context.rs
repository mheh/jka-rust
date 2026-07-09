//! `GameContext` — the module-side `Dispatch<C>` receiver (SEAM-Q12 RESOLVED,
//! round-4; SUPERSEDES the round-3 `WorldPtr`).

use mp_engine_select::Engine;

use super::game_world::GameWorld;

/// The copyable `Dispatch<C>` receiver each `vmMain` command routes through
/// (SEAM-Q12 resolved 2026-07-03). Defined in `mp_game`; `Engine` is the
/// `mp_engine_select` module-side transport alias (NOT `mp_engine_core::Engine`).
/// `vmMain` constructs one per call — a plain struct literal built by the shell
/// from its `WORLD` + `ENGINE.get()` (SEAM-D1); fields are `pub` per the
/// round-5 resolution (a `Copy` struct of raw pointers has no invariant to
/// protect; the `WorldPtr` precedent, STATE-D8). Each `impl Dispatch<C> for
/// GameContext` unpacks `world` via STATE-D6 leaf reborrows and threads
/// `engine` into the logic fns' `trap::X(engine, …)` call sites (oracle syscall
/// order). The `&Engine` channel is the receiver's own field — no `dispatch`
/// parameter is added, SEAM-D8 stays untouched; the orphan rule is satisfied
/// because `GameContext` and its impls both live in `mp_game`.
///
/// Source: `docs/architecture/engine-seam.md` § inbound dual (SEAM-Q12 amendment 2026-07-03).
#[derive(Clone, Copy)]
pub struct GameContext<'e> {
    pub world: *mut GameWorld,
    pub engine: &'e Engine,
}

// The per-command `impl Dispatch<C> for GameContext` blocks colocate here
// (round-6 pinning: thin adapters only; per-command logic stays
// one-fn-per-file). Each unpacks `self.world` via STATE-D6 leaf reborrows and
// threads `self.engine` into the ported logic fns.

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::gentity::gentity_t;
use mp_qshared::shared::{qboolean, vec3_t};

use crate::prelude::*;

use mp_abi::game::vmcalls::BOTAI_START_FRAME::{BotAiStartFrame, BotAiStartFrameArgs};
use mp_abi::game::vmcalls::GAME_CLIENT_BEGIN::{GameClientBegin, GameClientBeginArgs};
use mp_abi::game::vmcalls::GAME_CLIENT_COMMAND::{GameClientCommand, GameClientCommandArgs};
use mp_abi::game::vmcalls::GAME_CLIENT_CONNECT::{GameClientConnect, GameClientConnectArgs};
use mp_abi::game::vmcalls::GAME_CLIENT_DISCONNECT::{
    GameClientDisconnect, GameClientDisconnectArgs,
};
use mp_abi::game::vmcalls::GAME_CLIENT_THINK::{GameClientThink, GameClientThinkArgs};
use mp_abi::game::vmcalls::GAME_CLIENT_USERINFO_CHANGED::{
    GameClientUserinfoChanged, GameClientUserinfoChangedArgs,
};
use mp_abi::game::vmcalls::GAME_CONSOLE_COMMAND::GameConsoleCommand;
use mp_abi::game::vmcalls::GAME_GETITEMINDEXBYTAG::{
    GameGetitemindexbytag, GameGetitemindexbytagArgs,
};
use mp_abi::game::vmcalls::GAME_INIT::{GameInit, GameInitArgs};
use mp_abi::game::vmcalls::GAME_NAV_CHECKNODEFAILEDFORENT::{
    GameNavChecknodefailedforent, GameNavChecknodefailedforentArgs,
};
use mp_abi::game::vmcalls::GAME_NAV_CLEARLOS::{GameNavClearlos, GameNavClearlosArgs};
use mp_abi::game::vmcalls::GAME_NAV_CLEARPATHBETWEENPOINTS::{
    GameNavClearpathbetweenpoints, GameNavClearpathbetweenpointsArgs,
};
use mp_abi::game::vmcalls::GAME_NAV_CLEARPATHTOPOINT::{
    GameNavClearpathtopoint, GameNavClearpathtopointArgs,
};
use mp_abi::game::vmcalls::GAME_NAV_ENTISBREAKABLE::{
    GameNavEntIsBreakable, GameNavEntIsBreakableArgs,
};
use mp_abi::game::vmcalls::GAME_NAV_ENTISDOOR::{GameNavEntIsDoor, GameNavEntIsDoorArgs};
use mp_abi::game::vmcalls::GAME_NAV_ENTISREMOVABLEUSABLE::{
    GameNavEntIsRemovableUsable, GameNavEntIsRemovableUsableArgs,
};
use mp_abi::game::vmcalls::GAME_NAV_ENTISUNLOCKEDDOOR::{
    GameNavEntIsUnlockedDoor, GameNavEntIsUnlockedDoorArgs,
};
use mp_abi::game::vmcalls::GAME_NAV_FINDCOMBATPOINTWAYPOINTS::GameNavFindcombatpointwaypoints;
use mp_abi::game::vmcalls::GAME_ROFF_NOTETRACK_CALLBACK::{
    GameRoffNotetrackCallback, GameRoffNotetrackCallbackArgs,
};
use mp_abi::game::vmcalls::GAME_RUN_FRAME::{GameRunFrame, GameRunFrameArgs};
use mp_abi::game::vmcalls::GAME_SHUTDOWN::{GameShutdown, GameShutdownArgs};
use mp_abi::game::vmcalls::GAME_SPAWN_RMG_ENTITY::GameSpawnRmgEntity;
use mp_abi::Dispatch;

// The 17 `GAME_ICARUS_*` inbound commands (`g_main.c:558-668`). Each carries an
// `Args = ()` marker (the payload arrives out-of-band in the module's
// `gSharedBuffer` shared-memory region — see below), so the impls read their
// `T_G_ICARUS_*` struct by overlay-casting that buffer.
use mp_abi::game::vmcalls::GAME_ICARUS_GETFLOAT::GameIcarusGetfloat;
use mp_abi::game::vmcalls::GAME_ICARUS_GETSETIDFORSTRING::GameIcarusGetsetidforstring;
use mp_abi::game::vmcalls::GAME_ICARUS_GETSTRING::GameIcarusGetstring;
use mp_abi::game::vmcalls::GAME_ICARUS_GETTAG::GameIcarusGettag;
use mp_abi::game::vmcalls::GAME_ICARUS_GETVECTOR::GameIcarusGetvector;
use mp_abi::game::vmcalls::GAME_ICARUS_KILL::GameIcarusKill;
use mp_abi::game::vmcalls::GAME_ICARUS_LERP2ANGLES::GameIcarusLerp2Angles;
use mp_abi::game::vmcalls::GAME_ICARUS_LERP2END::GameIcarusLerp2End;
use mp_abi::game::vmcalls::GAME_ICARUS_LERP2ORIGIN::GameIcarusLerp2Origin;
use mp_abi::game::vmcalls::GAME_ICARUS_LERP2POS::GameIcarusLerp2Pos;
use mp_abi::game::vmcalls::GAME_ICARUS_LERP2START::GameIcarusLerp2Start;
use mp_abi::game::vmcalls::GAME_ICARUS_PLAY::GameIcarusPlay;
use mp_abi::game::vmcalls::GAME_ICARUS_PLAYSOUND::GameIcarusPlaysound;
use mp_abi::game::vmcalls::GAME_ICARUS_REMOVE::GameIcarusRemove;
use mp_abi::game::vmcalls::GAME_ICARUS_SET::GameIcarusSet;
use mp_abi::game::vmcalls::GAME_ICARUS_SOUNDINDEX::GameIcarusSoundindex;
use mp_abi::game::vmcalls::GAME_ICARUS_USE::GameIcarusUse;

use mp_qshared::common::mp::qcommon::t_g_icarus_getfloat::T_G_ICARUS_GETFLOAT;
use mp_qshared::common::mp::qcommon::t_g_icarus_getsetidforstring::T_G_ICARUS_GETSETIDFORSTRING;
use mp_qshared::common::mp::qcommon::t_g_icarus_getstring::T_G_ICARUS_GETSTRING;
use mp_qshared::common::mp::qcommon::t_g_icarus_gettag::T_G_ICARUS_GETTAG;
use mp_qshared::common::mp::qcommon::t_g_icarus_getvector::T_G_ICARUS_GETVECTOR;
use mp_qshared::common::mp::qcommon::t_g_icarus_kill::T_G_ICARUS_KILL;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_angles::T_G_ICARUS_LERP2ANGLES;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_end::T_G_ICARUS_LERP2END;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_origin::T_G_ICARUS_LERP2ORIGIN;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_pos::T_G_ICARUS_LERP2POS;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_start::T_G_ICARUS_LERP2START;
use mp_qshared::common::mp::qcommon::t_g_icarus_play::T_G_ICARUS_PLAY;
use mp_qshared::common::mp::qcommon::t_g_icarus_playsound::T_G_ICARUS_PLAYSOUND;
use mp_qshared::common::mp::qcommon::t_g_icarus_remove::T_G_ICARUS_REMOVE;
use mp_qshared::common::mp::qcommon::t_g_icarus_set::T_G_ICARUS_SET;
use mp_qshared::common::mp::qcommon::t_g_icarus_soundindex::T_G_ICARUS_SOUNDINDEX;
use mp_qshared::common::mp::qcommon::t_g_icarus_use::T_G_ICARUS_USE;
use mp_qshared::shared::string_id_table::stringID_table_t;

use crate::g_icarus_set_type::setTable;
use crate::g_utils::G_SoundIndex;
use crate::g_ICARUScb::{
    Q3_GetFloat, Q3_GetString, Q3_GetTag, Q3_GetVector, Q3_Kill, Q3_Lerp2Angles, Q3_Lerp2End,
    Q3_Lerp2Origin, Q3_Lerp2Pos, Q3_Lerp2Start, Q3_Play, Q3_PlaySound, Q3_Remove, Q3_Set, Q3_Use,
};
use crate::q_shared::GetIDForString;

/// `GAME_INIT` → `G_InitGame( arg0, arg1, arg2 )` (`g_main.c:517-519`).
impl Dispatch<GameInit> for GameContext<'_> {
    fn dispatch(&self, args: GameInitArgs) {
        crate::g_init_game::g_init_game(*self, args)
    }
}

/// `GAME_SHUTDOWN` → `G_ShutdownGame( arg0 )` (`g_main.c:520-522`).
impl Dispatch<GameShutdown> for GameContext<'_> {
    fn dispatch(&self, args: GameShutdownArgs) {
        crate::g_shutdown_game::g_shutdown_game(*self, args)
    }
}

/// `GAME_CLIENT_CONNECT` → `return (int)ClientConnect( arg0, arg1, arg2 )`
/// (`g_main.c:523-524`).
impl Dispatch<GameClientConnect> for GameContext<'_> {
    fn dispatch(&self, args: GameClientConnectArgs) -> *const c_char {
        crate::g_client::ClientConnect(*self, args.client_num(), args.first_time(), args.is_bot())
            as *const c_char
    }
}

/// `GAME_CLIENT_THINK` → `ClientThink( arg0, NULL )` (`g_main.c:525-527`).
impl Dispatch<GameClientThink> for GameContext<'_> {
    fn dispatch(&self, args: GameClientThinkArgs) {
        crate::g_active::ClientThink(*self, args.client_num(), core::ptr::null_mut())
    }
}

/// `GAME_CLIENT_USERINFO_CHANGED` → `ClientUserinfoChanged( arg0 )`
/// (`g_main.c:528-530`).
impl Dispatch<GameClientUserinfoChanged> for GameContext<'_> {
    fn dispatch(&self, args: GameClientUserinfoChangedArgs) {
        crate::g_client::ClientUserinfoChanged(*self, args.client_num())
    }
}

/// `GAME_CLIENT_DISCONNECT` → `ClientDisconnect( arg0 )` (`g_main.c:531-533`).
impl Dispatch<GameClientDisconnect> for GameContext<'_> {
    fn dispatch(&self, args: GameClientDisconnectArgs) {
        crate::g_client::ClientDisconnect(*self, args.client_num())
    }
}

/// `GAME_CLIENT_BEGIN` → `ClientBegin( arg0, qtrue )` (`g_main.c:534-536`).
impl Dispatch<GameClientBegin> for GameContext<'_> {
    fn dispatch(&self, args: GameClientBeginArgs) {
        crate::g_client::ClientBegin(*self, args.client_num(), qtrue)
    }
}

/// `GAME_CLIENT_COMMAND` → `ClientCommand( arg0 )` (`g_main.c:537-539`).
impl Dispatch<GameClientCommand> for GameContext<'_> {
    fn dispatch(&self, args: GameClientCommandArgs) {
        crate::g_cmds::ClientCommand(*self, args.client_num())
    }
}

/// `GAME_RUN_FRAME` → `G_RunFrame( arg0 )` (`g_main.c:540-542`).
impl Dispatch<GameRunFrame> for GameContext<'_> {
    fn dispatch(&self, args: GameRunFrameArgs) {
        crate::g_main::G_RunFrame(*self, args.level_time())
    }
}

/// `GAME_CONSOLE_COMMAND` → `return ConsoleCommand()` (`g_main.c:543-544`).
impl Dispatch<GameConsoleCommand> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> qboolean {
        crate::g_svcmds::ConsoleCommand(*self)
    }
}

/// `BOTAI_START_FRAME` → `return BotAIStartFrame( arg0 )` (`g_main.c:545-546`).
impl Dispatch<BotAiStartFrame> for GameContext<'_> {
    fn dispatch(&self, args: BotAiStartFrameArgs) -> c_int {
        crate::ai_main::BotAIStartFrame(*self, args.time())
    }
}

/// `GAME_ROFF_NOTETRACK_CALLBACK` →
/// `G_ROFF_NotetrackCallback( &g_entities[arg0], (const char *)arg1 )`
/// (`g_main.c:547-549`).
impl Dispatch<GameRoffNotetrackCallback> for GameContext<'_> {
    fn dispatch(&self, args: GameRoffNotetrackCallbackArgs) {
        // SAFETY: seam reborrow of the owned entity arena (STATE-D6).
        let cent =
            unsafe { &mut (*self.world).g_entities[args.ent_num() as usize] as *mut gentity_t };
        crate::g_utils::G_ROFF_NotetrackCallback(*self, cent, args.notetrack())
    }
}

/// `GAME_SPAWN_RMG_ENTITY` →
/// `if (G_ParseSpawnVars(qfalse)) { G_SpawnGEntityFromSpawnVars(qfalse); }`
/// (`g_main.c:550-555`).
impl Dispatch<GameSpawnRmgEntity> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        if crate::g_spawn::G_ParseSpawnVars(*self, qfalse) != qfalse {
            crate::g_spawn::G_SpawnGEntityFromSpawnVars(*self, qfalse);
        }
    }
}

/// `GAME_NAV_CLEARPATHTOPOINT` →
/// `return NAV_ClearPathToPoint(&g_entities[arg0], (float *)arg1, (float *)arg2,
///  (float *)arg3, arg4, arg5)` (`g_main.c:672-673`).
impl Dispatch<GameNavClearpathtopoint> for GameContext<'_> {
    fn dispatch(&self, args: GameNavClearpathtopointArgs) -> qboolean {
        // SAFETY: seam reborrow of the owned entity arena; the `float *` vectors
        // are engine-owned, read by value at the seam (STATE-D6).
        let self_ =
            unsafe { &mut (*self.world).g_entities[args.entity_num() as usize] as *mut gentity_t };
        let pmins = unsafe { *(args.pmins() as *const vec3_t) };
        let pmaxs = unsafe { *(args.pmaxs() as *const vec3_t) };
        let point = unsafe { *(args.point() as *const vec3_t) };
        crate::g_nav::NAV_ClearPathToPoint(
            *self,
            self_,
            pmins,
            pmaxs,
            point,
            args.clipmask(),
            args.ok_to_hit_ent_num(),
        )
    }
}

/// `GAME_NAV_CLEARLOS` →
/// `return NPC_ClearLOS2(&g_entities[arg0], (const float *)arg1)`
/// (`g_main.c:674-675`).
impl Dispatch<GameNavClearlos> for GameContext<'_> {
    fn dispatch(&self, args: GameNavClearlosArgs) -> qboolean {
        // SAFETY: seam reborrow + engine-owned end vector, read at the seam.
        let ent =
            unsafe { &mut (*self.world).g_entities[args.entity_num() as usize] as *mut gentity_t };
        let end = unsafe { *(args.end() as *const vec3_t) };
        crate::NPC_utils::NPC_ClearLOS2(*self, ent, end)
    }
}

/// `GAME_NAV_CLEARPATHBETWEENPOINTS` →
/// `return NAVNEW_ClearPathBetweenPoints((float *)arg0, (float *)arg1,
///  (float *)arg2, (float *)arg3, arg4, arg5)` (`g_main.c:676-677`).
impl Dispatch<GameNavClearpathbetweenpoints> for GameContext<'_> {
    fn dispatch(&self, args: GameNavClearpathbetweenpointsArgs) -> c_int {
        // SAFETY: the four `float *` vectors are engine-owned, read at the seam.
        let start = unsafe { *(args.start() as *const vec3_t) };
        let end = unsafe { *(args.end() as *const vec3_t) };
        let mins = unsafe { *(args.mins() as *const vec3_t) };
        let maxs = unsafe { *(args.maxs() as *const vec3_t) };
        crate::g_navnew::NAVNEW_ClearPathBetweenPoints(
            *self,
            start,
            end,
            mins,
            maxs,
            args.ignore(),
            args.clipmask(),
        )
    }
}

/// `GAME_NAV_CHECKNODEFAILEDFORENT` →
/// `return NAV_CheckNodeFailedForEnt(&g_entities[arg0], arg1)`
/// (`g_main.c:678-679`).
impl Dispatch<GameNavChecknodefailedforent> for GameContext<'_> {
    fn dispatch(&self, args: GameNavChecknodefailedforentArgs) -> qboolean {
        // SAFETY: seam reborrow of the owned entity arena (STATE-D6).
        let ent =
            unsafe { &mut (*self.world).g_entities[args.entity_num() as usize] as *mut gentity_t };
        crate::g_navnew::NAV_CheckNodeFailedForEnt(ent, args.node_num())
    }
}

/// `GAME_NAV_ENTISUNLOCKEDDOOR` → `return G_EntIsUnlockedDoor(arg0)`
/// (`g_main.c:680-681`).
impl Dispatch<GameNavEntIsUnlockedDoor> for GameContext<'_> {
    fn dispatch(&self, args: GameNavEntIsUnlockedDoorArgs) -> qboolean {
        crate::g_mover::G_EntIsUnlockedDoor(*self, args.entity_num())
    }
}

/// `GAME_NAV_ENTISDOOR` → `return G_EntIsDoor(arg0)` (`g_main.c:682-683`).
impl Dispatch<GameNavEntIsDoor> for GameContext<'_> {
    fn dispatch(&self, args: GameNavEntIsDoorArgs) -> qboolean {
        crate::g_mover::G_EntIsDoor(*self, args.entity_num())
    }
}

/// `GAME_NAV_ENTISBREAKABLE` → `return G_EntIsBreakable(arg0)`
/// (`g_main.c:684-685`).
impl Dispatch<GameNavEntIsBreakable> for GameContext<'_> {
    fn dispatch(&self, args: GameNavEntIsBreakableArgs) -> qboolean {
        crate::g_mover::G_EntIsBreakable(*self, args.entity_num())
    }
}

/// `GAME_NAV_ENTISREMOVABLEUSABLE` → `return G_EntIsRemovableUsable(arg0)`
/// (`g_main.c:686-687`).
impl Dispatch<GameNavEntIsRemovableUsable> for GameContext<'_> {
    fn dispatch(&self, args: GameNavEntIsRemovableUsableArgs) -> qboolean {
        crate::g_mover::G_EntIsRemovableUsable(*self, args.entity_num())
    }
}

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` → `CP_FindCombatPointWaypoints()`
/// (`g_main.c:688-689`).
impl Dispatch<GameNavFindcombatpointwaypoints> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        crate::NPC_combat::CP_FindCombatPointWaypoints(*self)
    }
}

/// `GAME_GETITEMINDEXBYTAG` → `return BG_GetItemIndexByTag(arg0, arg1)`
/// (`g_main.c:690-691`).
impl Dispatch<GameGetitemindexbytag> for GameContext<'_> {
    fn dispatch(&self, args: GameGetitemindexbytagArgs) -> c_int {
        crate::bg_misc::BG_GetItemIndexByTag(args.tag(), args.type_())
    }
}

// --- ICARUS callbacks (`g_main.c:557-670`) -----------------------------------
//
// The engine writes each command's `T_G_ICARUS_*` payload into the shared-memory
// region it registered via `trap_SV_RegisterSharedMemory` — the module's
// `GameWorld.gSharedBuffer` (`G_InitGame`). Raven's switch overlay-casts that raw
// `gSharedBuffer` to the command's struct (`(T_G_ICARUS_X *)gSharedBuffer`) and
// reads/writes fields in place; the module and engine address the SAME bytes, so
// out-params (e.g. `GETFLOAT.value`) land back in the engine's view. Each arm
// reproduces that cast: `&mut gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_X`.
// Source: `oracle/codemp/game/g_main.c:557-670`.

/// libc `strcpy` mirror for the `GAME_ICARUS_GETSTRING` write-back
/// (`g_main.c:654`, `strcpy(sharedMem->value, crap)`); copies through the NUL.
unsafe fn c_strcpy(dst: *mut c_char, src: *const c_char) {
    let mut i = 0isize;
    loop {
        let ch = *src.offset(i);
        *dst.offset(i) = ch;
        if ch == 0 {
            break;
        }
        i += 1;
    }
}

/// `GAME_ICARUS_PLAYSOUND` →
/// `return Q3_PlaySound( m->taskID, m->entID, m->name, m->channel )`
/// (`g_main.c:558-562`).
impl Dispatch<GameIcarusPlaysound> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> c_int {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_PLAYSOUND)
        };
        Q3_PlaySound(
            *self,
            m.taskID,
            m.entID,
            m.name.as_ptr() as *const c_char,
            m.channel.as_ptr() as *const c_char,
        )
    }
}

/// `GAME_ICARUS_SET` →
/// `return Q3_Set( m->taskID, m->entID, m->type_name, m->data )`
/// (`g_main.c:563-567`).
impl Dispatch<GameIcarusSet> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> qboolean {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m =
            unsafe { &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_SET) };
        Q3_Set(
            *self,
            m.taskID,
            m.entID,
            m.type_name.as_ptr() as *const c_char,
            m.data.as_ptr() as *const c_char,
        )
    }
}

/// `GAME_ICARUS_LERP2POS` →
/// `if (m->nullAngles) Q3_Lerp2Pos( m->taskID, m->entID, m->origin, NULL, m->duration );`
/// `else Q3_Lerp2Pos( m->taskID, m->entID, m->origin, m->angles, m->duration );`
/// (`g_main.c:568-580`).
impl Dispatch<GameIcarusLerp2Pos> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_LERP2POS)
        };
        let (task_id, ent_id, duration) = (m.taskID, m.entID, m.duration);
        if m.nullAngles != qfalse {
            Q3_Lerp2Pos(*self, task_id, ent_id, &mut m.origin, None, duration);
        } else {
            Q3_Lerp2Pos(
                *self,
                task_id,
                ent_id,
                &mut m.origin,
                Some(&mut m.angles),
                duration,
            );
        }
    }
}

/// `GAME_ICARUS_LERP2ORIGIN` →
/// `Q3_Lerp2Origin( m->taskID, m->entID, m->origin, m->duration )`
/// (`g_main.c:581-586`).
impl Dispatch<GameIcarusLerp2Origin> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_LERP2ORIGIN)
        };
        Q3_Lerp2Origin(*self, m.taskID, m.entID, m.origin, m.duration);
    }
}

/// `GAME_ICARUS_LERP2ANGLES` →
/// `Q3_Lerp2Angles( m->taskID, m->entID, m->angles, m->duration )`
/// (`g_main.c:587-592`).
impl Dispatch<GameIcarusLerp2Angles> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_LERP2ANGLES)
        };
        Q3_Lerp2Angles(*self, m.taskID, m.entID, m.angles, m.duration);
    }
}

/// `GAME_ICARUS_GETTAG` →
/// `return Q3_GetTag( m->entID, m->name, m->lookup, m->info )`
/// (`g_main.c:593-597`).
impl Dispatch<GameIcarusGettag> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> c_int {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_GETTAG)
        };
        let (ent_id, lookup) = (m.entID, m.lookup);
        let name = m.name.as_ptr() as *const c_char;
        Q3_GetTag(*self, ent_id, name, lookup, &mut m.info)
    }
}

/// `GAME_ICARUS_LERP2START` →
/// `Q3_Lerp2Start( m->entID, m->taskID, m->duration )` (`g_main.c:598-603`).
impl Dispatch<GameIcarusLerp2Start> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_LERP2START)
        };
        Q3_Lerp2Start(*self, m.entID, m.taskID, m.duration);
    }
}

/// `GAME_ICARUS_LERP2END` →
/// `Q3_Lerp2End( m->entID, m->taskID, m->duration )` (`g_main.c:604-609`).
impl Dispatch<GameIcarusLerp2End> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_LERP2END)
        };
        Q3_Lerp2End(*self, m.entID, m.taskID, m.duration);
    }
}

/// `GAME_ICARUS_USE` → `Q3_Use( m->entID, m->target )` (`g_main.c:610-615`).
impl Dispatch<GameIcarusUse> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m =
            unsafe { &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_USE) };
        Q3_Use(*self, m.entID, m.target.as_ptr() as *const c_char);
    }
}

/// `GAME_ICARUS_KILL` → `Q3_Kill( m->entID, m->name )` (`g_main.c:616-621`).
impl Dispatch<GameIcarusKill> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_KILL)
        };
        Q3_Kill(*self, m.entID, m.name.as_ptr() as *const c_char);
    }
}

/// `GAME_ICARUS_REMOVE` → `Q3_Remove( m->entID, m->name )` (`g_main.c:622-627`).
impl Dispatch<GameIcarusRemove> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_REMOVE)
        };
        Q3_Remove(*self, m.entID, m.name.as_ptr() as *const c_char);
    }
}

/// `GAME_ICARUS_PLAY` →
/// `Q3_Play( m->taskID, m->entID, m->type, m->name )` (`g_main.c:628-633`).
impl Dispatch<GameIcarusPlay> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_PLAY)
        };
        Q3_Play(
            *self,
            m.taskID,
            m.entID,
            m.r#type.as_ptr() as *const c_char,
            m.name.as_ptr() as *const c_char,
        );
    }
}

/// `GAME_ICARUS_GETFLOAT` →
/// `return Q3_GetFloat( m->entID, m->type, m->name, &m->value )`
/// (`g_main.c:634-638`).
impl Dispatch<GameIcarusGetfloat> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> c_int {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_GETFLOAT)
        };
        let (ent_id, ty) = (m.entID, m.r#type);
        let name = m.name.as_ptr() as *const c_char;
        Q3_GetFloat(*self, ent_id, ty, name, &mut m.value as *mut f32)
    }
}

/// `GAME_ICARUS_GETVECTOR` →
/// `return Q3_GetVector( m->entID, m->type, m->name, m->value )`
/// (`g_main.c:639-643`).
impl Dispatch<GameIcarusGetvector> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> c_int {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_GETVECTOR)
        };
        let (ent_id, ty) = (m.entID, m.r#type);
        let name = m.name.as_ptr() as *const c_char;
        Q3_GetVector(*self, ent_id, ty, name, &mut m.value)
    }
}

/// `GAME_ICARUS_GETSTRING` → `Q3_GetString( m->entID, m->type, m->name, &crap )`
/// then `if (crap) strcpy( m->value, crap )`; returns the `Q3_GetString` result
/// (`g_main.c:644-658`).
impl Dispatch<GameIcarusGetstring> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> c_int {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_GETSTRING)
        };
        let (ent_id, ty) = (m.entID, m.r#type);
        let name = m.name.as_ptr() as *const c_char;
        // Raven's `char *crap = NULL; char **morecrap = &crap;` out-pointer dance.
        let mut crap: *mut c_char = core::ptr::null_mut();
        let morecrap = &mut crap as *mut *mut c_char;
        let r = Q3_GetString(*self, ent_id, ty, name, morecrap);
        if !crap.is_null() {
            // SAFETY: on success `crap` points at a valid NUL-terminated string;
            // `m->value` is a 2048-byte buffer (`g_public.h`).
            unsafe { c_strcpy(m.value.as_mut_ptr() as *mut c_char, crap) };
        }
        r
    }
}

/// `GAME_ICARUS_SOUNDINDEX` → `G_SoundIndex( m->filename )` (`g_main.c:659-664`).
impl Dispatch<GameIcarusSoundindex> for GameContext<'_> {
    fn dispatch(&self, _args: ()) {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8 as *mut T_G_ICARUS_SOUNDINDEX)
        };
        // Raven discards `G_SoundIndex`'s handle here (registration side effect).
        G_SoundIndex(m.filename.as_ptr() as *const c_char);
    }
}

/// `GAME_ICARUS_GETSETIDFORSTRING` →
/// `return GetIDForString( setTable, m->string )` (`g_main.c:665-669`).
impl Dispatch<GameIcarusGetsetidforstring> for GameContext<'_> {
    fn dispatch(&self, _args: ()) -> c_int {
        // SAFETY: overlay-cast of the engine-registered shared buffer (SEAM-D8).
        let m = unsafe {
            &mut *(&mut (*self.world).gSharedBuffer[0] as *mut u8
                as *mut T_G_ICARUS_GETSETIDFORSTRING)
        };
        GetIDForString(
            setTable.as_ptr() as *mut stringID_table_t,
            m.string.as_ptr() as *const c_char,
        )
    }
}
