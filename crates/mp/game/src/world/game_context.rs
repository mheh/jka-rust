//! `GameContext` is the module-side `Dispatch<C>` receiver.
//! It replaces the earlier `WorldPtr` type.

use mp_engine_select::Engine;

use super::game_world::GameWorld;

/// The copyable `Dispatch<C>` receiver each `vmMain` command routes through.
/// It is defined in `mp_game`.
/// `Engine` is the `mp_engine_select` module-side transport alias, not `mp_engine_core::Engine`.
/// `vmMain` constructs one receiver per call, as a plain struct literal built by the shell from its `WORLD` and `ENGINE.get()`.
/// The fields are `pub`, because a `Copy` struct of raw pointers holds no invariant to protect (the `WorldPtr` type set this precedent).
/// Each `impl Dispatch<C> for GameContext` unpacks `world` through leaf reborrows.
/// It passes `engine` into the logic functions' `trap::X(engine, ...)` call sites, in oracle syscall order.
/// The `&Engine` channel is the receiver's own field, so no `dispatch` parameter is needed.
/// The orphan rule is satisfied because `GameContext` and its impls both live in `mp_game`.
///
/// Source: `docs/architecture/engine-seam.md` § inbound dual (SEAM-Q12 amendment 2026-07-03).
pub struct GameContext<'e> {
    /// The one owned [`GameWorld`] instance, borrowed for the duration of the `vmMain` call.
    /// The borrow checker enforces state threading (porting rule §B4) directly.
    pub world: &'e mut GameWorld,
    pub engine: &'e Engine,
}

impl GameContext<'_> {
    /// This is a raw-pointer bridge for callers not yet converted to real borrows.
    /// The borrow ends at return, so pointer-heavy bodies keep their raw-pointer aliasing discipline unchanged.
    #[inline]
    pub fn world_raw(&mut self) -> *mut GameWorld {
        &raw mut *self.world
    }

    /// This borrows entity `id` from the owned arena.
    #[inline]
    pub fn entity(&self, id: EntityId) -> &gentity_t {
        &self.world.g_entities[id.index()]
    }

    /// This is the mutable version of [`Self::entity`].
    #[inline]
    pub fn entity_mut(&mut self, id: EntityId) -> &mut gentity_t {
        &mut self.world.g_entities[id.index()]
    }

    /// This recovers the [`EntityId`] of a raw `*mut gentity_t` pointer, and returns `None` for a NULL pointer.
    /// Callers that still use raw pointers call this at a converted function's boundary, to turn the pointer into a handle.
    ///
    /// This uses Raven's `ent - g_entities` pointer arithmetic, through [`ent_id_opt`].
    /// This is the canonical `ENTITYNUM` idiom, equivalent to reading `ent->s.number` (Raven sets `s.number = ent - g_entities` at spawn).
    /// Unlike a read of `s.number`, this does not depend on that field being current.
    ///
    /// SAFETY: `ent` is NULL, or it points into this world's contiguous `g_entities` arena, the only place `gentity_t` values live.
    /// The pointer-to-index arithmetic stays confined to the [`ent_id_opt`] helper.
    #[inline]
    pub fn entity_id_of(&self, ent: *const gentity_t) -> Option<EntityId> {
        unsafe { ent_id_opt(self.world.g_entities.as_ptr(), ent) }
    }
}

// The per-command `impl Dispatch<C> for GameContext` blocks live here as thin adapters only.
// Per-command logic stays one function per file.
// Each block unpacks `self.world` through leaf reborrows, and passes `self.engine` into the ported logic functions.

use core::ffi::{c_char, c_int};

use crate::entity::gentity_t;
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

// These are the 17 `GAME_ICARUS_*` inbound commands (`g_main.c:558-668`).
// Each carries an `Args = ()` marker, because the payload arrives out-of-band in the module's `gSharedBuffer` shared-memory region (see below).
// Each impl reads its `T_G_ICARUS_*` struct by overlay-casting that buffer.
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

use mp_qshared::shared::string_id_table::stringID_table_t;

use crate::g_ICARUScb::{
    Q3_GetFloat, Q3_GetString, Q3_GetTag, Q3_GetVector, Q3_Kill, Q3_Lerp2Angles, Q3_Lerp2End,
    Q3_Lerp2Origin, Q3_Lerp2Pos, Q3_Lerp2Start, Q3_Play, Q3_PlaySound, Q3_Remove, Q3_Set, Q3_Use,
};
use crate::g_icarus_set_type::setTable;
use crate::g_utils::G_SoundIndex;
use crate::q_shared::GetIDForString;

