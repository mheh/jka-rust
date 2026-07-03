//! `jampgame` — the MP game module cdylib shell (SEAM-D10). Thin: hosts the
//! `ENGINE: OnceLock<CEngine>` static (SEAM-D1), the `WORLD: WorldCell` static
//! (STATE-D6), the live entrypoint exports, and the `vmMain` export-enum match
//! that delegates into `mp_game` (`GameContext` receiver, SEAM-Q12). The logic
//! crate `mp_game` has no entrypoint/`OnceLock`/`WorldCell` code of its own.

use std::sync::OnceLock;

mod world_cell;

use abi_transport::entrypoints::{AbiCommand, AbiWord, RawExportTable, RawImportTable, RawSyscall};
use abi_transport::generic::engine::CEngine;
use mp_game::GameWorld;

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
    let _ = (
        command, arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
    );
    let _ = (&ENGINE, WORLD.0.get());
    let _construct = |w: *mut GameWorld| {
        // Per-call receiver from WORLD + ENGINE.get() (SEAM-Q12) — plain struct
        // literal, pub fields (round-5 resolution: no invariant on a Copy
        // struct of raw pointers; WorldPtr precedent, STATE-D8).
        mp_game::GameContext {
            world: w,
            engine: ENGINE.get().expect("dllEntry set ENGINE"),
        }
    };
    todo!("Port jampgame vmMain export-enum match — oracle/oracle/codemp/game/g_main.c:515")
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
