//! ABI smoke test — drive the BUILT `cgame` cdylib through the exact
//! engine/module contract (`dllEntry(syscall)` handshake, then
//! `vmMain(command, arg0..arg11)`) against a mock engine, asserting the module
//! loads, completes the handshake, bootstraps its `WORLD` island and answers
//! an unknown command with Raven's `CG_Error` + `-1` fall-through.
//!
//! Modeled on `crates/ui/tests/abi_smoke.rs` — same real transport, no shims
//! of our own (libloading loader, SEAM-D11 C-variadic trampoline,
//! `arm_game_slot`).
//!
//! Scope divergence from the ui smoke: cgame has NO `CG_GETAPIVERSION` export
//! — Raven's client calls `CG_INIT` first (`cl_cgame.cpp` `CL_InitCGame`),
//! and driving the full init needs a mock deep enough for cvar registration,
//! configstrings and media (the C6b referee rig, its own stage). What IS
//! drivable today is the `vmMain` default arm (`cg_main.c:355-357`): an
//! unknown command word routes `CG_Error("vmMain: unknown command %i")`
//! through the real trampoline as a `CG_ERROR` syscall and then returns `-1`
//! — proving dlopen, the `dllEntry` handshake, the multi-MB heap `CgState`
//! bootstrap, the outbound syscall path, and the dispatch fall-through in one
//! drive. (The mock returns from `CG_ERROR` where Raven's engine longjmps;
//! `trap::Error` tolerates that and the arm then returns `-1`.)
//!
//! Single-shot: the module's `ENGINE`/`WORLD` statics and the engine slot are
//! process singletons, so the whole drive runs in ONE `#[test]` fn.

#![allow(non_snake_case)]

use std::ffi::{c_void, CStr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use mp_abi::cgame::imports::MpCgameImport;
use mp_engine_qcommon::vm::{arm_game_slot, game_syscall_trampoline};
use native_platform::entrypoints::{AbiCommand, AbiWord, RawSyscall, RawVmMain};
use native_platform::module_loader::{sys_load_dll, ModuleNaming, ModuleSearchPolicy, SearchStep};

/// A command word no `MpCgameExport` variant carries — routes to `vmMain`'s
/// default arm.
const UNKNOWN_COMMAND: AbiCommand = 0x7fff;

/// Set by the mock when the expected `CG_ERROR` syscall arrives.
static SAW_CG_ERROR: AtomicBool = AtomicBool::new(false);

/// The mock engine's `systemCall` — the fixed-arity target the SEAM-D11
/// trampoline dispatches into. The unknown-command drive makes exactly one
/// syscall (`CG_ERROR` with Raven's message); anything else is a module bug
/// this test should surface loudly.
extern "C-unwind" fn mock_syscall(_ctx: *mut c_void, args: *const isize) -> isize {
    // SAFETY: the trampoline always passes its full 16-word frame; word 0 is
    // the syscall number (`vm.cpp:366`), word 1 the message pointer.
    let (number, msg_word) = unsafe { (*args, *args.add(1)) };
    if number == MpCgameImport::CG_ERROR as isize {
        // SAFETY: `trap::Error` passes a live NUL-terminated C string for the
        // duration of this synchronous call.
        let msg = unsafe { CStr::from_ptr(msg_word as *const i8) }.to_string_lossy();
        eprintln!("[cgame-smoke] CG_ERROR arrived: {msg}");
        assert!(
            msg.contains("unknown command"),
            "CG_ERROR must carry Raven's default-arm message, got: {msg}"
        );
        SAW_CG_ERROR.store(true, Ordering::Relaxed);
        return 0; // Raven's engine longjmps here; returning exercises the fall-through
    }
    panic!("cgame module made an unexpected syscall ({number}) during the unknown-command drive");
}

/// Platform cdylib filename for a `[lib] name = "<base>"` crate:
/// `"cgame"` → `libcgame.dylib` / `libcgame.so` / `cgame.dll`.
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
/// before the first `.`, so `libcgame.dylib` → `libcgame` (the `jampgame`
/// smoke test's spelling).
fn split_dylib_stem(file: &str) -> &str {
    let dot = file.find('.').expect("dylib name has an extension");
    &file[..dot]
}

/// Locate the cargo-built cdylib next to the test binary. Integration tests run
/// from `target/<profile>/deps/`; cargo places the cdylib in both `deps/` and
/// its parent `target/<profile>/`. `cargo build --workspace` (CI) or `cargo test
/// -p cgame` (which builds the package's cdylib target) produces it beforehand.
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
         Run `cargo build --workspace` (or `cargo test -p cgame`, which builds \
         the cdylib target) first — the test never spawns cargo.",
        deps.display()
    );
}

fn run_drive(dylib: PathBuf) {
    let dir = dylib.parent().unwrap().to_path_buf();
    let file = dylib
        .file_name()
        .expect("dylib filename")
        .to_str()
        .expect("utf8 dylib filename")
        .to_string();
    eprintln!("[cgame-smoke] loading {}", dylib.display());

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

    let result = vm_main(
        UNKNOWN_COMMAND,
        0 as AbiWord,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    );
    eprintln!("[cgame-smoke] vmMain(unknown) returned {result}");
    assert_eq!(
        result, -1,
        "an unknown command must reproduce Raven's fall-through return -1 (cg_main.c:355-357)"
    );
    assert!(
        SAW_CG_ERROR.load(Ordering::Relaxed),
        "the default arm must have routed CG_Error through the CG_ERROR syscall"
    );
}

/// Load the built cdylib next to the test binary and drive the handshake plus
/// the unknown-command arm on a generous-stack thread — the first `vmMain` of
/// any kind bootstraps the `WORLD` cell (STATE-D6), and the real engine drives
/// `vmMain` on its large main-thread stack.
#[test]
fn abi_smoke_handshake_and_unknown_command() {
    let dylib = locate_cargo_cdylib(&dylib_filename("cgame"));
    let handle = std::thread::Builder::new()
        .name("cgame-abi-smoke-engine".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || run_drive(dylib))
        .expect("spawn engine thread");
    handle.join().expect("cgame drive thread panicked");
}