/// `GAME_INIT` → `G_InitGame( arg0, arg1, arg2 )` (`g_main.c:517-519`).
impl Dispatch<GameInit> for GameContext<'_> {
    fn dispatch(&mut self, args: GameInitArgs) {
        crate::g_init_game::g_init_game(self, args)
    }
}

/// `GAME_SHUTDOWN` → `G_ShutdownGame( arg0 )` (`g_main.c:520-522`).
impl Dispatch<GameShutdown> for GameContext<'_> {
    fn dispatch(&mut self, args: GameShutdownArgs) {
        crate::g_shutdown_game::g_shutdown_game(self, args)
    }
}

/// `GAME_CLIENT_CONNECT` → `return (int)ClientConnect( arg0, arg1, arg2 )` (`g_main.c:523-524`).
impl Dispatch<GameClientConnect> for GameContext<'_> {
    fn dispatch(&mut self, args: GameClientConnectArgs) -> *const c_char {
        crate::g_client::ClientConnect(self, args.client_num(), args.first_time(), args.is_bot())
            as *const c_char
    }
}

/// `GAME_CLIENT_THINK` → `ClientThink( arg0, NULL )` (`g_main.c:525-527`).
impl Dispatch<GameClientThink> for GameContext<'_> {
    fn dispatch(&mut self, args: GameClientThinkArgs) {
        crate::g_active::ClientThink(self, args.client_num(), core::ptr::null_mut())
    }
}

/// `GAME_CLIENT_USERINFO_CHANGED` → `ClientUserinfoChanged( arg0 )` (`g_main.c:528-530`).
impl Dispatch<GameClientUserinfoChanged> for GameContext<'_> {
    fn dispatch(&mut self, args: GameClientUserinfoChangedArgs) {
        crate::g_client::ClientUserinfoChanged(self, args.client_num())
    }
}

/// `GAME_CLIENT_DISCONNECT` → `ClientDisconnect( arg0 )` (`g_main.c:531-533`).
impl Dispatch<GameClientDisconnect> for GameContext<'_> {
    fn dispatch(&mut self, args: GameClientDisconnectArgs) {
        crate::g_client::ClientDisconnect(self, args.client_num())
    }
}

/// `GAME_CLIENT_BEGIN` → `ClientBegin( arg0, qtrue )` (`g_main.c:534-536`).
impl Dispatch<GameClientBegin> for GameContext<'_> {
    fn dispatch(&mut self, args: GameClientBeginArgs) {
        crate::g_client::ClientBegin(self, args.client_num(), qtrue)
    }
}

/// `GAME_CLIENT_COMMAND` → `ClientCommand( arg0 )` (`g_main.c:537-539`).
impl Dispatch<GameClientCommand> for GameContext<'_> {
    fn dispatch(&mut self, args: GameClientCommandArgs) {
        crate::g_cmds::ClientCommand(self, args.client_num())
    }
}

/// `GAME_RUN_FRAME` → `G_RunFrame( arg0 )` (`g_main.c:540-542`).
impl Dispatch<GameRunFrame> for GameContext<'_> {
    fn dispatch(&mut self, args: GameRunFrameArgs) {
        crate::g_main::G_RunFrame(self, args.level_time())
    }
}

/// `GAME_CONSOLE_COMMAND` → `return ConsoleCommand()` (`g_main.c:543-544`).
impl Dispatch<GameConsoleCommand> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> qboolean {
        crate::g_svcmds::ConsoleCommand(self)
    }
}

/// `BOTAI_START_FRAME` → `return BotAIStartFrame( arg0 )` (`g_main.c:545-546`).
impl Dispatch<BotAiStartFrame> for GameContext<'_> {
    fn dispatch(&mut self, args: BotAiStartFrameArgs) -> c_int {
        crate::ai_main::BotAIStartFrame(self, args.time())
    }
}

/// `GAME_ROFF_NOTETRACK_CALLBACK` → `G_ROFF_NotetrackCallback( &g_entities[arg0], (const char *)arg1 )` (`g_main.c:547-549`).
impl Dispatch<GameRoffNotetrackCallback> for GameContext<'_> {
    fn dispatch(&mut self, args: GameRoffNotetrackCallbackArgs) {
        // SAFETY: this reborrows the owned entity arena at the seam.
        let cent = &mut self.world.g_entities[args.ent_num() as usize] as *mut gentity_t;
        crate::g_utils::G_ROFF_NotetrackCallback(self, self.entity_id_of(cent), args.notetrack())
    }
}

