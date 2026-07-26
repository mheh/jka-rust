//! ABI smoke test — drive the BUILT `ui` cdylib through the exact
//! engine/module contract (`dllEntry(syscall)` handshake, then
//! `vmMain(command, arg0..arg11)`) against a mock engine, asserting the module
//! loads, completes the handshake and answers `UI_GETAPIVERSION` with Raven's
//! `UI_API_VERSION`.
//!
//! Modeled on `crates/jampgame/tests/abi_smoke.rs` and its `tests/common/mod.rs`
//! machinery — same real transport, no shims of our own:
//!   * the module is loaded through `native_platform::module_loader::sys_load_dll`
//!     — the ported libloading loader; it resolves and invokes `dllEntry`,
//!     handing the module the engine syscall trampoline;
//!   * the syscall trampoline handed over is `mp_engine_qcommon`'s real
//!     C-variadic `game_syscall_trampoline` (SEAM-D11) — one monomorphic
//!     trampoline serves whichever slot is armed, and the ui module is the only
//!     module this test process loads;
//!   * `arm_game_slot` injects `(ctx, mock_syscall)` into that slot before the
//!     first `vmMain`.
//!
//! Scope is deliberately the handshake plus the one command Raven's client
//! always calls first (`CL_InitUI` → `UI_GETAPIVERSION`, `cl_ui.cpp`): the ui
//! module's live command arms (`UI_INIT` onward) drive menu loading through
//! `DisplayContext`, which has no built implementor yet (`display_context_stub`).
//!
//! Single-shot: the module's `ENGINE`/`WORLD` statics and the engine slot are
//! process singletons, so the whole drive runs in ONE `#[test]` fn.

#![allow(non_snake_case)]

use std::ffi::c_void;
use std::path::PathBuf;

use mp_abi::ui::exports::MpUiExport;
use mp_abi::ui::public::UI_API_VERSION;
use mp_engine_qcommon::vm::{arm_game_slot, game_syscall_trampoline};
use native_platform::entrypoints::{AbiCommand, AbiWord, RawSyscall, RawVmMain};
use native_platform::module_loader::{sys_load_dll, ModuleNaming, ModuleSearchPolicy, SearchStep};

/// The mock engine's `systemCall` — the fixed-arity target the SEAM-D11
/// trampoline dispatches into. `UI_GETAPIVERSION` makes no syscall at all, so
/// any arrival here is a module bug this test should surface loudly rather than
/// silently answer.
extern "C-unwind" fn mock_syscall(_ctx: *mut c_void, args: *const isize) -> isize {
    // SAFETY: the trampoline always passes its full 16-word frame; word 0 is the
    // syscall number (`vm.cpp:366`).
    let number = unsafe { *args };
    panic!("ui module made an unexpected syscall ({number}) during UI_GETAPIVERSION");
}

/// Platform cdylib filename for a `[lib] name = "<base>"` crate:
/// `"ui"` → `libui.dylib` / `libui.so` / `ui.dll`.
fn dylib_filename(base: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("lib{base}.dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{base}.so")
    }
    #[cfg(windows)]
    {
        format!("{base}.dll")
    }
}

/// The `ModuleNaming.suffix` the loader appends — a `&'static str`, so it comes
/// from the platform-fixed literal rather than a slice of the located filename.
fn platform_dylib_suffix() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        ".dylib"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        ".so"
    }
    #[cfg(windows)]
    {
        ".dll"
    }
}

/// The name the loader re-decorates with `ModuleNaming.suffix`: everything
/// before the first `.`, so `libui.dylib` → `libui` (the `jampgame` smoke
/// test's spelling).
fn split_dylib_stem(file: &str) -> &str {
    let dot = file.find('.').expect("dylib name has an extension");
    &file[..dot]
}

/// Locate the cargo-built cdylib next to the test binary. Integration tests run
/// from `target/<profile>/deps/`; cargo places the cdylib in both `deps/` and
/// its parent `target/<profile>/`. `cargo build --workspace` (CI) or `cargo test
/// -p ui` (which builds the package's cdylib target) produces it beforehand.
fn locate_cargo_cdylib(filename: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("current_exe");
    let deps = exe.parent().expect("deps dir");
    for dir in [deps, deps.parent().expect("target/<profile> dir")] {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return candidate;
        }
    }
    panic!(
        "built cdylib `{filename}` not found next to test binary ({}). \
         Run `cargo build --workspace` (or `cargo test -p ui`, which builds the \
         cdylib target) first — the test never spawns cargo.",
        deps.display()
    );
}

/// Call the module's `vmMain` with Raven's 13-word frame, padding the unused
/// argument words with 0 exactly as the engine does.
fn call_vm(vm_main: RawVmMain, command: MpUiExport, args: &[AbiWord]) -> AbiWord {
    let mut a = [0 as AbiWord; 12];
    a[..args.len()].copy_from_slice(args);
    vm_main(
        command as AbiCommand,
        a[0],
        a[1],
        a[2],
        a[3],
        a[4],
        a[5],
        a[6],
        a[7],
        a[8],
        a[9],
        a[10],
        a[11],
    )
}

fn run_drive(dylib: PathBuf) {
    let dir = dylib.parent().unwrap().to_path_buf();
    let file = dylib
        .file_name()
        .expect("dylib filename")
        .to_str()
        .expect("utf8 dylib filename")
        .to_string();
    eprintln!("[ui-smoke] loading {}", dylib.display());

    let name: &str = split_dylib_stem(&file);
    let policy = ModuleSearchPolicy {
        naming: ModuleNaming {
            suffix: Some(platform_dylib_suffix()),
        },
        direct_first: false,
        steps: vec![SearchStep::FsPath {
            base: dir,
            gamedir: String::new(),
        }],
    };

    // Arm the engine slot BEFORE any module syscall can fire. `dllEntry` (called
    // by the loader) only stores the pointer.
    arm_game_slot(std::ptr::null_mut(), mock_syscall);

    let syscall: RawSyscall = game_syscall_trampoline as *const c_void;
    let module = sys_load_dll(&policy, name, syscall)
        .expect("sys_load_dll resolved dllEntry+vmMain and completed the handshake");
    let vm_main: RawVmMain = module.entry();

    let version = call_vm(vm_main, MpUiExport::UI_GETAPIVERSION, &[]);
    eprintln!("[ui-smoke] UI_GETAPIVERSION returned {version}");
    assert_eq!(
        version, UI_API_VERSION as AbiWord,
        "ui module must answer Raven's UI_API_VERSION (ui_public.h)"
    );
}

/// Load the built cdylib next to the test binary and drive the handshake plus
/// `UI_GETAPIVERSION` on a generous-stack thread — the first `vmMain` of any
/// kind bootstraps the `WORLD` cell (STATE-D6), and the real engine drives
/// `vmMain` on its large main-thread stack.
#[test]
fn abi_smoke_getapiversion() {
    let dylib = locate_cargo_cdylib(&dylib_filename("ui"));
    let handle = std::thread::Builder::new()
        .name("ui-abi-smoke-engine".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || run_drive(dylib))
        .expect("spawn engine thread");
    handle.join().expect("ui drive thread panicked");
}
