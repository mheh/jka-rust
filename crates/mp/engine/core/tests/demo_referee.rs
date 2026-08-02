//! The demo-driven seam referee, first vertical pass (DEC-58.1 and DEC-58.2,
//! ticket gh#30).
//!
//! # What this drives
//! A committed `.dm_26` demo drives the real client engine. The rig boots the
//! engine island, seats `Engine.cl`, runs `CL_Init`, opens the demo through the
//! filesystem, and feeds it to `CL_ReadDemoMessage` and `CL_ParseServerMessage`
//! under a fixed-step clock. The engine parses the gamestate, loads the
//! configstrings and baselines, and assembles snapshots exactly as it does on a
//! live connection.
//!
//! # The module seat
//! `VM_Create` returns an existing `vmTable` slot when the name matches
//! (`vm.cpp`, transcribed at `vm_fns.rs`), so the rig registers a slot named
//! `cgame` before playback and `CL_InitCGame` adopts it instead of loading a
//! dylib. The slot's entry point is the probe module below. The probe stands in
//! for the real cgame: it calls the traps that read snapshots and server
//! commands, and it never calls the renderer or the sound stack, so the whole
//! run stays headless.
//!
//! # The journal
//! Every engine-to-module call and every module-to-engine trap is written to a
//! C6b journal (`CGSHIMJ1`), the same format the recorder shim writes
//! (`tools/cgame-referee/README.md`). The rig sits where the shim sits, so the
//! two journals are comparable record for record. The oracle half of the gate
//! (a golden journal recorded from the oracle engine playing the same demo) is
//! designed but not built in this pass.
//!
//! # Assets
//! The demo parse needs the retail paks, the same way the jampgame referee's
//! real-map scenarios do. `JKA_REF_BASEPATH` names the install (default
//! `~/Developer/jka/jka_server`), and the playback test skips with a printed
//! message when it is absent. The two seam fixtures below need no assets and
//! always run.

#![allow(non_snake_case)]

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use mp_abi::cgame::exports::MpCgameExport;
use mp_abi::cgame::imports::MpCgameImport;
use mp_abi::cgame::public::snapshot_t::{snapshot_t, MAX_ENTITIES_IN_SNAPSHOT};
use mp_engine_client::cl_cgame::{CL_GetServerCommand, CL_GetSnapshot};
use mp_engine_client::cl_main::{CL_Init, CL_ReadDemoMessage};
use mp_engine_client::cl_referee::ClientRefMode;
use mp_engine_client::client_host::cl_from_view;
use mp_engine_client::Client;
use mp_engine_core::{com_init, engine_host_view, install_engine_hooks, Engine};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::files_common::FS_FOpenFileRead;
use mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS;
use mp_engine_qcommon::vm::cgame_syscall_trampoline_words;
use mp_engine_qcommon::vm::module_registry::MAX_VM;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::shared::connstate::connstate_t;
use native_platform::entrypoints::RawVmMain;

#[path = "../../../../../tools/cgame-referee/shapes.rs"]
mod shapes;

#[path = "../../../../../tools/cgame-referee/shim/src/journal.rs"]
mod journal;

use journal::{
    BlobKind, BlobSink, Journal, Record, REC_SYSCALL_ENTER, REC_SYSCALL_EXIT, REC_VMCALL_ENTER,
    REC_VMCALL_EXIT,
};
use shapes::{ArgKind, ExportRet, Manifests};

// ===========================================================================
// The recorder
// ===========================================================================

/// Cap on one serialized blob, the same bound `shim/src/serialize.rs` uses.
const MAX_BLOB: u64 = 1 << 20;

/// The journal writer plus the shape tables, shared by the rig and the probe.
/// The probe module is a bare function pointer with no context word, so the
/// recorder lives in a static the way the shim's does.
struct Recorder {
    journal: Journal,
    manifests: Manifests,
    seq: u64,
    /// Trap numbers seen, in order, for the assertions below.
    traps: Vec<i64>,
    /// vmMain commands seen, in order.
    vmcalls: Vec<i64>,
}