/// `GAME_SPAWN_RMG_ENTITY` → `if (G_ParseSpawnVars(qfalse)) { G_SpawnGEntityFromSpawnVars(qfalse); }` (`g_main.c:550-555`).
impl Dispatch<GameSpawnRmgEntity> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        if crate::g_spawn::G_ParseSpawnVars(self, qfalse) != qfalse {
            crate::g_spawn::G_SpawnGEntityFromSpawnVars(self, qfalse);
        }
    }
}

/// `GAME_NAV_CLEARPATHTOPOINT` → `return NAV_ClearPathToPoint(&g_entities[arg0], (float *)arg1, (float *)arg2,
/// (float *)arg3, arg4, arg5)` (`g_main.c:672-673`).
impl Dispatch<GameNavClearpathtopoint> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavClearpathtopointArgs) -> qboolean {
        // SAFETY: this reborrows the owned entity arena at the seam.
        // The `float *` vectors are engine-owned, and this reads them by value at the seam.
        let self_ = &mut self.world.g_entities[args.entity_num() as usize] as *mut gentity_t;
        let pmins = unsafe { *(args.pmins() as *const vec3_t) };
        let pmaxs = unsafe { *(args.pmaxs() as *const vec3_t) };
        let point = unsafe { *(args.point() as *const vec3_t) };
        crate::g_nav::NAV_ClearPathToPoint(
            self,
            self.entity_id_of(self_).unwrap(),
            pmins,
            pmaxs,
            point,
            args.clipmask(),
            args.ok_to_hit_ent_num(),
        )
    }
}

/// `GAME_NAV_CLEARLOS` → `return NPC_ClearLOS2(&g_entities[arg0], (const float *)arg1)` (`g_main.c:674-675`).
impl Dispatch<GameNavClearlos> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavClearlosArgs) -> qboolean {
        // SAFETY: this reborrows the owned entity arena at the seam.
        // The `end` vector is engine-owned, and this reads it at the seam.
        let ent = &mut self.world.g_entities[args.entity_num() as usize] as *mut gentity_t;
        let end = unsafe { *(args.end() as *const vec3_t) };
        crate::NPC_utils::NPC_ClearLOS2(self, self.entity_id_of(ent), end)
    }
}

/// `GAME_NAV_CLEARPATHBETWEENPOINTS` → `return NAVNEW_ClearPathBetweenPoints((float *)arg0, (float *)arg1,
/// (float *)arg2, (float *)arg3, arg4, arg5)` (`g_main.c:676-677`).
impl Dispatch<GameNavClearpathbetweenpoints> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavClearpathbetweenpointsArgs) -> c_int {
        // SAFETY: the four `float *` vectors are engine-owned, and this reads them at the seam.
        let start = unsafe { *(args.start() as *const vec3_t) };
        let end = unsafe { *(args.end() as *const vec3_t) };
        let mins = unsafe { *(args.mins() as *const vec3_t) };
        let maxs = unsafe { *(args.maxs() as *const vec3_t) };
        crate::g_navnew::NAVNEW_ClearPathBetweenPoints(
            self,
            start,
            end,
            mins,
            maxs,
            args.ignore(),
            args.clipmask(),
        )
    }
}

/// `GAME_NAV_CHECKNODEFAILEDFORENT` → `return NAV_CheckNodeFailedForEnt(&g_entities[arg0], arg1)` (`g_main.c:678-679`).
impl Dispatch<GameNavChecknodefailedforent> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavChecknodefailedforentArgs) -> qboolean {
        // SAFETY: this reborrows the owned entity arena at the seam.
        let ent = &mut self.world.g_entities[args.entity_num() as usize] as *mut gentity_t;
        crate::g_navnew::NAV_CheckNodeFailedForEnt(unsafe { &*ent }, args.node_num())
    }
}

/// `GAME_NAV_ENTISUNLOCKEDDOOR` → `return G_EntIsUnlockedDoor(arg0)` (`g_main.c:680-681`).
impl Dispatch<GameNavEntIsUnlockedDoor> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavEntIsUnlockedDoorArgs) -> qboolean {
        crate::g_mover::G_EntIsUnlockedDoor(self, args.entity_num())
    }
}

