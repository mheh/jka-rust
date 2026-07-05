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
use mp_game::vmcalls::{GameInit, GameShutdown};
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
        (*WORLD.0.get()).as_mut().expect("GAME_INIT built the world") as *mut GameWorld
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
        MpGameExport::GAME_INIT => {
            GameInit::encode_return(Dispatch::<GameInit>::dispatch(
                &ctx,
                GameInit::decode_vm_main(transport),
            ))
        }
        // `case GAME_SHUTDOWN: G_ShutdownGame( arg0 ); return 0;`
        // (g_main.c:520-522).
        MpGameExport::GAME_SHUTDOWN => {
            GameShutdown::encode_return(Dispatch::<GameShutdown>::dispatch(
                &ctx,
                GameShutdown::decode_vm_main(transport),
            ))
        }
        other => todo!("Port vmMain command {other:?} — oracle/oracle/codemp/game/g_main.c:515"),
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