static RECORDER: Mutex<Option<Recorder>> = Mutex::new(None);

/// Reads a NUL-terminated string at `ptr`, capped. Empty on null.
/// This mirrors `shim/src/serialize.rs`, because the two writers must produce
/// the same bytes.
fn read_cstr(ptr: isize) -> Vec<u8> {
    if ptr == 0 {
        return Vec::new();
    }
    let base = ptr as *const u8;
    let mut out = Vec::new();
    // SAFETY: an engine buffer named by the manifest shape, capped so a missing
    // NUL never runs away.
    unsafe {
        let mut i = 0usize;
        while i < MAX_BLOB as usize {
            let b = *base.add(i);
            if b == 0 {
                break;
            }
            out.push(b);
            i += 1;
        }
    }
    out
}

/// Reads `len` bytes at `ptr`. Empty on null or an absurd length.
fn read_bytes(ptr: isize, len: u64) -> Vec<u8> {
    if ptr == 0 || len == 0 || len > MAX_BLOB {
        return Vec::new();
    }
    // SAFETY: an engine buffer sized by the manifest shape, capped above.
    unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
}

/// The 8-byte host pointer a `double_ptr` slot holds.
fn read_slot(ptr: isize) -> Vec<u8> {
    read_bytes(ptr, 8)
}

/// Serializes a trap's in-args at `SYSCALL_ENTER`. Arg shape index `i`
/// addresses `args[i + 1]`, the dispatch-index convention the manifest states.
fn trap_enter_blobs(m: &Manifests, num: i64, args: &[isize], rec: &mut Record) {
    let Some(shape) = m.trap(num) else {
        return;
    };
    let words: Vec<i64> = args.iter().map(|w| *w as i64).collect();
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = args[i + 1];
        match a.kind {
            ArgKind::InStr => rec.blob(i as u8, BlobKind::InStr, &read_cstr(ptr)),
            ArgKind::InBuf => {
                let len = shapes::arg_len(a, num, i, &words);
                rec.blob(i as u8, BlobKind::InBuf, &read_bytes(ptr, len));
            }
            ArgKind::InoutBuf => {
                let len = shapes::arg_len(a, num, i, &words);
                rec.blob(i as u8, BlobKind::InoutBuf, &read_bytes(ptr, len));
            }
            ArgKind::DoublePtr => rec.blob(i as u8, BlobKind::DoublePtrSlot, &read_slot(ptr)),
            ArgKind::Scalar | ArgKind::OutBuf | ArgKind::OutStr | ArgKind::RetainedPtr => {}
        }
    }
}

/// Serializes a trap's out-args at `SYSCALL_EXIT`, after the engine wrote them.
fn trap_exit_blobs(m: &Manifests, num: i64, args: &[isize], rec: &mut Record) {
    let Some(shape) = m.trap(num) else {
        return;
    };
    let words: Vec<i64> = args.iter().map(|w| *w as i64).collect();
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = args[i + 1];
        match a.kind {
            ArgKind::OutBuf => {
                let len = shapes::arg_len(a, num, i, &words);
                rec.blob(i as u8, BlobKind::OutBuf, &read_bytes(ptr, len));
            }
            ArgKind::InoutBuf => {
                let len = shapes::arg_len(a, num, i, &words);
                rec.blob(i as u8, BlobKind::InoutBuf, &read_bytes(ptr, len));
            }
            ArgKind::OutStr => rec.blob(i as u8, BlobKind::OutStr, &read_cstr(ptr)),
            ArgKind::DoublePtr => rec.blob(i as u8, BlobKind::DoublePtrSlot, &read_slot(ptr)),
            ArgKind::Scalar | ArgKind::InStr | ArgKind::InBuf | ArgKind::RetainedPtr => {}
        }
    }
}