/// `GAME_NAV_ENTISDOOR` → `return G_EntIsDoor(arg0)` (`g_main.c:682-683`).
impl Dispatch<GameNavEntIsDoor> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavEntIsDoorArgs) -> qboolean {
        crate::g_mover::G_EntIsDoor(self, args.entity_num())
    }
}

/// `GAME_NAV_ENTISBREAKABLE` → `return G_EntIsBreakable(arg0)` (`g_main.c:684-685`).
impl Dispatch<GameNavEntIsBreakable> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavEntIsBreakableArgs) -> qboolean {
        crate::g_mover::G_EntIsBreakable(self, args.entity_num())
    }
}

/// `GAME_NAV_ENTISREMOVABLEUSABLE` → `return G_EntIsRemovableUsable(arg0)` (`g_main.c:686-687`).
impl Dispatch<GameNavEntIsRemovableUsable> for GameContext<'_> {
    fn dispatch(&mut self, args: GameNavEntIsRemovableUsableArgs) -> qboolean {
        crate::g_mover::G_EntIsRemovableUsable(self, args.entity_num())
    }
}

/// `GAME_NAV_FINDCOMBATPOINTWAYPOINTS` → `CP_FindCombatPointWaypoints()` (`g_main.c:688-689`).
impl Dispatch<GameNavFindcombatpointwaypoints> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        crate::NPC_combat::CP_FindCombatPointWaypoints(self)
    }
}

/// `GAME_GETITEMINDEXBYTAG` → `return BG_GetItemIndexByTag(arg0, arg1)` (`g_main.c:690-691`).
impl Dispatch<GameGetitemindexbytag> for GameContext<'_> {
    fn dispatch(&mut self, args: GameGetitemindexbytagArgs) -> c_int {
        mp_bg::bg_misc::BG_GetItemIndexByTag(args.tag(), args.type_())
    }
}

// --- ICARUS callbacks (`g_main.c:557-670`) -----------------------------------
//
// The engine writes each command's `T_G_ICARUS_*` payload into a shared-memory region.
// It registers that region through `trap_SV_RegisterSharedMemory`, as the module's `GameWorld.gSharedBuffer` (set up in `G_InitGame`).
// Raven's switch overlay-casts the raw `gSharedBuffer` pointer to the command's struct, `(T_G_ICARUS_X *)gSharedBuffer`.
// It reads and writes fields in place.
// The module and the engine address the same bytes, so out-params (for example `GETFLOAT.value`) land back in the engine's view.
// The overlay cast now lives behind one typed accessor per command on `SharedBuffer` (`world/shared_buffer.rs`).
// Each arm calls, for example, `self.world.gSharedBuffer.playsound()` for a typed `&mut T_G_ICARUS_X`.
// It copies the scalar and string inputs out, which ends the buffer borrow so the ported logic function can take `self`.
// For out-param commands, it writes the result back through the accessor after the call.
// Source: `oracle/codemp/game/g_main.c:557-670`.

/// This mirrors libc `strcpy` for the `GAME_ICARUS_GETSTRING` write-back (`g_main.c:654`, `strcpy(sharedMem->value, crap)`).
/// It copies through the NUL terminator.
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

/// `GAME_ICARUS_PLAYSOUND` → `return Q3_PlaySound( m->taskID, m->entID, m->name, m->channel )` (`g_main.c:558-562`).
impl Dispatch<GameIcarusPlaysound> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> c_int {
        let m = self.world.gSharedBuffer.playsound();
        let (task_id, ent_id) = (m.taskID, m.entID);
        let name = m.name.as_ptr() as *const c_char;
        let channel = m.channel.as_ptr() as *const c_char;
        Q3_PlaySound(self, task_id, ent_id, name, channel)
    }
}

/// `GAME_ICARUS_SET` → `return Q3_Set( m->taskID, m->entID, m->type_name, m->data )` (`g_main.c:563-567`).
impl Dispatch<GameIcarusSet> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> qboolean {
        let m = self.world.gSharedBuffer.set();
        let (task_id, ent_id) = (m.taskID, m.entID);
        let type_name = m.type_name.as_ptr() as *const c_char;
        let data = m.data.as_ptr() as *const c_char;
        Q3_Set(self, task_id, ent_id, type_name, data)
    }
}

