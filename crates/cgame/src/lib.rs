//! `cgame` — the MP cgame module cdylib shell (SEAM-D10). Thin: hosts the
//! `ENGINE: OnceLock<CEngine>` static (SEAM-D1), the `WORLD: WorldCell`
//! static (STATE-D6), the live entrypoint exports, and the `vmMain` export
//! forwarding into `mp_cgame::cg_main::vmMain` — which owns the
//! `MpCgameExport` dispatch, splits the `CgState` into its three disjoint
//! borrows and builds the `CgContext` receiver (DEC-47.1, the DEC-38 shape).
//! The logic crate `mp_cgame` has no entrypoint/`OnceLock`/`WorldCell` code of
//! its own. Structure, naming and safety comments mirror the `ui` shell
//! (`crates/ui/src/lib.rs`) exactly except where cgame's own contract
//! genuinely differs — each such spot is called out at the divergence.

use std::panic::AssertUnwindSafe;
use std::sync::OnceLock;

mod panic_guard;
mod world_cell;

use abi_transport::entrypoints::{AbiCommand, AbiWord, RawSyscall};
use abi_transport::generic::engine::CEngine;
use mp_cgame::cg_main::vmMain as mp_cgame_vm_main;
use mp_cgame::world::cg_state::CgState;

use crate::world_cell::WorldCell;

/// The single outbound-syscall backend seam global (SEAM-D1, porting-rules §B6
/// exception — `vmMain` takes no context argument). Set once at `dllEntry`.
static ENGINE: OnceLock<CEngine> = OnceLock::new();

/// The module island's one owned `CgState` across `vmMain` calls (STATE-D6,
/// the second sanctioned static exemption). Bootstrapped lazily on the first
/// `vmMain` call of any kind (see `vm_main_dispatch`'s bootstrap doc) and
/// never torn down — the persistence Raven's file-scope `cg`/`cgs`/
/// `cg_entities` statics have for the life of the loaded DLL.
static WORLD: WorldCell = WorldCell::new();

/// Raven `dllEntry` (`cg_syscalls.c:15-18`, same shape as `g_syscalls.c`).
/// Stores the engine syscall trampoline into the one `OnceLock<CEngine>`.
/// `extern "C-unwind"` (SEAM-D12), matching `jampgame` and `ui`.
///
/// PANIC POLICY (2026-07-08, mirrored from `jampgame`): `dllEntry` runs BEFORE
/// the engine syscall pointer is armed, so there is no `Com_Error`/`CG_Error`
/// path to route a failure through yet. Its only sound failure mode is
/// `eprintln!` + `std::process::abort()` — a panic must never unwind raw
/// across the `extern "C-unwind"` boundary into the C engine (UB). The capture
/// hook is installed FIRST so any panic in the remaining setup is still
/// recorded with `file:line`.
///
/// DIVERGENCE from `jampgame` (same as `ui`): `mp_cgame`'s `Com_Error`/
/// `Com_Printf` (`cg_main.rs`) take `ctx: &mut CgContext` directly and route
/// through `trap::Error`/`trap::Print` at the call site — cgame has no
/// engine-wide `Com_Printf`/`Com_Error` C callback to register, so there is no
/// `set_com_print_sink`/`set_com_error_sink` counterpart here.
#[no_mangle]
pub extern "C-unwind" fn dllEntry(syscall: RawSyscall) {
    let armed = std::panic::catch_unwind(AssertUnwindSafe(|| {
        panic_guard::install_hook();
        ENGINE.set(CEngine::new(syscall)).ok();
    }));
    if armed.is_err() {
        let msg = panic_guard::take().unwrap_or_else(|| "panic in dllEntry".to_string());
        eprintln!("cgame: fatal panic during dllEntry (engine error path not yet armed): {msg}");
        std::process::abort();
    }
}

/// Raven `vmMain` (`cg_main.c:190`) — the single engine→cgame choke point.
/// `extern "C-unwind"` (SEAM-D12), and deliberately NO `catch_unwind`
/// (foreign-exception ruling, user 2026-07-12, mirrored from `jampgame`): the
/// HOST ENGINE's error exception passes back THROUGH these frames on every
/// in-trap `Com_Error`/`CG_Error` — our Rust engine's `ComError` (a foreign
/// exception to this image's runtime) or retail's MSVC C++ `throw` — and a
/// `catch_unwind` intercepting a foreign exception is an instant abort. The
/// module never throws across this boundary itself: every deliberate error is
/// the `CG_ERROR` trap (`cg_main::CG_Error` → `trap::Error`), and a genuine
/// module bug prints `file:line — message` via the `panic_guard` hook and
/// then dies fatally, exactly as a crashing C module would (LIFE-D3: real bug
/// → fatal).
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
    vm_main_dispatch(
        command, arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
    )
}

