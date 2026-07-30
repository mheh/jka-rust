//! The C6b trap-stream RECORDER shim (DEC-48 rulings 1+3).
//!
//! openjk.app dlopens this cdylib as the cgame module. On `dllEntry` we dlopen
//! the REAL cgame module named by `JKA_SHIM_REAL_CGAME` and hand it OUR logging
//! trampoline in place of the engine syscall; on every `vmMain` and every trap
//! we journal the bidirectional stream (length-prefixed LE records, see
//! ../README.md 'Journal format') for later headless replay. The shim only
//! observes and forwards - it never changes a word crossing the seam.
//!
//! The variadic syscall trampoline and the forward to the real engine live in
//! src/trampoline.c (stable Rust can neither define nor correctly call a
//! C-variadic fn on Apple arm64); this file does the logging and owns the vmMain
//! export.

#![allow(non_snake_case)]

use std::cell::RefCell;
use std::ffi::{c_int, c_void, CString};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

mod journal;
mod serialize;

use journal::{
    BlobKind, BlobSink, Journal, Record, REC_MALFORMED, REC_SYSCALL_ENTER, REC_SYSCALL_EXIT,
    REC_VMCALL_ENTER, REC_VMCALL_EXIT,
};
use serialize::{
    export_enter_blobs, export_exit_blobs, export_shape, trap_enter_blobs, trap_exit_blobs,
    trap_shape, SharedKind, CG_SET_SHARED_BUFFER, SHARED_BUFFER_SIZE,
};

// the C half (src/trampoline.c).
extern "C" {
    fn shim_set_engine_syscall(fn_ptr: *mut c_void);
    fn shim_get_trampoline() -> *mut c_void;
}

/// Real cgame `vmMain` - non-variadic, 13 fixed words (the oracle widened its
/// int params to intptr_t; cg_main.c:190, README 'vmMain word width').
type RealVm = unsafe extern "C-unwind" fn(
    c_int,
    isize,
    isize,
    isize,
    isize,
    isize,
    isize,
    isize,
    isize,
    isize,
    isize,
    isize,
    isize,
) -> isize;

/// Real cgame `dllEntry(syscall)`.
type RealDllEntry = unsafe extern "C-unwind" fn(*mut c_void);

static REAL_VM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
/// The engine-retained shared region (CG_SET_SHARED_BUFFER); 0 until registered.
static SHARED_BUF: AtomicUsize = AtomicUsize::new(0);
static SEQ: AtomicU64 = AtomicU64::new(0);
static JOURNAL: Mutex<Option<Journal>> = Mutex::new(None);
/// Kept alive so the real module stays mapped; never dlclose'd.
static DL_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

