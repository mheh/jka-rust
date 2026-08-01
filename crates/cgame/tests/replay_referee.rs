//! The C6b REPLAY referee (DEC-48 rulings 1,2,5 + the .4 amendment).
//!
//! Two `--ignored` tests drive a cgame module dylib headlessly from a recorded
//! trace and byte-diff its outgoing trap stream against the recording:
//!
//! - `replay_oracle_self_check` - the oracle dylib replayed against its OWN
//!   recording must be byte-identical (zero findings, reaches the end).
//! - `replay_rust_cgame` - the Rust cgame dylib replayed against the same
//!   recording must also be byte-identical (zero findings, reaches the end).
//!   The report-only phase closed at commit 153ade70 (full swoop1 trace clean).
//!
//! Traces stay OUT of git (DEC-48.4): the trace path comes from `JKA_TRACE`
//! (default `$HOME/Developer/jka/trace-swoop1.bin`); both tests SKIP with a clear
//! message when no trace file is present.
//!
//! Run serially (the game slot + module statics are process singletons), and
//! build the cdylib first - `cargo test` does NOT refresh the dylib this test
//! dlopens:
//!   cargo build -p cgame --release
//!   cargo test -p cgame --release -- --ignored --test-threads=1
//!
//! Modeled on tests/abi_smoke.rs for the dylib loader + trampoline handshake.

#![allow(non_snake_case)]

use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::Mutex;

use mp_engine_qcommon::vm::{arm_game_slot, game_syscall_trampoline};
use native_platform::entrypoints::RawSyscall;
use native_platform::module_loader::{sys_load_dll, ModuleNaming, ModuleSearchPolicy, SearchStep};

mod replay_support;
use replay_support::shapes::Manifests;
use replay_support::{referee_dir, replay_syscall, Reader, ReplayState, RunOutcome};

/// Both replay tests arm the one process-global game slot, so parallel runs corrupt each other.
/// This lock serializes them regardless of the cargo test-thread count.
/// A poisoned lock stays usable: the other test still runs after one panics.
static REPLAY_LOCK: Mutex<()> = Mutex::new(());

/// Trace path: `JKA_TRACE`, else the local-disk default (DEC-48.4).
fn trace_path() -> PathBuf {
    if let Ok(p) = std::env::var("JKA_TRACE") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("Developer/jka/trace-swoop1.bin")
}

/// The oracle cgame dylib (tools/cgame-oracle/build/liboraclecgame.dylib).
fn oracle_dylib() -> PathBuf {
    referee_dir()
        .parent()
        .unwrap()
        .join("cgame-oracle/build/liboraclecgame.dylib")
}

/// Locate the cargo-built `libcgame.dylib` next to the test binary (the
/// abi_smoke recipe).
fn rust_cgame_dylib() -> PathBuf {
    let filename = dylib_filename("cgame");
    let exe = std::env::current_exe().expect("current_exe");
    let deps = exe.parent().expect("deps dir");
    for dir in [deps, deps.parent().expect("target/<profile> dir")] {
        let candidate = dir.join(&filename);
        if candidate.exists() {
            return candidate;
        }
    }
    panic!(
        "built cdylib `{filename}` not found next to the test binary ({}). \
         Run `cargo build -p cgame --release` first - `cargo test` does not \
         build or refresh this dylib.",
        deps.display()
    );
}

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

fn platform_suffix() -> &'static str {
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

fn split_stem(file: &str) -> &str {
    let dot = file.find('.').expect("dylib name has an extension");
    &file[..dot]
}