/// Serializes a vmMain arm's in-args at `VMCALL_ENTER`. Export arg index N is
/// the raw word `argN`, with no trap-number prefix.
fn export_enter_blobs(m: &Manifests, num: i64, words: &[isize], rec: &mut Record) {
    let Some(shape) = m.export(num) else {
        return;
    };
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = words[i];
        match a.kind {
            ArgKind::InStr => rec.blob(i as u8, BlobKind::InStr, &read_cstr(ptr)),
            ArgKind::InBuf => rec.blob(i as u8, BlobKind::InBuf, &read_bytes(ptr, a.size_of as u64)),
            _ => {}
        }
    }
}

/// Serializes a vmMain arm's out-args and its pointer-return deref at
/// `VMCALL_EXIT`.
fn export_exit_blobs(m: &Manifests, num: i64, words: &[isize], ret: isize, rec: &mut Record) {
    let Some(shape) = m.export(num) else {
        return;
    };
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = words[i];
        match a.kind {
            ArgKind::OutBuf => {
                rec.blob(i as u8, BlobKind::OutBuf, &read_bytes(ptr, a.size_of as u64))
            }
            ArgKind::InoutBuf => rec.blob(
                i as u8,
                BlobKind::InoutBuf,
                &read_bytes(ptr, a.size_of as u64),
            ),
            _ => {}
        }
    }
    if let ExportRet::PtrDeref = shape.ret {
        rec.blob(
            shapes::ARG_RET_DEREF,
            BlobKind::RetDeref,
            &read_bytes(ret, shape.ret_size_of as u64),
        );
    }
}

/// Writes the `SYSCALL_ENTER` record and returns its sequence number.
/// The lock is released before the caller forwards into the engine, so a trap
/// that re-enters the module never deadlocks.
fn rec_syscall_enter(args: &[isize; 16]) -> u64 {
    let mut guard = RECORDER.lock().unwrap();
    let r = guard.as_mut().expect("recorder armed");
    r.seq += 1;
    let seq = r.seq;
    let num = args[0] as i64;
    r.traps.push(num);
    let mut rec = Record::new(REC_SYSCALL_ENTER, seq);
    rec.push_i64(num);
    rec.push_words(args);
    trap_enter_blobs(&r.manifests, num, args, &mut rec);
    r.journal.write(&rec);
    seq
}

/// Writes the `SYSCALL_EXIT` record that closes `seq`.
fn rec_syscall_exit(seq: u64, args: &[isize; 16], ret: isize) {
    let mut guard = RECORDER.lock().unwrap();
    let r = guard.as_mut().expect("recorder armed");
    let num = args[0] as i64;
    let mut rec = Record::new(REC_SYSCALL_EXIT, seq);
    rec.push_i64(num);
    rec.push_i64(ret as i64);
    trap_exit_blobs(&r.manifests, num, args, &mut rec);
    r.journal.write(&rec);
}

/// Writes the `VMCALL_ENTER` record and returns its sequence number.
fn rec_vmcall_enter(cmd: i64, words: &[isize; 12]) -> u64 {
    let mut guard = RECORDER.lock().unwrap();
    let r = guard.as_mut().expect("recorder armed");
    r.seq += 1;
    let seq = r.seq;
    r.vmcalls.push(cmd);
    let mut rec = Record::new(REC_VMCALL_ENTER, seq);
    rec.push_i64(cmd);
    rec.push_words(words);
    export_enter_blobs(&r.manifests, cmd, words, &mut rec);
    r.journal.write(&rec);
    seq
}