/// `GAME_ICARUS_LERP2POS` → `if (m->nullAngles) Q3_Lerp2Pos( m->taskID, m->entID, m->origin, NULL, m->duration );`
/// `else Q3_Lerp2Pos( m->taskID, m->entID, m->origin, m->angles, m->duration );` (`g_main.c:568-580`).
impl Dispatch<GameIcarusLerp2Pos> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.lerp2pos();
        let (task_id, ent_id, duration, null_angles) =
            (m.taskID, m.entID, m.duration, m.nullAngles);
        // `origin` and `angles` are read-only inputs to `Q3_Lerp2Pos` (verified: the function only reads them).
        // Local copies are behavior-identical, so no write-back is needed.
        let mut origin = m.origin;
        let mut angles = m.angles;
        if null_angles != qfalse {
            Q3_Lerp2Pos(self, task_id, ent_id, &mut origin, None, duration);
        } else {
            Q3_Lerp2Pos(
                self,
                task_id,
                ent_id,
                &mut origin,
                Some(&mut angles),
                duration,
            );
        }
    }
}

/// `GAME_ICARUS_LERP2ORIGIN` → `Q3_Lerp2Origin( m->taskID, m->entID, m->origin, m->duration )` (`g_main.c:581-586`).
impl Dispatch<GameIcarusLerp2Origin> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.lerp2origin();
        let (task_id, ent_id, origin, duration) = (m.taskID, m.entID, m.origin, m.duration);
        Q3_Lerp2Origin(self, task_id, ent_id, origin, duration);
    }
}

/// `GAME_ICARUS_LERP2ANGLES` → `Q3_Lerp2Angles( m->taskID, m->entID, m->angles, m->duration )` (`g_main.c:587-592`).
impl Dispatch<GameIcarusLerp2Angles> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.lerp2angles();
        let (task_id, ent_id, angles, duration) = (m.taskID, m.entID, m.angles, m.duration);
        Q3_Lerp2Angles(self, task_id, ent_id, angles, duration);
    }
}

/// `GAME_ICARUS_GETTAG` → `return Q3_GetTag( m->entID, m->name, m->lookup, m->info )` (`g_main.c:593-597`).
impl Dispatch<GameIcarusGettag> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> c_int {
        let m = self.world.gSharedBuffer.gettag();
        let (ent_id, lookup) = (m.entID, m.lookup);
        let name = m.name.as_ptr() as *const c_char;
        // `info` is an out-param. Copy it in, call over a local, then write the result back through the accessor.
        // This is equivalent to the in-place overlay write.
        let mut info = m.info;
        let r = Q3_GetTag(self, ent_id, name, lookup, &mut info);
        self.world.gSharedBuffer.gettag().info = info;
        r
    }
}

/// `GAME_ICARUS_LERP2START` → `Q3_Lerp2Start( m->entID, m->taskID, m->duration )` (`g_main.c:598-603`).
impl Dispatch<GameIcarusLerp2Start> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.lerp2start();
        let (ent_id, task_id, duration) = (m.entID, m.taskID, m.duration);
        Q3_Lerp2Start(self, ent_id, task_id, duration);
    }
}

/// `GAME_ICARUS_LERP2END` → `Q3_Lerp2End( m->entID, m->taskID, m->duration )` (`g_main.c:604-609`).
impl Dispatch<GameIcarusLerp2End> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.lerp2end();
        let (ent_id, task_id, duration) = (m.entID, m.taskID, m.duration);
        Q3_Lerp2End(self, ent_id, task_id, duration);
    }
}

/// `GAME_ICARUS_USE` → `Q3_Use( m->entID, m->target )` (`g_main.c:610-615`).
impl Dispatch<GameIcarusUse> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.use_cmd();
        let ent_id = m.entID;
        let target = m.target.as_ptr() as *const c_char;
        Q3_Use(self, ent_id, target);
    }
}

/// `GAME_ICARUS_KILL` → `Q3_Kill( m->entID, m->name )` (`g_main.c:616-621`).
impl Dispatch<GameIcarusKill> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.kill();
        let ent_id = m.entID;
        let name = m.name.as_ptr() as *const c_char;
        Q3_Kill(self, ent_id, name);
    }
}

