//! `jampgame` — the MP game module cdylib shell (SEAM-D10). Thin: hosts the
//! `ENGINE: OnceLock<CEngine>` static (SEAM-D1), the `WORLD: WorldCell` static
//! (STATE-D6), the live entrypoint exports, and the `vmMain` export-enum match
//! that delegates into `mp_game` (`GameContext` receiver, SEAM-Q12). The logic
//! crate `mp_game` has no entrypoint/`OnceLock`/`WorldCell` code of its own.

use std::ffi::{c_char, CString};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

mod panic_guard;
mod world_cell;

use abi_transport::entrypoints::{AbiCommand, AbiWord, RawSyscall};
use abi_transport::generic::engine::CEngine;
use abi_transport::generic::{DecodeVmMain, Dispatch, EncodeVmMainReturn, VmMainTransport};
use mp_game::com_boundary::{route_error, route_print, set_com_error_sink, set_com_print_sink};
use mp_game::vmcalls::{
    BotAiStartFrame, GameClientBegin, GameClientCommand, GameClientConnect, GameClientDisconnect,
    GameClientThink, GameClientUserinfoChanged, GameConsoleCommand, GameGetitemindexbytag,
    GameIcarusGetfloat, GameIcarusGetsetidforstring, GameIcarusGetstring, GameIcarusGettag,
    GameIcarusGetvector, GameIcarusKill, GameIcarusLerp2Angles, GameIcarusLerp2End,
    GameIcarusLerp2Origin, GameIcarusLerp2Pos, GameIcarusLerp2Start, GameIcarusPlay,
    GameIcarusPlaysound, GameIcarusRemove, GameIcarusSet, GameIcarusSoundindex, GameIcarusUse,
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
///
/// PANIC POLICY (2026-07-08): `dllEntry` runs BEFORE the engine syscall pointer
/// is armed, so there is no `Com_Error`/`G_ERROR` path to route a failure
/// through yet. Its only sound failure mode is `eprintln!` + `std::process::
/// abort()` — a panic must never unwind raw across the `extern "C-unwind"`
/// boundary into the C engine (UB). The capture hook is installed FIRST so any
/// panic in the remaining setup is still recorded with `file:line`.
#[no_mangle]
pub extern "C-unwind" fn dllEntry(syscall: RawSyscall) {
    let armed = std::panic::catch_unwind(AssertUnwindSafe(|| {
        panic_guard::install_hook();
        ENGINE.set(CEngine::new(syscall)).ok();
        set_com_print_sink(com_print_sink);
        set_com_error_sink(com_error_sink);
    }));
    if armed.is_err() {
        let msg = panic_guard::take().unwrap_or_else(|| "panic in dllEntry".to_string());
        eprintln!("jampgame: fatal panic during dllEntry (engine error path not yet armed): {msg}");
        std::process::abort();
    }
}

/// The registered `Com_Printf` route (SEAM-D1 narrow extension): reads the
/// one `ENGINE` static and forwards through `trap_Printf` exactly like
/// `G_Printf` does.
fn com_print_sink(msg: *const c_char) {
    let engine = ENGINE.get().expect("dllEntry set ENGINE");
    route_print(engine, msg);
}

/// The registered `Com_Error` route (SEAM-D1 narrow extension): reads the
/// one `ENGINE` static and forwards through `trap_Error` exactly like
/// `G_Error` does.
fn com_error_sink(msg: *const c_char) {
    let engine = ENGINE.get().expect("dllEntry set ENGINE");
    route_error(engine, msg);
}

/// Raven `vmMain` (`g_main.c:515`) — the single engine→game choke point AND the
/// ABI panic firewall (panic policy, 2026-07-08). Every engine call flows
/// through here; the decoded dispatch itself lives in [`vm_main_dispatch`],
/// wrapped in `std::panic::catch_unwind` so NO Rust panic ever unwinds raw
/// across the `extern "C-unwind"` boundary into the C engine (that is UB, or at
/// best an abort with no context). On a caught panic we report a readable
/// `file:line — message` through the engine's `Com_Error`/`G_ERROR` path (see
/// [`report_panic_and_die`], which then never returns). `extern "C-unwind"`
/// (SEAM-D12).
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
    let dispatched = std::panic::catch_unwind(AssertUnwindSafe(|| {
        vm_main_dispatch(
            command, arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
        )
    }));
    match dispatched {
        Ok(word) => word,
        Err(_) => {
            // The hook (installed at dllEntry) already recorded the payload +
            // file:line into a thread-local — catch_unwind's `Any` is lossy, so
            // prefer that record. report_panic_and_die never returns.
            let msg = panic_guard::take()
                .unwrap_or_else(|| "panic with no captured payload/location".to_string());
            report_panic_and_die(&msg)
        }
    }
}