/// Load `dylib`, build the replay state around its `vmMain`, arm the game slot,
/// and drive the whole recording. Runs on a generous-stack thread (the first
/// vmMain bootstraps the module's WORLD island, same as abi_smoke).
fn drive(dylib: PathBuf, trace: PathBuf) -> RunOutcome {
    let handle = std::thread::Builder::new()
        .name("cgame-replay-engine".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let manifests = Manifests::load(&referee_dir()).expect("load shape manifests");
            let reader = Reader::open(&trace).expect("open trace");

            let dir = dylib.parent().unwrap().to_path_buf();
            let file = dylib.file_name().unwrap().to_str().unwrap().to_string();
            let name = split_stem(&file);
            let policy = ModuleSearchPolicy {
                naming: ModuleNaming {
                    suffix: Some(platform_suffix()),
                },
                direct_first: false,
                steps: vec![SearchStep::FsPath {
                    base: dir,
                    gamedir: String::new(),
                }],
            };

            let syscall: RawSyscall = game_syscall_trampoline as *const c_void;
            let module = sys_load_dll(&policy, name, syscall)
                .expect("sys_load_dll resolved dllEntry+vmMain");
            let module_vm = module.entry();

            let state = ReplayState::new(manifests, reader, module_vm);
            let ctx = &*state as *const ReplayState as *mut c_void;
            arm_game_slot(ctx, replay_syscall);

            let outcome = state.run();
            // keep the module + state mapped until the drive is fully done.
            drop(module);
            outcome
        })
        .expect("spawn engine thread");
    handle.join().expect("replay drive thread panicked")
}

fn print_summary(label: &str, o: &RunOutcome) {
    eprintln!(
        "[{label}] records={} vmcalls={} syscalls={}",
        o.records, o.vmcalls, o.syscalls
    );
    eprintln!("[{label}] findings={}", o.finding_total);
    for (class, count, first) in &o.finding_census {
        eprintln!("[{label}]   census {count:>8}  {class}");
        eprintln!("[{label}]            first: {first}");
    }
    if let Some(d) = &o.desync {
        eprintln!("[{label}] HARD DESYNC: {d}");
    }
    if let Some(first) = o.findings.first() {
        eprintln!("[{label}] first finding: {first}");
    }
    for (i, fnd) in o.findings.iter().enumerate() {
        eprintln!("[{label}]   #{i}: {fnd}");
    }
}

/// True + a skip message when no trace file is present (DEC-48.4).
fn skip_if_no_trace(trace: &PathBuf) -> bool {
    if trace.exists() {
        return false;
    }
    eprintln!(
        "SKIP: no trace at {} - record one (or set JKA_TRACE); traces stay out of git (DEC-48.4).",
        trace.display()
    );
    true
}

#[test]
#[ignore = "needs a recorded trace on local disk (DEC-48.4); run with --ignored"]
fn replay_oracle_self_check() {
    let _serial = REPLAY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let trace = trace_path();
    if skip_if_no_trace(&trace) {
        return;
    }
    let dylib = oracle_dylib();
    assert!(
        dylib.exists(),
        "oracle dylib missing at {} - run tools/cgame-oracle/build.sh first",
        dylib.display()
    );

    let o = drive(dylib, trace);
    print_summary("oracle-self-check", &o);

    // The self-check bar: the oracle dylib replayed against its own recording is
    // byte-identical. A hard desync or any finding is a harness/recording bug.
    assert!(
        o.desync.is_none(),
        "oracle self-check hit a hard desync: {:?}",
        o.desync
    );
    assert_eq!(
        o.finding_total,
        0,
        "oracle self-check must be byte-identical; first finding: {}",
        o.findings
            .first()
            .map(|f| f.to_string())
            .unwrap_or_default()
    );
    assert!(o.records > 0, "self-check consumed no records");
}

#[test]
#[ignore = "needs a recorded trace on local disk (DEC-48.4); run with --ignored"]
fn replay_rust_cgame() {
    let _serial = REPLAY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let trace = trace_path();
    if skip_if_no_trace(&trace) {
        return;
    }
    let dylib = rust_cgame_dylib();
    let o = drive(dylib, trace);
    print_summary("rust-cgame", &o);

    // The bar matches the oracle self-check: the Rust dylib must replay the
    // whole recording byte-identical. A stale dylib fakes a regression - run
    // `cargo build -p cgame --release` before this test.
    assert!(
        o.desync.is_none(),
        "rust replay hit a hard desync: {:?}",
        o.desync
    );
    assert_eq!(
        o.finding_total,
        0,
        "rust replay must be byte-identical; first finding: {}",
        o.findings
            .first()
            .map(|f| f.to_string())
            .unwrap_or_default()
    );
    assert!(
        o.records > 0,
        "rust replay consumed no records - the recording never opened"
    );
}