/// The decoded-dispatch body of [`vmMain`]. Bootstraps/derives the `WORLD`
/// pointer and forwards the raw `AbiWord` words into
/// `mp_cgame::cg_main::vmMain` — which splits the `CgState` into `world`/
/// `menus`/`cgDC`, builds the `CgContext` receiver and routes the command
/// (DEC-47.1). The words pass through unconverted: `mp_cgame`'s `vmMain`
/// takes `isize` slots directly because several arms carry raw addresses
/// (Raven's ILP32 `(int)&x` casts, sound at `AbiWord` width on both module
/// and dev builds); only the `AbiCommand` (`c_int`) widens.
///
/// BOOTSTRAP (STATE-D6, the `ui` spelling): Raven's `cg`/`cgs`/`cg_entities`
/// are file-scope statics alive from dlopen, and `CG_Init` re-zeroes every one
/// of them itself (`memset(&cg)/(&cgs)/(cg_entities)/(cg_weapons)/(cg_items)`,
/// `cg_main.c:1047-1541`, all ported into `CG_Init`) — so the cell is
/// populated lazily on the FIRST `vmMain` call of ANY kind (in practice
/// `CG_INIT`, which the engine always calls first, `cl_cgame.cpp`
/// `CL_InitCGame`) and never torn down; a second `CG_INIT` against the same
/// loaded image gets exactly Raven's re-init, not a rebuild.
///
/// REENTRANCY (DIVERGES from `ui`'s dispatch-spanning `&mut`): cgame traps DO
/// re-enter `vmMain` — `CG_Init`'s `CG_LoadingString` → `trap_UpdateScreen` →
/// `SCR_UpdateScreen` → `CL_CGameRendering` → `VM_Call(cgvm,
/// CG_DRAW_ACTIVE_FRAME)` (`cl_scrn.cpp:439-442`, the loading screen) — so
/// this shell keeps `jampgame`'s discipline: each entry derives its OWN
/// pointer chain from the cell, never a borrow stored across the dispatch.
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
    // SAFETY: single-threaded per Raven's contract; no reentrancy is possible
    // before the world exists (STATE-D6) — the reentrant CG_DRAW_ACTIVE_FRAME
    // chain only fires from inside a dispatched CG_INIT, by which point the
    // cell is populated. `CgState::new_boxed` builds the island directly on
    // the heap (jampgame's zeroed_boxed lesson — the pools never transit this
    // deep engine-called frame by value).
    unsafe {
        if (*WORLD.0.get()).is_none() {
            *WORLD.0.get() = Some(CgState::new_boxed());
        }
    }

    // SAFETY: single-threaded per Raven's contract; each (possibly reentrant)
    // entry derives its OWN pointer chain from the cell — the jampgame
    // STATE-D6 spelling, NOT ui's dispatch-spanning-borrow rationale, because
    // the trap_UpdateScreen chain above genuinely re-enters mid-CG_INIT.
    // `mp_cgame::vmMain` splits this borrow into the three disjoint field
    // borrows the ported fns thread.
    let state: &mut CgState = unsafe {
        let b = (*WORLD.0.get())
            .as_mut()
            .expect("the bootstrap above always populates WORLD first");
        &mut **b
    };
    let engine = ENGINE.get().expect("dllEntry set ENGINE");

    mp_cgame_vm_main(
        state,
        engine,
        command as isize,
        arg0,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6,
        arg7,
        arg8,
        arg9,
        arg10,
        arg11,
    )
}

// `GetModuleAPI` is deliberately NOT exported (SEAM-Q7 ruling, 2026-07-06,
// same as `jampgame`): OpenJK hard-fails on a present-but-NULL-returning
// symbol and falls back to the legacy `dllEntry`/`vmMain` path only when it
// is absent. Tracked in https://github.com/mheh/jka-rust/issues/1.
