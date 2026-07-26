//! `ui` — the MP ui module cdylib shell (SEAM-D10). Thin: hosts the
//! `ENGINE: OnceLock<CEngine>` static (SEAM-D1), the `WORLD: WorldCell`
//! static (STATE-D6), the live entrypoint exports, and the `vmMain` export
//! forwarding into `mp_ui::ui_main::vmMain` — which owns the `MpUiExport`
//! match, splits the `UiState` into its three disjoint borrows and builds the
//! `UiContext` receiver (DEC-38 ruling 1; see `vm_main_dispatch`'s doc). The logic crate `mp_ui` has no entrypoint/`OnceLock`/`WorldCell`
//! code of its own. Structure, naming and safety comments mirror the
//! `jampgame` shell (`crates/jampgame/src/lib.rs`) exactly except where ui's
//! own contract genuinely differs — each such spot is called out at the
//! divergence.

use std::panic::AssertUnwindSafe;
use std::sync::OnceLock;

mod panic_guard;
mod world_cell;

use abi_transport::entrypoints::{AbiCommand, AbiWord, RawSyscall};
use abi_transport::generic::engine::CEngine;
use abi_transport::generic::word_to_c_int;
use mp_ui::ui_main::vmMain as mp_ui_vm_main;
use mp_ui::world::ui_state::UiState;

use crate::world_cell::WorldCell;

/// The single outbound-syscall backend seam global (SEAM-D1, porting-rules §B6
/// exception — `vmMain` takes no context argument). Set once at `dllEntry`.
static ENGINE: OnceLock<CEngine> = OnceLock::new();

/// The module island's one owned `UiState` across `vmMain` calls (STATE-D6,
/// the second sanctioned static exemption). Bootstrapped lazily on the first
/// `vmMain` call of any kind (see `vm_main_dispatch`'s bootstrap doc — this is
/// the one genuine divergence from `jampgame`'s `WORLD`, which is built on the
/// `GAME_INIT` command specifically).
static WORLD: WorldCell = WorldCell::new();

/// Raven `dllEntry` (`ui_atoms.c`-style stub, same shape as
/// `g_syscalls.c:14-16`). Stores the engine syscall trampoline into the one
/// `OnceLock<CEngine>`. `extern "C-unwind"` (SEAM-D12), matching `jampgame`.
///
/// PANIC POLICY (2026-07-08, mirrored from `jampgame`): `dllEntry` runs BEFORE
/// the engine syscall pointer is armed, so there is no `Com_Error`/`UI_Error`
/// path to route a failure through yet. Its only sound failure mode is
/// `eprintln!` + `std::process::abort()` — a panic must never unwind raw
/// across the `extern "C-unwind"` boundary into the C engine (UB). The capture
/// hook is installed FIRST so any panic in the remaining setup is still
/// recorded with `file:line`.
///
/// DIVERGENCE from `jampgame`: `mp_ui`'s `Com_Error`/`Com_Printf`
/// (`ui_atoms.rs`) take `ctx: &mut UiContext` directly and route through
/// `trap::Error`/`trap::Print` at the call site — unlike `mp_game`'s
/// `com_boundary` global-sink pattern, ui has no engine-wide `Com_Printf`/
/// `Com_Error` C callback to register, so there is no `set_com_print_sink`/
/// `set_com_error_sink` counterpart here.
#[no_mangle]
pub extern "C-unwind" fn dllEntry(syscall: RawSyscall) {
    let armed = std::panic::catch_unwind(AssertUnwindSafe(|| {
        panic_guard::install_hook();
        ENGINE.set(CEngine::new(syscall)).ok();
    }));
    if armed.is_err() {
        let msg = panic_guard::take().unwrap_or_else(|| "panic in dllEntry".to_string());
        eprintln!("ui: fatal panic during dllEntry (engine error path not yet armed): {msg}");
        std::process::abort();
    }
}