/// Formats one blob preview: length, then hex of the first bytes, then floats
/// when the length is word-aligned and small.
fn blob_preview(kind: u8, bytes: &[u8]) -> String {
    let hex: String = bytes
        .iter()
        .take(16)
        .map(|b| format!("{b:02x} "))
        .collect();
    let mut out = format!("kind={kind} len={} [{}]", bytes.len(), hex.trim_end());
    if bytes.len() % 4 == 0 && bytes.len() <= 48 {
        let floats: Vec<String> = bytes
            .chunks_exact(4)
            .map(|c| format!("{:.9}", f32::from_le_bytes(c.try_into().unwrap())))
            .collect();
        out.push_str(&format!(" f32[{}]", floats.join(", ")));
    }
    out
}

/// Trace inspector for divergence hunts and census spikes.
/// `JKA_DUMP=<lo>:<hi>` prints every journal record with seq in `[lo, hi]`,
/// plus the last vmMain enter before the window for frame context.
#[test]
#[ignore = "inspection tool - set JKA_TRACE and JKA_DUMP=<lo>:<hi>, run with --ignored"]
fn dump_trace_window() {
    use replay_support::{REC_SYSCALL_ENTER, REC_SYSCALL_EXIT, REC_VMCALL_ENTER, REC_VMCALL_EXIT};

    let Ok(spec) = std::env::var("JKA_DUMP") else {
        eprintln!("SKIP: set JKA_DUMP=<lo>:<hi>");
        return;
    };
    let (lo, hi) = spec
        .split_once(':')
        .and_then(|(a, b)| Some((a.parse::<u64>().ok()?, b.parse::<u64>().ok()?)))
        .expect("JKA_DUMP must be <lo>:<hi>");

    let trace = trace_path();
    if skip_if_no_trace(&trace) {
        return;
    }
    let manifests = Manifests::load(&referee_dir()).expect("load shape manifests");
    let mut reader = Reader::open(&trace).expect("open trace");

    let mut last_vm: Option<String> = None;
    let mut context_shown = false;
    while let Some(r) = reader.take() {
        if r.seq > hi {
            break;
        }
        let line = |name: &str, tag: &str| {
            let words: Vec<String> = r.words.iter().map(|w| w.to_string()).collect();
            let mut s = format!("seq {:>9} {tag:<8} {name} words[{}]", r.seq, words.join(", "));
            for b in &r.blobs {
                s.push_str(&format!("\n              arg{} {}", b.arg_index, blob_preview(b.kind, &b.bytes)));
            }
            s
        };
        match r.rec_type {
            REC_VMCALL_ENTER => {
                let name = manifests
                    .export(r.cmd)
                    .map(|e| e.name.clone())
                    .unwrap_or_else(|| format!("vm#{}", r.cmd));
                let s = line(&name, "VM>");
                if r.seq < lo {
                    last_vm = Some(s);
                } else {
                    eprintln!("{s}");
                }
            }
            REC_VMCALL_EXIT if r.seq >= lo => {
                eprintln!("seq {:>9} VM<      ret={}", r.seq, r.ret);
            }
            REC_SYSCALL_ENTER | REC_SYSCALL_EXIT if r.seq >= lo => {
                if !context_shown {
                    context_shown = true;
                    if let Some(vm) = last_vm.take() {
                        eprintln!("(last vmMain before window)\n{vm}");
                    }
                }
                let name = manifests
                    .trap(r.cmd)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| format!("trap#{}", r.cmd));
                let tag = if r.rec_type == REC_SYSCALL_ENTER { "SYS>" } else { "SYS<" };
                let mut s = line(&name, tag);
                if r.rec_type == REC_SYSCALL_EXIT {
                    s.push_str(&format!(" ret={}", r.ret));
                }
                eprintln!("{s}");
            }
            _ => {}
        }
    }
}