thread_local! {
    /// LIFO seq stack for pairing SYSCALL_ENTER with its EXIT across the two
    /// separate C calls. vmMain pairs with a local seq, so only syscalls push.
    static SEQ_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

fn next_seq() -> u64 {
    SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Runs `f` with the journal if it is open. Never held across a forwarded call,
/// so the reentrant trap->vmMain chain cannot deadlock on it.
fn with_journal<F: FnOnce(&mut Journal)>(f: F) {
    if let Ok(mut g) = JOURNAL.lock() {
        if let Some(j) = g.as_mut() {
            f(j);
        }
    }
}

/// Dumps the 2048-byte engine-retained shared region into `rec` if registered.
fn dump_shared(rec: &mut Record) {
    let p = SHARED_BUF.load(Ordering::Relaxed);
    if p != 0 {
        // SAFETY: the module registered this region via CG_SET_SHARED_BUFFER; it
        // stays live for the module's lifetime (cg_local.h:997).
        let bytes = unsafe { std::slice::from_raw_parts(p as *const u8, SHARED_BUFFER_SIZE) };
        rec.blob(0xFF, BlobKind::SharedBuffer, bytes);
    }
}

// ---- inbound: the module's traps (called from the C trampoline) -----------

/// SYSCALL_ENTER: record the trap + raw frame + in-blobs, and register the
/// shared buffer when this is CG_SET_SHARED_BUFFER. `args` is the flat 16-word
/// frame (args[0] = trap number).
#[no_mangle]
pub extern "C" fn rust_log_syscall_enter(args: *const isize) {
    let _ = std::panic::catch_unwind(|| {
        // SAFETY: the C trampoline always passes its full 16-word frame.
        let frame: [isize; 16] = unsafe { std::ptr::read(args as *const [isize; 16]) };
        let number = frame[0] as i64;

        if number == CG_SET_SHARED_BUFFER {
            SHARED_BUF.store(frame[1] as usize, Ordering::Relaxed);
        }

        let seq = next_seq();
        SEQ_STACK.with(|s| s.borrow_mut().push(seq));

        let mut rec = Record::new(REC_SYSCALL_ENTER, seq);
        rec.push_i64(number);
        let known = if let Some(shape) = trap_shape(number) {
            // trim the word block to the trap's real arity - the trampoline
            // grabs 16 stack words and everything past the arity is garbage
            // that would false-diff between the two modules at replay
            rec.push_words(&frame[..shape.args.len() + 1]);
            trap_enter_blobs(shape, &frame, &mut rec);
            if shape.dumps_shared {
                dump_shared(&mut rec);
            }
            true
        } else {
            rec.push_words(&frame);
            false
        };
        with_journal(|j| {
            j.write(&rec);
            if !known {
                // unclassifiable trap - keep forwarding, flag for the differ.
                let mut m = Record::new(REC_MALFORMED, seq);
                m.push_i64(number);
                m.push_words(&frame);
                j.write(&m);
            }
        });
    });
}

/// SYSCALL_EXIT: record the return word + out-blobs (engine has written them).
#[no_mangle]
pub extern "C" fn rust_log_syscall_exit(args: *const isize, ret: isize) {
    let _ = std::panic::catch_unwind(|| {
        // SAFETY: same full 16-word frame the enter saw.
        let frame: [isize; 16] = unsafe { std::ptr::read(args as *const [isize; 16]) };
        let number = frame[0] as i64;
        let seq = SEQ_STACK
            .with(|s| s.borrow_mut().pop())
            .unwrap_or_else(next_seq);

        let mut rec = Record::new(REC_SYSCALL_EXIT, seq);
        rec.push_i64(number);
        rec.push_i64(ret as i64);
        if let Some(shape) = trap_shape(number) {
            trap_exit_blobs(shape, &frame, &mut rec);
            if shape.dumps_shared {
                dump_shared(&mut rec);
            }
        }
        with_journal(|j| j.write(&rec));
    });
}

// ---- the exports the engine calls -----------------------------------------

/// Raven `dllEntry` (cg_syscalls.c:15-18). Store the engine syscall, dlopen the
/// real module, hand it our logging trampoline. Loud + fatal on a missing or
/// unloadable real module - the shim is useless without it.
///
/// PANIC POLICY: like crates/cgame, no error path is armed here, so a panic can
/// only `eprintln!` + abort - never unwind raw across the C boundary.
#[no_mangle]
pub extern "C-unwind" fn dllEntry(syscall: *mut c_void) {
    let armed = std::panic::catch_unwind(|| setup(syscall));
    if armed.is_err() {
        eprintln!("cgame-shim: fatal panic during dllEntry");
        std::process::abort();
    }
}

fn setup(syscall: *mut c_void) {
    let real_path = match std::env::var("JKA_SHIM_REAL_CGAME") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            fail("JKA_SHIM_REAL_CGAME unset - nowhere to forward to");
            std::process::abort();
        }
    };

    // journal path: JKA_SHIM_JOURNAL, else beside the real module (never a
    // silent /tmp default - DEC-48 recorder contract).
    let journal_path = match std::env::var("JKA_SHIM_JOURNAL") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => real_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("cgame-shim-journal.bin"),
    };
    match Journal::create(&journal_path) {
        Ok(j) => *JOURNAL.lock().unwrap() = Some(j),
        Err(e) => {
            eprintln!(
                "cgame-shim: cannot open journal {}: {e}",
                journal_path.display()
            );
            std::process::abort();
        }
    }
    with_journal(|j| {
        j.marker(
            next_seq(),
            &format!("journal opened at {}", journal_path.display()),
        )
    });

    // hand the C half the real engine syscall to forward to.
    // SAFETY: FFI to our own C trampoline store.
    unsafe { shim_set_engine_syscall(syscall) };

    // dlopen the real cgame module.
    let cpath = CString::new(real_path.as_os_str().to_string_lossy().as_bytes()).unwrap();
    // SAFETY: standard dlopen; RTLD_NOW binds every undefined symbol now so a
    // bad module fails here, not mid-frame.
    let handle = unsafe { libc::dlopen(cpath.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() {
        // SAFETY: dlerror returns a static C string valid until the next call.
        let err = unsafe { libc::dlerror() };
        let msg = if err.is_null() {
            "unknown dlerror".to_string()
        } else {
            unsafe { std::ffi::CStr::from_ptr(err) }
                .to_string_lossy()
                .into_owned()
        };
        fail(&format!("dlopen {} failed: {msg}", real_path.display()));
        std::process::abort();
    }
    DL_HANDLE.store(handle, Ordering::Relaxed);

    let real_entry = dlsym_required(handle, b"dllEntry\0");
    let real_vm = dlsym_required(handle, b"vmMain\0");
    REAL_VM.store(real_vm, Ordering::Relaxed);

    // call the real module's dllEntry with OUR trampoline.
    // SAFETY: the trampoline pointer is our own exported variadic C fn.
    let trampoline = unsafe { shim_get_trampoline() };
    let entry: RealDllEntry = unsafe { std::mem::transmute(real_entry) };
    unsafe { entry(trampoline) };

    with_journal(|j| {
        j.marker(
            next_seq(),
            &format!("real cgame loaded: {}", real_path.display()),
        )
    });
}

/// dlsym a required export or abort loudly.
fn dlsym_required(handle: *mut c_void, name: &[u8]) -> *mut c_void {
    // SAFETY: name is a NUL-terminated byte literal.
    let sym = unsafe { libc::dlsym(handle, name.as_ptr() as *const _) };
    if sym.is_null() {
        let n = String::from_utf8_lossy(&name[..name.len() - 1]);
        fail(&format!("real cgame missing export `{n}`"));
        std::process::abort();
    }
    sym
}

/// Loud stderr + a marker record (when the journal is open) for a fatal setup
/// failure.
fn fail(msg: &str) {
    eprintln!("cgame-shim: FATAL - {msg}");
    with_journal(|j| j.marker(next_seq(), &format!("FATAL: {msg}")));
}

/// Raven `vmMain` (cg_main.c:190). Bracket the real call with VMCALL_ENTER /
/// VMCALL_EXIT; the engine's error exception may unwind THROUGH the real call
/// (extern "C-unwind"), in which case the EXIT is simply absent - exactly the
/// nesting the bracketed journal encodes.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
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
    let seq = next_seq();
    let cmd = command as i64;

    // ENTER (logging panics never escape into the engine).
    let _ = std::panic::catch_unwind(|| write_vmcall_enter(seq, cmd, &words));

    // forward - NOT wrapped, so a host Com_Error can unwind straight through.
    let real: RealVm = unsafe { std::mem::transmute(REAL_VM.load(Ordering::Relaxed)) };
    let ret = unsafe {
        real(
            command, arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
        )
    };

    // EXIT.
    let _ = std::panic::catch_unwind(|| write_vmcall_exit(seq, cmd, &words, ret));

    // CG_SHUTDOWN (cgameExport_t = 1): the recording session ends here - finish
    // the gzip stream so the trailer lands (statics never run Drop at exit).
    // Anything after a vid_restart-style re-init goes unrecorded.
    if cmd == 1 {
        if let Ok(mut g) = JOURNAL.lock() {
            if let Some(j) = g.take() {
                j.finish();
            }
        }
    }
    ret
}

fn write_vmcall_enter(seq: u64, cmd: i64, words: &[isize]) {
    let mut rec = Record::new(REC_VMCALL_ENTER, seq);
    rec.push_i64(cmd);
    rec.push_words(words);
    let known = if let Some(shape) = export_shape(cmd) {
        export_enter_blobs(shape, words, &mut rec);
        if matches!(shape.shared, SharedKind::In | SharedKind::Inout) {
            dump_shared(&mut rec);
        }
        true
    } else {
        false
    };
    with_journal(|j| {
        j.write(&rec);
        if !known {
            let mut m = Record::new(REC_MALFORMED, seq);
            m.push_i64(cmd);
            m.push_words(words);
            j.write(&m);
        }
    });
}

fn write_vmcall_exit(seq: u64, cmd: i64, words: &[isize], ret: isize) {
    let mut rec = Record::new(REC_VMCALL_EXIT, seq);
    rec.push_i64(cmd);
    rec.push_i64(ret as i64);
    if let Some(shape) = export_shape(cmd) {
        export_exit_blobs(shape, words, ret, &mut rec);
        if matches!(shape.shared, SharedKind::Out | SharedKind::Inout) {
            dump_shared(&mut rec);
        }
    }
    with_journal(|j| j.write(&rec));
}