/// Raven `vmMain` (`ui_main.c:579`) — the single engine→ui choke point.
/// `extern "C-unwind"` (SEAM-D12), and deliberately NO `catch_unwind`
/// (foreign-exception ruling, user 2026-07-12, mirrored from `jampgame`): the
/// HOST ENGINE's error exception passes back THROUGH these frames on every
/// in-trap `Com_Error`/`UI_Error` — our Rust engine's `ComError` (a foreign
/// exception to this image's runtime) or retail's MSVC C++ `throw` — and a
/// `catch_unwind` intercepting a foreign exception is an instant abort. The
/// module never throws across this boundary itself: every deliberate error is
/// the `UI_ERROR` trap (`ui_atoms::Com_Error` → `trap::Error`), and a genuine
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
/// pointer and forwards the decoded `AbiWord` words into
/// `mp_ui::ui_main::vmMain` — which splits the `UiState` into `world`/`menus`/
/// `uiDC`, builds the `UiContext` receiver (itself the module's
/// `DisplayContext`, DEC-38 ruling 1) and runs the exhaustive `MpUiExport`
/// match (SEAM-D3/D8), mirroring `jampgame::vm_main_dispatch`'s shape.
///
/// BOOTSTRAP (STATE-D6, DIVERGES from `jampgame`'s `GAME_INIT`-gated
/// rebuild): Raven's `uiInfo_t uiInfo` (`ui_local.h:729-843`) is a single
/// file-scope static that `_UI_Init` only mutates in place
/// (`oracle/codemp/ui/ui_main.c:10661` onward has no `memset`/reinit) — it is
/// never freed or rebuilt across a `UI_SHUTDOWN`/`UI_INIT` pair, unlike
/// `g_entities`/`level`, which `G_InitGame` re-creates every level restart.
/// There is no ui command analogous to `GAME_INIT` that must run first and
/// "create" the world, so `WORLD` is instead populated lazily on the FIRST
/// `vmMain` call of ANY kind (in practice `UI_GETAPIVERSION`, which the
/// engine always calls before `UI_INIT`) and is never torn down on
/// `UI_SHUTDOWN` — the same persistence Raven's static gave `uiInfo`.
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
    // before the world exists (STATE-D6). See the fn doc for why the
    // bootstrap condition is "cell empty", not "command == some INIT arm".
    unsafe {
        if (*WORLD.0.get()).is_none() {
            // UiState is ~34 KB (Vec/String throughout), so the Box::new
            // temporary transits the engine's stack safely — unlike GameWorld's
            // ~1.4 MB, which needed jampgame's zeroed_boxed path.
            *WORLD.0.get() = Some(Box::new(UiState::default()));
        }
    }

    // SAFETY: single-threaded per Raven's contract, and — unlike jampgame,
    // whose traps can re-enter vmMain and therefore thread a raw `*mut` per
    // STATE-D6 — a single dispatch-spanning `&mut UiState` is sound HERE
    // because no ui trap re-enters vmMain: the one engine chain that would
    // (`trap_UpdateScreen` → `SCR_UpdateScreen` → `VM_Call(uivm, UI_REFRESH)`)
    // is dead — `UI_UpdateScreen` has zero call sites in the ui tree (oracle
    // and port alike), and `executeText` script paths use `EXEC_APPEND`.
    // `mp_ui::vmMain` splits this one borrow into the three disjoint field
    // borrows the ported fns thread. If a re-entering ui trap is ever wired,
    // this must convert to jampgame's raw-pointer threading.
    let state: &mut UiState = unsafe {
        let b = (*WORLD.0.get())
            .as_mut()
            .expect("the bootstrap above always populates WORLD first");
        &mut **b
    };
    let engine = ENGINE.get().expect("dllEntry set ENGINE");

    let result = mp_ui_vm_main(
        state,
        engine,
        command,
        word_to_c_int(arg0),
        word_to_c_int(arg1),
        word_to_c_int(arg2),
        word_to_c_int(arg3),
        word_to_c_int(arg4),
        word_to_c_int(arg5),
        word_to_c_int(arg6),
        word_to_c_int(arg7),
        word_to_c_int(arg8),
        word_to_c_int(arg9),
        word_to_c_int(arg10),
        word_to_c_int(arg11),
    );

    result as AbiWord
}

// `GetModuleAPI` is deliberately NOT exported (SEAM-Q7 ruling, 2026-07-06,
// same as `jampgame`): OpenJK hard-fails on a present-but-NULL-returning
// symbol and falls back to the legacy `dllEntry`/`vmMain` path only when it
// is absent. Tracked in https://github.com/mheh/jka-rust/issues/1.
