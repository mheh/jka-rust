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
use mp_qshared::shared::{qboolean, vec3_t, QFALSE, QTRUE};

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
        crate::g_client::ClientBegin(*self, args.client_num(), QTRUE)
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
        if crate::g_spawn::G_ParseSpawnVars(*self, QFALSE) != QFALSE {
            crate::g_spawn::G_SpawnGEntityFromSpawnVars(*self, QFALSE);
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

//TODO: Port Dispatch<C> for GameContext (GAME_ICARUS_* commands)
// The 17 ICARUS arms read `T_G_ICARUS_*` structs out of the module's
// `gSharedBuffer` shared-memory region, which is not yet modeled in `GameWorld`
// (a design decision — where the registered buffer lives and how it is typed).
// Source: docs/architecture/engine-seam.md § inbound dual (SEAM-D8);
// oracle/oracle/codemp/game/g_main.c:558-668