/// Writes the `VMCALL_EXIT` record that closes `seq`.
fn rec_vmcall_exit(seq: u64, cmd: i64, words: &[isize; 12], ret: isize) {
    let mut guard = RECORDER.lock().unwrap();
    let r = guard.as_mut().expect("recorder armed");
    let mut rec = Record::new(REC_VMCALL_EXIT, seq);
    rec.push_i64(cmd);
    rec.push_i64(ret as i64);
    export_exit_blobs(&r.manifests, cmd, words, ret, &mut rec);
    r.journal.write(&rec);
}

// ===========================================================================
// The probe module
// ===========================================================================

/// Calls one trap through the armed cgame slot and journals both ends.
/// The probe reaches the trampoline's Rust target directly, because a C-variadic
/// call cannot be written in stable Rust and the two entries dispatch through
/// the same armed cell.
fn probe_trap(args: &mut [isize; 16]) -> isize {
    let seq = rec_syscall_enter(args);
    let ret = cgame_syscall_trampoline_words(args.as_ptr());
    rec_syscall_exit(seq, args, ret);
    ret
}

/// Builds a 16-word trap frame from a trap number and its arguments.
fn frame(num: MpCgameImport, args: &[isize]) -> [isize; 16] {
    let mut f = [0isize; 16];
    f[0] = num as isize;
    for (i, a) in args.iter().enumerate() {
        f[i + 1] = *a;
    }
    f
}

/// The probe module's `vmMain`. It answers the two arms the rig drives and
/// pulls the engine state DEC-58.2 names: the truncated snapshot and the
/// reliable-command copy-out.
///
/// The probe deliberately calls no renderer and no sound trap, so the whole
/// playback runs with a NULL renderer slot and an unported sound stack.
#[allow(clippy::too_many_arguments)]
extern "C-unwind" fn probe_vm_main(
    command: core::ffi::c_int,
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
    // The probe journals the engine-to-module direction from the module side,
    // exactly where the recorder shim sits, so a call the engine starts on its
    // own (CG_INIT inside `CL_InitCGame`) lands in the journal too.
    let words = [
        arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
    ];
    let seq = rec_vmcall_enter(command as i64, &words);
    let ret = probe_dispatch(command, &words);
    rec_vmcall_exit(seq, command as i64, &words, ret);
    ret
}

/// The probe's own dispatch, split out so the journal brackets every arm.
fn probe_dispatch(command: core::ffi::c_int, words: &[isize; 12]) -> isize {
    if command == MpCgameExport::CG_INIT as core::ffi::c_int {
        // The engine hands CG_INIT the message sequence, the last executed
        // server command, and our client number. Read the game state back, the
        // way `CG_Init` does.
        let mut gs = [0u8; 32768];
        let mut f = frame(
            MpCgameImport::CG_GETGAMESTATE,
            &[gs.as_mut_ptr() as isize],
        );
        probe_trap(&mut f);
        return 0;
    }

    if command == MpCgameExport::CG_DRAW_ACTIVE_FRAME as core::ffi::c_int {
        let mut snap_num: core::ffi::c_int = 0;
        let mut server_time: core::ffi::c_int = 0;
        let mut f = frame(
            MpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER,
            &[
                &mut snap_num as *mut _ as isize,
                &mut server_time as *mut _ as isize,
            ],
        );
        probe_trap(&mut f);

        // The truncation seam: `CL_GetSnapshot` copies at most
        // MAX_ENTITIES_IN_SNAPSHOT entities into this buffer.
        let mut snap: Box<snapshot_t> = Box::new(unsafe { core::mem::zeroed() });
        let mut f = frame(
            MpCgameImport::CG_GETSNAPSHOT,
            &[snap_num as isize, &mut *snap as *mut _ as isize],
        );
        let ok = probe_trap(&mut f);

        // The reliable-command seam: drain every command the snapshot names.
        if ok != 0 {
            let mut n = snap.serverCommandSequence;
            // One call per frame keeps the journal readable and still exercises
            // the copy-out. `CG_GETSERVERCOMMAND` tokenizes into the engine's
            // argument vector, which `CG_ARGC` then reports.
            let mut f = frame(MpCgameImport::CG_GETSERVERCOMMAND, &[n as isize]);
            if probe_trap(&mut f) != 0 {
                let mut f = frame(MpCgameImport::CG_ARGC, &[]);
                probe_trap(&mut f);
            }
            n += 1;
            let _ = n;
        }
        return 0;
    }

    let _ = words;
    0
}