/// `GAME_ICARUS_REMOVE` → `Q3_Remove( m->entID, m->name )` (`g_main.c:622-627`).
impl Dispatch<GameIcarusRemove> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.remove();
        let ent_id = m.entID;
        let name = m.name.as_ptr() as *const c_char;
        Q3_Remove(self, ent_id, name);
    }
}

/// `GAME_ICARUS_PLAY` → `Q3_Play( m->taskID, m->entID, m->type, m->name )` (`g_main.c:628-633`).
impl Dispatch<GameIcarusPlay> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.play();
        let (task_id, ent_id) = (m.taskID, m.entID);
        let ty = m.r#type.as_ptr() as *const c_char;
        let name = m.name.as_ptr() as *const c_char;
        Q3_Play(self, task_id, ent_id, ty, name);
    }
}

/// `GAME_ICARUS_GETFLOAT` → `return Q3_GetFloat( m->entID, m->type, m->name, &m->value )` (`g_main.c:634-638`).
impl Dispatch<GameIcarusGetfloat> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> c_int {
        let m = self.world.gSharedBuffer.getfloat();
        let (ent_id, ty) = (m.entID, m.r#type);
        let name = m.name.as_ptr() as *const c_char;
        // `value` is an out-param. Copy it in, call over a local, then write back.
        // This is equivalent to the in-place overlay write.
        let mut value = m.value;
        let r = Q3_GetFloat(self, ent_id, ty, name, &mut value as *mut f32);
        self.world.gSharedBuffer.getfloat().value = value;
        r
    }
}

/// `GAME_ICARUS_GETVECTOR` → `return Q3_GetVector( m->entID, m->type, m->name, m->value )` (`g_main.c:639-643`).
impl Dispatch<GameIcarusGetvector> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> c_int {
        let m = self.world.gSharedBuffer.getvector();
        let (ent_id, ty) = (m.entID, m.r#type);
        let name = m.name.as_ptr() as *const c_char;
        // `value` is an out-param. Copy it in, call over a local, then write back.
        // This is equivalent to the in-place overlay write.
        let mut value = m.value;
        let r = Q3_GetVector(self, ent_id, ty, name, &mut value);
        self.world.gSharedBuffer.getvector().value = value;
        r
    }
}

/// `GAME_ICARUS_GETSTRING` → `Q3_GetString( m->entID, m->type, m->name, &crap )`,
/// then `if (crap) strcpy( m->value, crap )` (`g_main.c:644-658`).
/// This impl returns the `Q3_GetString` result.
impl Dispatch<GameIcarusGetstring> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> c_int {
        let m = self.world.gSharedBuffer.getstring();
        let (ent_id, ty) = (m.entID, m.r#type);
        let name = m.name.as_ptr() as *const c_char;
        // This follows Raven's `char *crap = NULL; char **morecrap = &crap;` out-pointer pattern.
        let mut crap: *mut c_char = core::ptr::null_mut();
        let morecrap = &mut crap as *mut *mut c_char;
        let r = Q3_GetString(self, ent_id, ty, name, morecrap);
        if !crap.is_null() {
            // SAFETY: on success, `crap` points at a valid NUL-terminated string.
            // `m->value` is a 2048-byte buffer (`g_public.h`).
            // The `c_strcpy` write-back stays raw by design, for a later cleanup pass.
            let value = self.world.gSharedBuffer.getstring().value.as_mut_ptr() as *mut c_char;
            unsafe { c_strcpy(value, crap) };
        }
        r
    }
}

/// `GAME_ICARUS_SOUNDINDEX` → `G_SoundIndex( m->filename )` (`g_main.c:659-664`).
impl Dispatch<GameIcarusSoundindex> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) {
        let m = self.world.gSharedBuffer.soundindex();
        let filename = m.filename.as_ptr() as *const c_char;
        // Raven discards `G_SoundIndex`'s return handle here, because only the registration side effect matters.
        G_SoundIndex(self, &(unsafe { cstr_to_str(filename) }));
    }
}

/// `GAME_ICARUS_GETSETIDFORSTRING` → `return GetIDForString( setTable, m->string )` (`g_main.c:665-669`).
impl Dispatch<GameIcarusGetsetidforstring> for GameContext<'_> {
    fn dispatch(&mut self, _args: ()) -> c_int {
        let m = self.world.gSharedBuffer.getsetidforstring();
        let string = unsafe { cstr_to_str(m.string.as_ptr() as *const c_char) };
        GetIDForString(setTable.as_ptr() as *mut stringID_table_t, &string)
    }
}
