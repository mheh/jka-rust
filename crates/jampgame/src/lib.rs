//! `jampgame` — the MP game module cdylib shell (SEAM-D10). Thin: hosts the
//! `ENGINE: OnceLock<CEngine>` static (SEAM-D1), the `WORLD: WorldCell` static
//! (STATE-D6), the live entrypoint exports, and the `vmMain` export-enum match
//! that delegates into `mp_game` (`GameContext` receiver, SEAM-Q12). The logic
//! crate `mp_game` has no entrypoint/`OnceLock`/`WorldCell` code of its own.

use std::sync::OnceLock;

mod world_cell;

use abi_transport::entrypoints::{AbiCommand, AbiWord, RawExportTable, RawImportTable, RawSyscall};
use abi_transport::generic::engine::CEngine;
use abi_transport::generic::{DecodeVmMain, Dispatch, EncodeVmMainReturn, VmMainTransport};
use mp_game::vmcalls::{
    BotAiStartFrame, GameClientBegin, GameClientCommand, GameClientConnect, GameClientDisconnect,
    GameClientThink, GameClientUserinfoChanged, GameConsoleCommand, GameGetitemindexbytag,
    GameInit, GameNavChecknodefailedforent, GameNavClearlos, GameNavClearpathbetweenpoints,
    GameNavClearpathtopoint, GameNavEntIsBreakable, GameNavEntIsDoor, GameNavEntIsRemovableUsable,
    GameNavEntIsUnlockedDoor, GameNavFindcombatpointwaypoints, GameRoffNotetrackCallback,
    GameRunFrame, GameShutdown, GameSpawnRmgEntity,
};
use mp_game::{GameContext, GameWorld, MpGameExport};

use crate::world_cell::WorldCell;

/// The single outbound-syscall backend seam global (SEAM-D1, porting-rules §B6
/// exception — `vmMain` takes no context argument). Set once at `dllEntry`.
static ENGINE: OnceLock<CEngine> = OnceLock::new();

/// The module island's one owned `GameWorld` across `vmMain` calls (STATE-D6,
/// the second sanctioned static exemption). `None` until `GAME_INIT` builds it.
static WORLD: WorldCell = WorldCell::new();

/// Raven `dllEntry` (`g_syscalls.c:14-16`). Stores the engine syscall trampoline
/// into the one `OnceLock<CEngine>`. `extern "C-unwind"` (SEAM-D12).
#[no_mangle]
pub extern "C-unwind" fn dllEntry(syscall: RawSyscall) {
    ENGINE.set(CEngine::new(syscall)).ok();
}