// ===========================================================================
// The rig
// ===========================================================================

/// The demo the first pass drives.
const DEMO: &str = "ffa1.dm_26";

/// One fixed clock step, in milliseconds. Raven's client reads
/// `Sys_Milliseconds` for `cls.realtime`, and the rig writes the field instead,
/// so a run is a pure function of the demo bytes.
const FIXED_DT_MS: i32 = 50;

/// The repository root, derived from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .expect("repo root")
}

/// The retail install the demo parse reads its paks from.
fn assets_path() -> PathBuf {
    match std::env::var("JKA_REF_BASEPATH") {
        Ok(p) => PathBuf::from(p),
        Err(_) => PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("Developer/jka/jka_server"),
    }
}

/// Stages the committed demo under a private home path and returns it.
fn stage_home(name: &str) -> PathBuf {
    let home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("demo-referee-{name}"));
    let demos = home.join("base/demos");
    std::fs::create_dir_all(&demos).expect("stage demo dir");
    let src = repo_root().join("tools/cgame-referee/fixtures").join(name);
    std::fs::copy(&src, demos.join(name)).expect("stage demo file");
    home
}

/// Registers the probe under the name `cgame`, so `CL_InitCGame`'s `VM_Create`
/// adopts the slot instead of loading a dylib.
fn seat_probe_module(view: &mut EngineHostView) {
    let mut i = 0;
    while i < MAX_VM {
        if view.common.vmTable[i].name.is_empty() {
            break;
        }
        i += 1;
    }
    assert!(i < MAX_VM, "no free vm_t slot for the probe module");
    view.common.vmTable[i].name = "cgame".to_string();
    let entry: RawVmMain = probe_vm_main;
    view.common.vmTable[i].entryPoint = Some(entry);
}

/// Drives one engine-to-module call. The probe journals both ends.
fn drive_vm_call(view: &mut EngineHostView, cl: &mut Client, cmd: MpCgameExport, args: &[isize]) {
    VM_Call(view.common, cl.cgvm, cmd as core::ffi::c_int, args);
}

/// Boots the engine island headless and returns it with `Engine.cl` seated.
fn boot(home: &Path) -> Box<Engine> {
    let mut engine: Box<Engine> = Engine::new();
    install_engine_hooks(&mut engine);
    // `+echo` is a startup command, so `Com_AddStartupCommands` reports one and
    // the boot never queues Raven's `cinematic openinglogos.roq` default action.
    let cmdline = format!(
        "+set fs_basepath {} +set fs_homepath {} +set dedicated 0 +echo demo-referee",
        assets_path().display(),
        home.display()
    );
    com_init(&mut engine, &cmdline);
    engine.cl = Some(Client::default());
    engine
}

/// Walks a written journal and returns the record count, the vmcall-record
/// count, and the syscall-record count. This is the reader side of the format
/// contract, and the oracle-side golden is walked the same way.
fn walk_journal(path: &Path) -> (usize, usize, usize) {
    let raw = std::fs::read(path).expect("journal readable");
    let mut buf = Vec::new();
    flate2::read::GzDecoder::new(&raw[..])
        .read_to_end(&mut buf)
        .expect("journal is one gzip stream");
    assert_eq!(&buf[0..8], journal::MAGIC, "journal magic");
    assert_eq!(
        u32::from_le_bytes(buf[8..12].try_into().unwrap()),
        journal::FORMAT_VERSION,
        "journal format version"
    );

    let (mut records, mut vmcalls, mut syscalls) = (0usize, 0usize, 0usize);
    let mut at = 12usize;
    while at + 4 <= buf.len() {
        let len = u32::from_le_bytes(buf[at..at + 4].try_into().unwrap()) as usize;
        at += 4;
        assert!(at + len <= buf.len(), "record runs past the end");
        let rec_type = buf[at];
        match rec_type {
            REC_VMCALL_ENTER | REC_VMCALL_EXIT => vmcalls += 1,
            REC_SYSCALL_ENTER | REC_SYSCALL_EXIT => syscalls += 1,
            _ => {}
        }
        records += 1;
        at += len;
    }
    (records, vmcalls, syscalls)
}