/// Set for the duration of a panic report. A second panic reaching
/// [`report_panic_and_die`] while this is set means the engine error path
/// itself failed, so we abort rather than loop forever.
static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Report a panic caught at [`vmMain`] through the engine's `Com_Error`/
/// `G_ERROR` path, then never return.
///
/// The engine's `Com_Error` longjmps back into the engine and NEVER returns —
/// that is the exact contract C game code relies on when it calls `G_Error`
/// (`oracle/codemp/game/g_main.c:1208`). So on success this call does not
/// come back; the game dies with a readable message, same as a C-side fatal.
/// If the error trap is unavailable (engine pointer not yet armed, or the
/// message held an interior NUL), or the trap itself panics, or we are already
/// inside a panic report (recursion), there is nothing sound left to do but
/// `std::process::abort()` — after printing `file:line — reason` so the operator
/// still gets context.
fn report_panic_and_die(msg: &str) -> ! {
    if PANIC_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        eprintln!("jampgame: recursive panic while reporting a panic; aborting");
        std::process::abort();
    }
    let full = format!("jampgame panic: {msg}");
    if ENGINE.get().is_some() {
        if let Ok(cmsg) = CString::new(full.clone()) {
            // Route through Com_Error (G_ERROR): longjmps, never returns.
            // Guarded so a panic *inside* the trap falls through to abort
            // instead of unwinding across the boundary a second time.
            let _ = std::panic::catch_unwind(|| com_error_sink(cmsg.as_ptr()));
        }
    }
    // Reached only if the engine error path was unavailable or (impossibly)
    // returned. Print and abort — never let the panic escape the boundary.
    eprintln!("{full}");
    eprintln!("jampgame: engine error path unavailable or returned; aborting");
    std::process::abort();
}

