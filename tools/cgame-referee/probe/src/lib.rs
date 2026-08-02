//! The standalone demo-referee probe module (ticket gh#30).
//!
//! The oracle engine dlopens this cdylib as its cgame module, so both engines
//! run the SAME probe body (`probe.rs`) and their journals are comparable record
//! for record. Stage the built `libcgamearm64.dylib` as `cgamearm64.dylib` under
//! the engine's `base` directory. `tools/cgame-referee/record-golden.sh` does
//! the staging and the run.
//!
//! # Environment
//! - `JKA_PROBE_JOURNAL` - where to write the journal. Required. There is no
//!   silent default, the recorder-shim rule.
//! - `JKA_PROBE_MANIFESTS` - the directory holding `trap-shapes.json` and
//!   `export-shapes.json`. Required.
//! - `JKA_PROBE_BRACKET_CAP` - journaled snapshots to keep. Defaults to 400,
//!   the DEC-62.2 golden bound.
//! - `JKA_PROBE_QUIT_AT_CAP` - set to `0` to keep the engine running after the
//!   cap. The default sends `quit` so a recording run needs no operator.
//!
//! A missing variable is a loud stderr message plus an abort, because a silent
//! recording that writes nowhere would look like a passing run.

#![allow(non_snake_case)]

use std::ffi::{c_int, c_void};
use std::path::PathBuf;
use std::sync::Mutex;

#[path = "../../shapes.rs"]
mod shapes;

#[path = "../../shim/src/journal.rs"]
mod journal;

mod probe;

use probe::{Probe, DEFAULT_BRACKET_CAP};

extern "C" {
    fn probe_set_engine_syscall(fn_ptr: *mut c_void);
    fn probe_engine_call(args: *const isize) -> isize;
}

static PROBE: Mutex<Option<Probe>> = Mutex::new(None);

/// True when the cap should send `quit` to the engine.
static QUIT_AT_CAP: Mutex<bool> = Mutex::new(true);

/// Forwards one trap frame into the engine through the C variadic call.
fn forward(args: &mut [isize; 16]) -> isize {
    // SAFETY: the C half holds the syscall pointer the engine gave `dllEntry`,
    // and the frame is always the full 16 words that call reads.
    unsafe { probe_engine_call(args.as_ptr()) }
}

/// Reads a required environment variable or aborts with a named reason.
fn required_env(name: &str) -> String {
    match std::env::var(name) {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("cgame-probe: {name} is not set - refusing to record nothing");
            std::process::abort();
        }
    }
}

/// `dllEntry` - the engine hands the module its syscall pointer.
/// Source: `oracle/codemp/cgame/cg_syscalls.c:14-17`
#[no_mangle]
pub extern "C-unwind" fn dllEntry(syscall: *mut c_void) {
    // SAFETY: the pointer the engine passed is its own variadic syscall.
    unsafe { probe_set_engine_syscall(syscall) };

    let journal_path = PathBuf::from(required_env("JKA_PROBE_JOURNAL"));
    let manifest_dir = PathBuf::from(required_env("JKA_PROBE_MANIFESTS"));
    let cap = std::env::var("JKA_PROBE_BRACKET_CAP")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_BRACKET_CAP);
    *QUIT_AT_CAP.lock().unwrap() = std::env::var("JKA_PROBE_QUIT_AT_CAP").as_deref() != Ok("0");

    match Probe::new(&journal_path, &manifest_dir, cap, forward as probe::TrapFn) {
        Ok(p) => *PROBE.lock().unwrap() = Some(p),
        Err(e) => {
            eprintln!("cgame-probe: {e}");
            std::process::abort();
        }
    }
    eprintln!(
        "cgame-probe: recording {} brackets to {}",
        cap,
        journal_path.display()
    );
}

/// `vmMain` - the engine-to-module entry. The oracle widened its int params to
/// `intptr_t`, so the shape matches the recorder shim's `RealVm`.
/// Source: `oracle/codemp/cgame/cg_main.c:190`
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub extern "C-unwind" fn vmMain(
    command: c_int,
    arg0: isize,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
    arg5: isize,
    arg6: isize,
    arg7: isize,
    arg8: isize,
    arg9: isize,
    arg10: isize,
    arg11: isize,
) -> isize {
    let words = [
        arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
    ];
    let mut guard = PROBE.lock().unwrap();
    let Some(p) = guard.as_mut() else {
        return 0;
    };
    let was_done = p.done();
    let ret = p.vm_main(command as i64, &words);
    let capped = !was_done && p.done() && command as i64 == probe::CG_DRAW_ACTIVE_FRAME;
    if capped && *QUIT_AT_CAP.lock().unwrap() {
        // The cap closed the journal, so the recording is complete. Ending the
        // engine here keeps a golden run free of operator input.
        eprintln!("cgame-probe: bracket cap reached, quitting the engine");
        p.send_console_command("quit\n");
    }
    ret
}