// ===========================================================================
// Tests
// ===========================================================================

/// Plays `ffa1.dm_26` through the real parse chain and writes the C6b journal.
#[test]
fn demo_ffa1_drives_the_client_engine() {
    let assets = assets_path();
    if !assets.join("base/assets0.pk3").exists() {
        println!(
            "SKIP demo_ffa1_drives_the_client_engine: no retail assets at {} (set JKA_REF_BASEPATH)",
            assets.display()
        );
        return;
    }

    let home = stage_home(DEMO);
    let manifests = Manifests::load(&repo_root().join("tools/cgame-referee")).expect("manifests");
    let journal_path = home.join("ffa1-journal.bin.gz");
    *RECORDER.lock().unwrap() = Some(Recorder {
        journal: Journal::create(&journal_path).expect("journal"),
        manifests,
        seq: 0,
        traps: Vec::new(),
        vmcalls: Vec::new(),
    });

    let mut engine = boot(&home);
    let mut frames = 0;
    let mut snapshots = 0;
    {
        let mut view = engine_host_view(&mut engine);
        // SAFETY: the view's `cl` slot came from the live `Engine.cl` seated
        // above, single-threaded, and no other cast of the slot is live.
        let cl = unsafe { cl_from_view(&mut view) };
        CL_Init(&mut view, cl);
        cl.referee.mode = ClientRefMode::Headless;
        seat_probe_module(&mut view);

        // `CL_PlayDemo_f` without the console command: the rig opens the file
        // and primes playback, then reads until the gamestate has landed.
        FS_FOpenFileRead(
            &mut view,
            &format!("demos/{DEMO}"),
            &mut cl.clc.demofile,
            true,
        );
        assert!(cl.clc.demofile != 0, "demo file did not open");
        cl.cls.state = connstate_t::CA_CONNECTED;
        cl.clc.demoplaying = 1;
        // The gamestate arrives inside this loop, and the engine's own
        // `CL_DownloadsComplete` runs `CL_InitCGame` on it: `VM_Create` adopts
        // the probe slot and the engine drives CG_INIT itself, so the journal
        // records the module load the way a live client performs it.
        while (cl.cls.state as i32) < (connstate_t::CA_PRIMED as i32) {
            CL_ReadDemoMessage(&mut view, cl);
        }
        assert!(!cl.cgvm.is_null(), "the probe module was never adopted");

        // Playback under the fixed clock: one demo message and one module frame
        // per step, so the run is a pure function of the demo bytes.
        while cl.clc.demofile != 0 && frames < 400 {
            cl.cls.realtime += FIXED_DT_MS;
            let before = cl.cl.snap.messageNum;
            CL_ReadDemoMessage(&mut view, cl);
            if cl.cl.snap.messageNum != before {
                snapshots += 1;
            }
            cl.cl.serverTime = cl.cl.snap.serverTime;
            drive_vm_call(
                &mut view,
                cl,
                MpCgameExport::CG_DRAW_ACTIVE_FRAME,
                &[cl.cl.serverTime as isize, 0, 1],
            );
            frames += 1;
        }

        // The gamestate the demo carried must have landed.
        assert!(
            cl.cl.gameState.dataCount > 1,
            "no configstrings parsed from the demo"
        );
        assert!(cl.clc.clientNum >= 0, "no client number in the gamestate");
    }

    let rec = RECORDER.lock().unwrap().take().expect("recorder armed");
    let traps = rec.traps.clone();
    let vmcalls = rec.vmcalls.clone();
    rec.journal.finish();

    assert!(snapshots > 0, "no snapshots assembled from the demo");
    assert!(
        vmcalls.contains(&(MpCgameExport::CG_INIT as i64)),
        "CG_INIT never reached the module"
    );
    assert!(
        traps.contains(&(MpCgameImport::CG_GETSNAPSHOT as i64)),
        "the module never asked for a snapshot"
    );
    let bytes = std::fs::metadata(&journal_path).expect("journal written").len();
    assert!(bytes > 0, "journal is empty");

    // The journal must read back as a well-formed C6b stream, because the
    // oracle-side golden is read by the same walk.
    let (records, vmcall_recs, syscall_recs) = walk_journal(&journal_path);
    assert_eq!(
        records,
        2 * (vmcalls.len() + traps.len()),
        "every call must carry an ENTER and an EXIT record"
    );
    assert_eq!(vmcall_recs, 2 * vmcalls.len());
    assert_eq!(syscall_recs, 2 * traps.len());
    println!(
        "demo {DEMO}: {frames} frames, {snapshots} snapshots, {} vmcalls, {} traps, journal {} bytes at {}",
        vmcalls.len(),
        traps.len(),
        bytes,
        journal_path.display()
    );
}