/// The decoded-dispatch body of [`vmMain`], factored out so the export can wrap
/// it in `catch_unwind`. Bootstraps/derives the `WORLD` pointer, constructs a
/// `GameContext` per call from `WORLD` + `ENGINE.get()` (SEAM-Q12), and routes
/// the decoded `MpGameExport` command through the exhaustive match to its
/// `Dispatch<C>` impl.
#[allow(clippy::too_many_arguments)]
fn vm_main_dispatch(
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
    // before reading it — it stores a heap-boxed zeroed GameWorld
    // (GameWorld::zeroed_boxed, STATE-D9), THEN falls through so the dispatched
    // GAME_INIT arm runs G_InitGame's init against it (g_main.c:515,979).
    // `zeroed_boxed` builds the ~1.4 MB island directly on the heap so it never
    // transits this deep engine-called frame by value (an inline
    // `Some(GameWorld::zeroed())` overflowed the guard page). The pre-decode
    // compare is the frozen round-6 pinning spelling.
    if command == MpGameExport::GAME_INIT as AbiCommand {
        // SAFETY: single-threaded init; no reentrancy is possible before the
        // world exists (STATE-D6).
        unsafe {
            *WORLD.0.get() = Some(GameWorld::zeroed_boxed());
        }
    }

    // SAFETY: single-threaded per Raven's contract; each (possibly reentrant)
    // entry derives its OWN raw `*mut GameWorld` — aliasing raw pointers are
    // sound; a dispatch-spanning `&mut` would be UB (STATE-D6 discipline). The
    // cell holds `Box<GameWorld>`; `&mut **b` reborrows through the Box to the
    // same `*mut GameWorld` the whole downstream (`GameContext`) expects.
    let world = unsafe {
        let b = (*WORLD.0.get())
            .as_mut()
            .expect("GAME_INIT built the world");
        &mut **b as *mut GameWorld
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
        // The 17 ICARUS callback cases (g_main.c:558-668). Each reads its
        // `T_G_ICARUS_*` payload out of the engine-registered `gSharedBuffer`
        // shared-memory region (registered in G_InitGame via
        // trap::SV_RegisterSharedMemory); the per-command `Dispatch<C> for
        // GameContext` impls (game_context.rs) overlay-cast that buffer and thread
        // into the ported Q3_* handlers. `Args = ()` — the payload arrives
        // out-of-band, not through the vmMain arg words.
        // Source: oracle/codemp/game/g_main.c:558-668.
        MpGameExport::GAME_ICARUS_PLAYSOUND => {
            GameIcarusPlaysound::encode_return(Dispatch::<GameIcarusPlaysound>::dispatch(
                &ctx,
                GameIcarusPlaysound::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_SET => GameIcarusSet::encode_return(
            Dispatch::<GameIcarusSet>::dispatch(&ctx, GameIcarusSet::decode_vm_main(transport)),
        ),
        MpGameExport::GAME_ICARUS_LERP2POS => {
            GameIcarusLerp2Pos::encode_return(Dispatch::<GameIcarusLerp2Pos>::dispatch(
                &ctx,
                GameIcarusLerp2Pos::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_LERP2ORIGIN => {
            GameIcarusLerp2Origin::encode_return(Dispatch::<GameIcarusLerp2Origin>::dispatch(
                &ctx,
                GameIcarusLerp2Origin::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_LERP2ANGLES => {
            GameIcarusLerp2Angles::encode_return(Dispatch::<GameIcarusLerp2Angles>::dispatch(
                &ctx,
                GameIcarusLerp2Angles::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_GETTAG => {
            GameIcarusGettag::encode_return(Dispatch::<GameIcarusGettag>::dispatch(
                &ctx,
                GameIcarusGettag::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_LERP2START => {
            GameIcarusLerp2Start::encode_return(Dispatch::<GameIcarusLerp2Start>::dispatch(
                &ctx,
                GameIcarusLerp2Start::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_LERP2END => {
            GameIcarusLerp2End::encode_return(Dispatch::<GameIcarusLerp2End>::dispatch(
                &ctx,
                GameIcarusLerp2End::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_USE => GameIcarusUse::encode_return(
            Dispatch::<GameIcarusUse>::dispatch(&ctx, GameIcarusUse::decode_vm_main(transport)),
        ),
        MpGameExport::GAME_ICARUS_KILL => GameIcarusKill::encode_return(
            Dispatch::<GameIcarusKill>::dispatch(&ctx, GameIcarusKill::decode_vm_main(transport)),
        ),
        MpGameExport::GAME_ICARUS_REMOVE => {
            GameIcarusRemove::encode_return(Dispatch::<GameIcarusRemove>::dispatch(
                &ctx,
                GameIcarusRemove::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_PLAY => GameIcarusPlay::encode_return(
            Dispatch::<GameIcarusPlay>::dispatch(&ctx, GameIcarusPlay::decode_vm_main(transport)),
        ),
        MpGameExport::GAME_ICARUS_GETFLOAT => {
            GameIcarusGetfloat::encode_return(Dispatch::<GameIcarusGetfloat>::dispatch(
                &ctx,
                GameIcarusGetfloat::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_GETVECTOR => {
            GameIcarusGetvector::encode_return(Dispatch::<GameIcarusGetvector>::dispatch(
                &ctx,
                GameIcarusGetvector::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_GETSTRING => {
            GameIcarusGetstring::encode_return(Dispatch::<GameIcarusGetstring>::dispatch(
                &ctx,
                GameIcarusGetstring::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_SOUNDINDEX => {
            GameIcarusSoundindex::encode_return(Dispatch::<GameIcarusSoundindex>::dispatch(
                &ctx,
                GameIcarusSoundindex::decode_vm_main(transport),
            ))
        }
        MpGameExport::GAME_ICARUS_GETSETIDFORSTRING => GameIcarusGetsetidforstring::encode_return(
            Dispatch::<GameIcarusGetsetidforstring>::dispatch(
                &ctx,
                GameIcarusGetsetidforstring::decode_vm_main(transport),
            ),
        ),
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

// `GetModuleAPI` is deliberately NOT exported (SEAM-Q7 ruling, 2026-07-06).
// OpenJK's loader (sv_gameapi.cpp SV_BindGame) treats a present-but-NULL-
// returning symbol as ERR_FATAL with no fallback; with the symbol absent it
// falls back to the legacy `dllEntry`/`vmMain` path, whose widened intptr_t
// signatures the exports above already match. Modern-path implementation is
// tracked in https://github.com/mheh/jka-rust/issues/1.