/// Raven `vmMain` (`g_main.c:515`). Bootstraps/derives the `WORLD` pointer,
/// constructs a `GameContext` per call from `WORLD` + `ENGINE.get()` (SEAM-Q12),
/// and routes the decoded `MpGameExport` command through the exhaustive match to
/// its `Dispatch<C>` impl. `extern "C-unwind"` (SEAM-D12).
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C-unwind" fn vmMain(
    command: AbiCommand,
    arg0: AbiWord,
    arg1: AbiWord,
    arg2: AbiWord,
    arg3: AbiWord,
    arg4: AbiWord,
    arg5: AbiWord,
    arg6: AbiWord,
    arg7: AbiWord,
    arg8: AbiWord,
    arg9: AbiWord,
    arg10: AbiWord,
    arg11: AbiWord,
) -> AbiWord {
    // BOOTSTRAP (STATE-D6): GAME_INIT is the ONE command that WRITES the cell
    // before reading it — it stores a zeroed GameWorld (GameWorld::zeroed,
    // STATE-D9), THEN falls through so the dispatched GAME_INIT arm runs
    // G_InitGame's init against it (g_main.c:515,979). The pre-decode compare
    // is the frozen round-6 pinning spelling.
    if command == MpGameExport::GAME_INIT as AbiCommand {
        // SAFETY: single-threaded init; no reentrancy is possible before the
        // world exists (STATE-D6).
        unsafe {
            *WORLD.0.get() = Some(GameWorld::zeroed());
        }
    }

    // SAFETY: single-threaded per Raven's contract; each (possibly reentrant)
    // entry derives its OWN raw `*mut GameWorld` — aliasing raw pointers are
    // sound; a dispatch-spanning `&mut` would be UB (STATE-D6 discipline).
    let world = unsafe {
        (*WORLD.0.get())
            .as_mut()
            .expect("GAME_INIT built the world") as *mut GameWorld
    };
    // Per-call receiver from WORLD + ENGINE.get() (SEAM-Q12) — plain struct
    // literal, pub fields (round-5 resolution; WorldPtr precedent, STATE-D8).
    let ctx = GameContext {
        world,
        engine: ENGINE.get().expect("dllEntry set ENGINE"),
    };

    let transport = VmMainTransport::new([
        arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
    ]);

    // Fallible pre-decode (SEAM-D6): an unrecognized command word reproduces
    // Raven's fall-through `return -1` (g_main.c:695) at the conversion's Err,
    // not in a match arm; the match stays exhaustive over the valid variants.
    let Ok(export) = MpGameExport::try_from(command) else {
        return -1;
    };

    // The inline exhaustive export-enum dispatch match (SEAM-D3/D8; round-6
    // pinning — mirrors the outbound sv_game_system_calls match shape). Each
    // arm: decode via DecodeVmMain → route to the command's Dispatch<C> impl →
    // encode via EncodeVmMainReturn. Unimplemented arms: todo!("Port <cmd>").
    let result = match export {
        // `case GAME_INIT: G_InitGame( arg0, arg1, arg2 ); return 0;`
        // (g_main.c:517-519).
        MpGameExport::GAME_INIT => GameInit::encode_return(Dispatch::<GameInit>::dispatch(
            &ctx,
            GameInit::decode_vm_main(transport),
        )),
        // `case GAME_SHUTDOWN: G_ShutdownGame( arg0 ); return 0;`
        // (g_main.c:520-522).
        MpGameExport::GAME_SHUTDOWN => GameShutdown::encode_return(
            Dispatch::<GameShutdown>::dispatch(&ctx, GameShutdown::decode_vm_main(transport)),
        ),
        // `case GAME_CLIENT_CONNECT: return (int)ClientConnect( arg0, arg1, arg2 );`
        // (g_main.c:523-524).
        MpGameExport::GAME_CLIENT_CONNECT => {
            GameClientConnect::encode_return(Dispatch::<GameClientConnect>::dispatch(
                &ctx,
                GameClientConnect::decode_vm_main(transport),
            ))
        }
        // `case GAME_CLIENT_THINK: ClientThink( arg0, NULL ); return 0;`
        // (g_main.c:525-527).
        MpGameExport::GAME_CLIENT_THINK => {
            GameClientThink::encode_return(Dispatch::<GameClientThink>::dispatch(
                &ctx,
                GameClientThink::decode_vm_main(transport),
            ))
        }
        // `case GAME_CLIENT_USERINFO_CHANGED: ClientUserinfoChanged( arg0 ); return 0;`
        // (g_main.c:528-530).
        MpGameExport::GAME_CLIENT_USERINFO_CHANGED => GameClientUserinfoChanged::encode_return(
            Dispatch::<GameClientUserinfoChanged>::dispatch(
                &ctx,
                GameClientUserinfoChanged::decode_vm_main(transport),
            ),
        ),
        // `case GAME_CLIENT_DISCONNECT: ClientDisconnect( arg0 ); return 0;`
        // (g_main.c:531-533).
        MpGameExport::GAME_CLIENT_DISCONNECT => {
            GameClientDisconnect::encode_return(Dispatch::<GameClientDisconnect>::dispatch(
                &ctx,
                GameClientDisconnect::decode_vm_main(transport),
            ))
        }
        // `case GAME_CLIENT_BEGIN: ClientBegin( arg0, qtrue ); return 0;`
        // (g_main.c:534-536).
        MpGameExport::GAME_CLIENT_BEGIN => {
            GameClientBegin::encode_return(Dispatch::<GameClientBegin>::dispatch(
                &ctx,
                GameClientBegin::decode_vm_main(transport),
            ))
        }
        // `case GAME_CLIENT_COMMAND: ClientCommand( arg0 ); return 0;`
        // (g_main.c:537-539).
        MpGameExport::GAME_CLIENT_COMMAND => {
            GameClientCommand::encode_return(Dispatch::<GameClientCommand>::dispatch(
                &ctx,
                GameClientCommand::decode_vm_main(transport),
            ))
        }
        // `case GAME_RUN_FRAME: G_RunFrame( arg0 ); return 0;` (g_main.c:540-542).
        MpGameExport::GAME_RUN_FRAME => GameRunFrame::encode_return(
            Dispatch::<GameRunFrame>::dispatch(&ctx, GameRunFrame::decode_vm_main(transport)),
        ),
        // `case GAME_CONSOLE_COMMAND: return ConsoleCommand();` (g_main.c:543-544).
        MpGameExport::GAME_CONSOLE_COMMAND => {
            GameConsoleCommand::encode_return(Dispatch::<GameConsoleCommand>::dispatch(
                &ctx,
                GameConsoleCommand::decode_vm_main(transport),
            ))
        }
        // `case BOTAI_START_FRAME: return BotAIStartFrame( arg0 );` (g_main.c:545-546).
        MpGameExport::BOTAI_START_FRAME => {
            BotAiStartFrame::encode_return(Dispatch::<BotAiStartFrame>::dispatch(
                &ctx,
                BotAiStartFrame::decode_vm_main(transport),
            ))
        }
        // `case GAME_ROFF_NOTETRACK_CALLBACK:
        //   G_ROFF_NotetrackCallback( &g_entities[arg0], (const char *)arg1 ); return 0;`
        // (g_main.c:547-549).
        MpGameExport::GAME_ROFF_NOTETRACK_CALLBACK => GameRoffNotetrackCallback::encode_return(
            Dispatch::<GameRoffNotetrackCallback>::dispatch(
                &ctx,
                GameRoffNotetrackCallback::decode_vm_main(transport),
            ),
        ),
        // `case GAME_SPAWN_RMG_ENTITY:
        //   if (G_ParseSpawnVars(qfalse)) G_SpawnGEntityFromSpawnVars(qfalse); return 0;`
        // (g_main.c:550-555).
        MpGameExport::GAME_SPAWN_RMG_ENTITY => {
            GameSpawnRmgEntity::encode_return(Dispatch::<GameSpawnRmgEntity>::dispatch(
                &ctx,
                GameSpawnRmgEntity::decode_vm_main(transport),
            ))
        }
        // ESCALATION: the 17 ICARUS callback cases (g_main.c:558-668) read their
        // `T_G_ICARUS_*` payloads out of the module's `gSharedBuffer`
        // shared-memory region, which is not yet modeled in `GameWorld`. Wiring
        // them needs a design decision (where the registered buffer lives and how
        // it is typed at the seam), so they stay a single loud todo!() rather than
        // a fake. Source: oracle/oracle/codemp/game/g_main.c:558-668.
        MpGameExport::GAME_ICARUS_PLAYSOUND
        | MpGameExport::GAME_ICARUS_SET
        | MpGameExport::GAME_ICARUS_LERP2POS
        | MpGameExport::GAME_ICARUS_LERP2ORIGIN
        | MpGameExport::GAME_ICARUS_LERP2ANGLES
        | MpGameExport::GAME_ICARUS_GETTAG
        | MpGameExport::GAME_ICARUS_LERP2START
        | MpGameExport::GAME_ICARUS_LERP2END
        | MpGameExport::GAME_ICARUS_USE
        | MpGameExport::GAME_ICARUS_KILL
        | MpGameExport::GAME_ICARUS_REMOVE
        | MpGameExport::GAME_ICARUS_PLAY
        | MpGameExport::GAME_ICARUS_GETFLOAT
        | MpGameExport::GAME_ICARUS_GETVECTOR
        | MpGameExport::GAME_ICARUS_GETSTRING
        | MpGameExport::GAME_ICARUS_SOUNDINDEX
        | MpGameExport::GAME_ICARUS_GETSETIDFORSTRING => {
            todo!(
                "Port GAME_ICARUS_* dispatch — gSharedBuffer module shared-memory \
                 transport is unmodeled (T_G_ICARUS_* payloads); \
                 oracle/oracle/codemp/game/g_main.c:558-668"
            )
        }
        // `case GAME_NAV_CLEARPATHTOPOINT: return NAV_ClearPathToPoint(...);`
        // (g_main.c:672-673).
        MpGameExport::GAME_NAV_CLEARPATHTOPOINT => {
            GameNavClearpathtopoint::encode_return(Dispatch::<GameNavClearpathtopoint>::dispatch(
                &ctx,
                GameNavClearpathtopoint::decode_vm_main(transport),
            ))
        }
        // `case GAME_NAV_CLEARLOS: return NPC_ClearLOS2(...);` (g_main.c:674-675).
        MpGameExport::GAME_NAV_CLEARLOS => {
            GameNavClearlos::encode_return(Dispatch::<GameNavClearlos>::dispatch(
                &ctx,
                GameNavClearlos::decode_vm_main(transport),
            ))
        }
        // `case GAME_NAV_CLEARPATHBETWEENPOINTS: return NAVNEW_ClearPathBetweenPoints(...);`
        // (g_main.c:676-677).
        MpGameExport::GAME_NAV_CLEARPATHBETWEENPOINTS => {
            GameNavClearpathbetweenpoints::encode_return(
                Dispatch::<GameNavClearpathbetweenpoints>::dispatch(
                    &ctx,
                    GameNavClearpathbetweenpoints::decode_vm_main(transport),
                ),
            )
        }
        // `case GAME_NAV_CHECKNODEFAILEDFORENT: return NAV_CheckNodeFailedForEnt(...);`
        // (g_main.c:678-679).
        MpGameExport::GAME_NAV_CHECKNODEFAILEDFORENT => {
            GameNavChecknodefailedforent::encode_return(
                Dispatch::<GameNavChecknodefailedforent>::dispatch(
                    &ctx,
                    GameNavChecknodefailedforent::decode_vm_main(transport),
                ),
            )
        }
        // `case GAME_NAV_ENTISUNLOCKEDDOOR: return G_EntIsUnlockedDoor(arg0);`
        // (g_main.c:680-681).
        MpGameExport::GAME_NAV_ENTISUNLOCKEDDOOR => {
            GameNavEntIsUnlockedDoor::encode_return(Dispatch::<GameNavEntIsUnlockedDoor>::dispatch(
                &ctx,
                GameNavEntIsUnlockedDoor::decode_vm_main(transport),
            ))
        }
        // `case GAME_NAV_ENTISDOOR: return G_EntIsDoor(arg0);` (g_main.c:682-683).
        MpGameExport::GAME_NAV_ENTISDOOR => {
            GameNavEntIsDoor::encode_return(Dispatch::<GameNavEntIsDoor>::dispatch(
                &ctx,
                GameNavEntIsDoor::decode_vm_main(transport),
            ))
        }
        // `case GAME_NAV_ENTISBREAKABLE: return G_EntIsBreakable(arg0);`
        // (g_main.c:684-685).
        MpGameExport::GAME_NAV_ENTISBREAKABLE => {
            GameNavEntIsBreakable::encode_return(Dispatch::<GameNavEntIsBreakable>::dispatch(
                &ctx,
                GameNavEntIsBreakable::decode_vm_main(transport),
            ))
        }
        // `case GAME_NAV_ENTISREMOVABLEUSABLE: return G_EntIsRemovableUsable(arg0);`
        // (g_main.c:686-687).
        MpGameExport::GAME_NAV_ENTISREMOVABLEUSABLE => GameNavEntIsRemovableUsable::encode_return(
            Dispatch::<GameNavEntIsRemovableUsable>::dispatch(
                &ctx,
                GameNavEntIsRemovableUsable::decode_vm_main(transport),
            ),
        ),
        // `case GAME_NAV_FINDCOMBATPOINTWAYPOINTS: CP_FindCombatPointWaypoints(); return 0;`
        // (g_main.c:688-689).
        MpGameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS => {
            GameNavFindcombatpointwaypoints::encode_return(Dispatch::<
                GameNavFindcombatpointwaypoints,
            >::dispatch(
                &ctx,
                GameNavFindcombatpointwaypoints::decode_vm_main(transport),
            ))
        }
        // `case GAME_GETITEMINDEXBYTAG: return BG_GetItemIndexByTag(arg0, arg1);`
        // (g_main.c:690-691).
        MpGameExport::GAME_GETITEMINDEXBYTAG => {
            GameGetitemindexbytag::encode_return(Dispatch::<GameGetitemindexbytag>::dispatch(
                &ctx,
                GameGetitemindexbytag::decode_vm_main(transport),
            ))
        }
    };

    // GAME_SHUTDOWN takes the world OUT of the cell AFTER its dispatch returns
    // — module-unload lifetime; dropping the Some(GameWorld) runs the owned
    // island's Drop (§C9, STATE-D6).
    if command == MpGameExport::GAME_SHUTDOWN as AbiCommand {
        // SAFETY: single-threaded; the just-returned GAME_SHUTDOWN dispatch
        // holds no live borrow (STATE-D6).
        unsafe {
            *WORLD.0.get() = None;
        }
    }
    result
}

/// OpenJK-only `GetModuleAPI` handshake (SEAM-Q7 open — zero oracle occurrences).
/// Slice 0 does not touch it; stays a null stub.
#[no_mangle]
pub extern "C-unwind" fn GetModuleAPI(
    _api_version: AbiCommand,
    _import: RawImportTable,
) -> RawExportTable {
    //TODO: Port GetModuleAPI — contract is SEAM-Q7 (open)
    // Source: docs/architecture/engine-seam.md § Live entrypoint exports (SEAM-Q7)
    core::ptr::null_mut()
}