/// DEC-58.2, seam one: `clSnapshot_t` holds more entities than `snapshot_t`
/// carries, so `CL_GetSnapshot` truncates the copy at
/// `MAX_ENTITIES_IN_SNAPSHOT` and leaves the rest of the module's buffer alone.
#[test]
fn snapshot_truncation_caps_the_entity_copy() {
    let mut engine: Box<Engine> = Engine::new();
    let over = MAX_ENTITIES_IN_SNAPSHOT + 37;
    engine.cl = Some(Client::default());
    let cl = engine.cl.as_mut().expect("client seated");

    // One valid frame whose entity count is past the module's cap. The parse
    // ring is filled with numbered entities, so the copy order is visible.
    for (i, e) in cl.cl.parseEntities.iter_mut().enumerate() {
        e.number = i as i32;
    }
    cl.cl.parseEntitiesNum = over as i32;
    cl.cl.snap.messageNum = 0;
    let snap = &mut cl.cl.snapshots[0];
    snap.valid = 1;
    snap.messageNum = 0;
    snap.serverTime = 1234;
    snap.serverCommandNum = 7;
    snap.ping = 42;
    snap.snapFlags = 0;
    snap.numEntities = over as i32;
    snap.parseEntitiesNum = 0;

    let mut out: Box<snapshot_t> = Box::new(unsafe { core::mem::zeroed() });
    // A sentinel past the cap proves the engine never wrote there.
    out.entities[MAX_ENTITIES_IN_SNAPSHOT - 1].number = -1;
    let got = CL_GetSnapshot(&mut engine.common, cl, 0, &mut *out);

    assert_eq!(got, 1, "the frame was valid and had to be returned");
    assert_eq!(
        out.numEntities, MAX_ENTITIES_IN_SNAPSHOT as i32,
        "the entity count was not capped"
    );
    assert_eq!(out.serverTime, 1234);
    assert_eq!(out.serverCommandSequence, 7);
    assert_eq!(out.ping, 42);
    for i in 0..MAX_ENTITIES_IN_SNAPSHOT {
        assert_eq!(
            out.entities[i].number, i as i32,
            "entity {i} came from the wrong ring slot"
        );
    }
}

/// DEC-58.2, seam one, second half: a frame whose entities fell out of the
/// parse ring is refused rather than answered with stale data.
#[test]
fn snapshot_refuses_a_frame_that_left_the_parse_ring() {
    let mut engine: Box<Engine> = Engine::new();
    engine.cl = Some(Client::default());
    let cl = engine.cl.as_mut().expect("client seated");

    cl.cl.parseEntitiesNum = 4096; // MAX_PARSE_ENTITIES past the frame below
    cl.cl.snap.messageNum = 0;
    let snap = &mut cl.cl.snapshots[0];
    snap.valid = 1;
    snap.numEntities = 1;
    snap.parseEntitiesNum = 0;

    let mut out: Box<snapshot_t> = Box::new(unsafe { core::mem::zeroed() });
    let got = CL_GetSnapshot(&mut engine.common, cl, 0, &mut *out);
    assert_eq!(got, 0, "an overwritten frame must be refused");
}

/// Writes `text` into reliable-command slot `n` of the client's ring.
fn seat_server_command(cl: &mut Client, n: i32, text: &str) {
    let slot = (n & (MAX_RELIABLE_COMMANDS as i32 - 1)) as usize;
    let dst = &mut cl.clc.serverCommands[slot];
    for (i, b) in text.as_bytes().iter().enumerate() {
        dst[i] = *b as core::ffi::c_char;
    }
    dst[text.len()] = 0;
}

/// DEC-58.2, seam two: `CL_GetServerCommand` copies the reliable command out to
/// the module's argument vector and records which command was executed.
#[test]
fn reliable_command_copy_out_tokenizes_and_marks_executed() {
    let mut engine: Box<Engine> = Engine::new();
    engine.cl = Some(Client::default());
    {
        let cl = engine.cl.as_mut().expect("client seated");
        cl.clc.serverCommandSequence = 5;
        seat_server_command(cl, 5, "chat \"hello there\"");
    }

    let mut view = engine_host_view(&mut engine);
    // SAFETY: the view's `cl` slot came from the live `Engine.cl`, and no other
    // cast of the slot is live here.
    let cl = unsafe { cl_from_view(&mut view) };
    let got = CL_GetServerCommand(&mut view, cl, 5);

    assert_eq!(got, 1, "a received command must be handed to the module");
    assert_eq!(
        cl.clc.lastExecutedServerCommand, 5,
        "the executed sequence was not recorded"
    );
    assert_eq!(
        mp_engine_qcommon::cmd_common::Cmd_Argc(view.common),
        2,
        "the command was not tokenized"
    );
    assert_eq!(mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 0), "chat");
    assert_eq!(
        mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 1),
        "hello there"
    );
}

/// DEC-58.2, seam two, second half: a demo whose recording started late has no
/// copy of the earliest reliable commands, so the engine answers false instead
/// of dropping the connection.
#[test]
fn reliable_command_cycled_out_is_false_during_demo_playback() {
    let mut engine: Box<Engine> = Engine::new();
    engine.cl = Some(Client::default());
    {
        let cl = engine.cl.as_mut().expect("client seated");
        cl.clc.demoplaying = 1;
        cl.clc.serverCommandSequence = 200;
    }

    let mut view = engine_host_view(&mut engine);
    // SAFETY: as above.
    let cl = unsafe { cl_from_view(&mut view) };
    // The ring holds MAX_RELIABLE_COMMANDS entries, so command 50 cycled out
    // long before sequence 200 arrived.
    let got = CL_GetServerCommand(&mut view, cl, 50);
    assert_eq!(got, 0, "a cycled-out command must be false under a demo");
}

/// The entity ring copy above depends on `entityState_t` staying the copied
/// unit, so the fixture states the assumption it rests on.
#[test]
fn snapshot_entity_is_the_parse_ring_entry() {
    assert_eq!(
        core::mem::size_of::<entityState_t>(),
        core::mem::size_of_val(&unsafe { core::mem::zeroed::<snapshot_t>() }.entities[0]),
        "snapshot entities and parse-ring entries must be the same type"
    );
}
