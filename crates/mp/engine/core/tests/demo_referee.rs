//! The demo-driven seam referee (DEC-58.1 and DEC-58.2, ticket gh#30).
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
//! dylib. The slot's entry point is the probe module
//! (`tools/cgame-referee/probe/src/probe.rs`), the same body the standalone
//! cdylib runs inside the oracle engine.
//!
//! # The gate
//! The probe writes a C6b journal (`CGSHIMJ1`, `tools/cgame-referee/README.md`).
//! `tools/cgame-referee/goldens/` holds the twin journal recorded from the
//! oracle engine over the same demo, and the gate walks both record by record
//! and compares them byte for byte. `journal_diff.rs` lists the four host-state
//! exclusions.
//!
//! # Assets
//! The demo parse needs the retail paks, the same way the jampgame referee's
//! real-map scenarios do. `JKA_REF_BASEPATH` names the install (default
//! `~/Developer/jka/jka_server`), and the playback tests skip with a printed
//! message when it is absent (DEC-62.1). The golden-shape test and the seam
//! fixtures need no assets and always run.

#![allow(non_snake_case)]

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
use mp_engine_qcommon::cmd_common::{Cmd_Argc, Cmd_Argv};
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

#[path = "../../../../../tools/cgame-referee/probe/src/probe.rs"]
mod probe;

#[path = "../../../../../tools/cgame-referee/journal_diff.rs"]
mod journal_diff;

use journal_diff::{bracket_snapshots, census, diff, exclusions, read_journal, JournalRecord};
use journal::{REC_SYSCALL_ENTER, REC_SYSCALL_EXIT, REC_VMCALL_ENTER, REC_VMCALL_EXIT};
use probe::{Probe, DEFAULT_BRACKET_CAP};
use shapes::Manifests;

// ===========================================================================
// The probe seat
// ===========================================================================

/// The one live probe. A `vm_t` entry point is a bare function pointer with no
/// context word, so the probe lives in a static the way the recorder shim's
/// does. The playback tests run one at a time behind `RIG`.
static PROBE: Mutex<Option<Probe>> = Mutex::new(None);

/// Serializes the playback tests. They share `PROBE`, the engine boot, and the
/// staged home directory, so only one may run at a time.
static RIG: Mutex<()> = Mutex::new(());

/// Forwards one trap frame into the armed cgame slot.
///
/// The probe reaches the trampoline's Rust target directly, because a C-variadic
/// call cannot be written in stable Rust and the two entries dispatch through
/// the same armed cell.
fn forward(args: &mut [isize; 16]) -> isize {
    cgame_syscall_trampoline_words(args.as_ptr())
}

/// The probe module's `vmMain`, seated in the `cgame` slot.
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
    let words = [
        arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
    ];
    let mut guard = PROBE.lock().unwrap();
    match guard.as_mut() {
        Some(p) => p.vm_main(command as i64, &words),
        None => 0,
    }
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

// ===========================================================================
// The rig
// ===========================================================================

/// The four committed demos the referee covers.
const DEMOS: &[&str] = &["ffa1", "sabers1", "spectator", "swoop1"];

/// One fixed clock step, in milliseconds. Raven's client reads
/// `Sys_Milliseconds` for `cls.realtime`, and the rig writes the field instead,
/// so a run is a pure function of the demo bytes.
const FIXED_DT_MS: i32 = 50;

/// Hard bound on demo messages one playback reads. The bracket cap normally
/// ends the run, and this stops a demo that yields no snapshots from hanging.
const MAX_MESSAGES: u32 = 200_000;

/// Brackets the gate lets the snapshot alignment drop off the front. The oracle
/// engine reaches `CA_ACTIVE` one snapshot later than our rig, so the real
/// number is 1. A wider allowance would let a broken run pass on a short
/// aligned range.
const MAX_ALIGNMENT_SKIP: usize = 4;

/// The repository root, derived from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .expect("repo root")
}

/// The `tools/cgame-referee` directory, which holds both manifests, the
/// fixtures, and the goldens.
fn referee_dir() -> PathBuf {
    repo_root().join("tools/cgame-referee")
}

/// The retail install the demo parse reads its paks from.
fn assets_path() -> PathBuf {
    match std::env::var("JKA_REF_BASEPATH") {
        Ok(p) => PathBuf::from(p),
        Err(_) => PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("Developer/jka/jka_server"),
    }
}

/// True when the retail paks are present. DEC-62.1 keeps the playback tests
/// gated on this and never bypasses `FS_CheckPak0`.
fn assets_present() -> bool {
    assets_path().join("base/assets0.pk3").exists()
}

/// Stages the committed demo under a private home path and returns it.
fn stage_home(demo: &str) -> PathBuf {
    let home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("demo-referee-{demo}"));
    let demos = home.join("base/demos");
    std::fs::create_dir_all(&demos).expect("stage demo dir");
    let name = format!("{demo}.dm_26");
    let src = referee_dir().join("fixtures").join(&name);
    std::fs::copy(&src, demos.join(&name)).expect("stage demo file");
    home
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

/// What one playback produced.
struct Playback {
    journal: PathBuf,
    messages: u32,
    brackets: u32,
    traps: Vec<i64>,
    vmcalls: Vec<i64>,
}

/// Plays one demo through the real parse chain and writes the probe's journal.
/// `cap` bounds the journaled snapshots.
fn play(demo: &str, cap: u32) -> Playback {
    let home = stage_home(demo);
    let journal_path = home.join(format!("{demo}.journal.gz"));
    *PROBE.lock().unwrap() = Some(
        Probe::new(&journal_path, &referee_dir(), cap, forward as probe::TrapFn)
            .expect("probe recording opens"),
    );

    let name = format!("{demo}.dm_26");
    let mut engine = boot(&home);
    let mut messages = 0u32;
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
        FS_FOpenFileRead(&mut view, &format!("demos/{name}"), &mut cl.clc.demofile, true);
        assert!(cl.clc.demofile != 0, "demo file {name} did not open");
        cl.cls.state = connstate_t::CA_CONNECTED;
        cl.clc.demoplaying = 1;
        // The gamestate arrives inside this loop, and the engine's own
        // `CL_DownloadsComplete` runs `CL_InitCGame` on it: `VM_Create` adopts
        // the probe slot and the engine drives CG_INIT itself, so the journal
        // records the module load the way a live client performs it.
        while (cl.cls.state as i32) < (connstate_t::CA_PRIMED as i32) {
            CL_ReadDemoMessage(&mut view, cl);
            assert!(cl.clc.demofile != 0, "{name} ended before the gamestate");
        }
        assert!(!cl.cgvm.is_null(), "the probe module was never adopted");

        // Playback under the fixed clock: one demo message and one module frame
        // per step, so the run is a pure function of the demo bytes. The probe
        // journals only the frames that carried a new snapshot, so the record
        // stream never depends on this cadence.
        while cl.clc.demofile != 0 && messages < MAX_MESSAGES && !probe_done() {
            cl.cls.realtime += FIXED_DT_MS;
            CL_ReadDemoMessage(&mut view, cl);
            cl.cl.serverTime = cl.cl.snap.serverTime;
            VM_Call(
                view.common,
                cl.cgvm,
                MpCgameExport::CG_DRAW_ACTIVE_FRAME as core::ffi::c_int,
                &[cl.cl.serverTime as isize, 0, 1],
            );
            messages += 1;
        }

        // The gamestate the demo carried must have landed.
        assert!(
            cl.cl.gameState.dataCount > 1,
            "no configstrings parsed from {name}"
        );
        assert!(cl.clc.clientNum >= 0, "no client number in {name}");
    }

    let mut probe = PROBE.lock().unwrap().take().expect("recorder armed");
    probe.finish();
    Playback {
        journal: journal_path,
        messages,
        brackets: probe.brackets(),
        traps: probe.traps.clone(),
        vmcalls: probe.vmcalls.clone(),
    }
}

/// True once the probe closed its journal at the bracket cap.
fn probe_done() -> bool {
    PROBE
        .lock()
        .unwrap()
        .as_ref()
        .map(|p| p.done())
        .unwrap_or(true)
}

/// The committed golden for one demo.
fn golden_path(demo: &str) -> PathBuf {
    referee_dir().join("goldens").join(format!("{demo}.journal.gz"))
}

/// Prints the record census of one journal.
fn print_census(label: &str, records: &[JournalRecord]) {
    let c = census(records);
    println!(
        "  {label}: {} records ({} vmcall, {} syscall)",
        records.len(),
        c.get(&REC_VMCALL_ENTER).unwrap_or(&0) + c.get(&REC_VMCALL_EXIT).unwrap_or(&0),
        c.get(&REC_SYSCALL_ENTER).unwrap_or(&0) + c.get(&REC_SYSCALL_EXIT).unwrap_or(&0),
    );
}

/// Diffs one playback against its golden and returns the finding lines.
///
/// A run that compares almost no brackets would pass on nothing, so the gate
/// also fails when the aligned range is short.
fn gate(demo: &str, manifests: &Manifests, play: &Playback, golden: &Path) -> Vec<String> {
    let ours = read_journal(&play.journal).expect("our journal reads back");
    let theirs = read_journal(golden).expect("golden reads back");
    println!(
        "demo {demo}: {} messages, {} brackets",
        play.messages, play.brackets
    );
    print_census("ours  ", &ours);
    print_census("golden", &theirs);

    let report = diff(manifests, &ours, &theirs, 20);
    println!(
        "  compared {} brackets (skipped {} ours, {} golden before the first common snapshot)",
        report.compared, report.skipped_ours, report.skipped_golden
    );
    let mut lines: Vec<String> = report
        .findings
        .into_iter()
        .map(|f| format!("  {demo} record {}: {}", f.index, f.what))
        .collect();
    let want = (play.brackets as usize).saturating_sub(MAX_ALIGNMENT_SKIP);
    if report.compared < want {
        lines.push(format!(
            "  {demo}: only {} brackets aligned, wanted at least {want}",
            report.compared
        ));
    }
    lines
}

// ===========================================================================
// The golden-shape check - no assets needed
// ===========================================================================

/// The goldens are committed, so this runs everywhere and needs no assets.
/// A recording that skipped a snapshot would make the byte gate report a
/// difference our engine did not cause, so the density check guards the
/// goldens themselves.
#[test]
fn goldens_are_well_formed_and_skip_no_snapshot() {
    for demo in DEMOS {
        let path = golden_path(demo);
        assert!(
            path.exists(),
            "missing golden {}. Record it with tools/cgame-referee/record-golden.sh {demo}",
            path.display()
        );
        let records = read_journal(&path).expect("golden reads back");
        assert!(!records.is_empty(), "{demo}: golden holds no records");

        let opens = records
            .iter()
            .filter(|r| r.rec_type == REC_VMCALL_ENTER || r.rec_type == REC_SYSCALL_ENTER)
            .count();
        let closes = records
            .iter()
            .filter(|r| r.rec_type == REC_VMCALL_EXIT || r.rec_type == REC_SYSCALL_EXIT)
            .count();
        assert_eq!(opens, closes, "{demo}: golden has an unbalanced bracket");

        let vmcalls = records
            .iter()
            .filter(|r| r.rec_type == REC_VMCALL_ENTER)
            .count();
        assert!(
            vmcalls >= 2,
            "{demo}: golden holds {vmcalls} vmcalls, so playback never started"
        );

        let numbers = bracket_snapshots(&records);
        assert!(!numbers.is_empty(), "{demo}: golden read no snapshot number");
        for pair in numbers.windows(2) {
            assert_eq!(
                pair[1],
                pair[0] + 1,
                "{demo}: golden jumped from snapshot {} to {}, so the recording dropped a frame",
                pair[0],
                pair[1]
            );
        }
        println!(
            "golden {demo}: {} records, {} brackets, snapshots {}..{}",
            records.len(),
            vmcalls,
            numbers[0],
            numbers[numbers.len() - 1]
        );
    }
}

/// The probe's compile-time sizes must be the ones the seam really carries, or
/// every blob would be the wrong length.
#[test]
fn probe_buffer_sizes_match_the_seam_types() {
    assert_eq!(
        core::mem::size_of::<snapshot_t>(),
        probe::SNAPSHOT_SIZE,
        "the probe's snapshot buffer is not sizeof(snapshot_t)"
    );
    assert_eq!(probe::CG_INIT, MpCgameExport::CG_INIT as i64);
    assert_eq!(probe::CG_SHUTDOWN, MpCgameExport::CG_SHUTDOWN as i64);
    assert_eq!(
        probe::CG_DRAW_ACTIVE_FRAME,
        MpCgameExport::CG_DRAW_ACTIVE_FRAME as i64
    );
    assert_eq!(probe::CG_ARGC, MpCgameImport::CG_ARGC as i64);
    assert_eq!(probe::CG_ARGV, MpCgameImport::CG_ARGV as i64);
    assert_eq!(
        probe::CG_SENDCONSOLECOMMAND,
        MpCgameImport::CG_SENDCONSOLECOMMAND as i64
    );
    assert_eq!(probe::CG_GETGAMESTATE, MpCgameImport::CG_GETGAMESTATE as i64);
    assert_eq!(
        probe::CG_GETCURRENTSNAPSHOTNUMBER,
        MpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER as i64
    );
    assert_eq!(probe::CG_GETSNAPSHOT, MpCgameImport::CG_GETSNAPSHOT as i64);
    assert_eq!(
        probe::CG_GETSERVERCOMMAND,
        MpCgameImport::CG_GETSERVERCOMMAND as i64
    );
}

// ===========================================================================
// The byte gate - asset-gated (DEC-62.1)
// ===========================================================================

/// Every demo plays through our engine and its journal must equal the oracle
/// golden byte for byte, outside the named exclusions.
#[test]
fn demos_match_the_oracle_goldens() {
    let _rig = RIG.lock().unwrap_or_else(|e| e.into_inner());
    if !assets_present() {
        println!(
            "SKIP demos_match_the_oracle_goldens: no retail assets at {} (set JKA_REF_BASEPATH)",
            assets_path().display()
        );
        return;
    }
    println!("gate exclusions: {}", exclusions().join("; "));

    let manifests = Manifests::load(&referee_dir()).expect("manifests");
    let mut findings: Vec<String> = Vec::new();
    for demo in DEMOS {
        let run = play(demo, DEFAULT_BRACKET_CAP);
        assert!(run.brackets > 0, "{demo}: no snapshot ever reached the module");
        assert!(
            run.vmcalls.contains(&(MpCgameExport::CG_INIT as i64)),
            "{demo}: CG_INIT never reached the module"
        );
        assert!(
            run.traps.contains(&(MpCgameImport::CG_GETSNAPSHOT as i64)),
            "{demo}: the module never asked for a snapshot"
        );
        findings.extend(gate(demo, &manifests, &run, &golden_path(demo)));
    }
    assert!(
        findings.is_empty(),
        "the demo referee found {} differences:\n{}",
        findings.len(),
        findings.join("\n")
    );
}

/// The DEC-62.2 extended check: the same gate over the whole demo rather than
/// the committed 400-bracket bound. `JKA_REF_FULL_GOLDENS` names a directory of
/// locally recorded full-length goldens, which stay out of git for size.
#[test]
fn full_demos_match_local_goldens() {
    let _rig = RIG.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(dir) = std::env::var("JKA_REF_FULL_GOLDENS") else {
        println!("SKIP full_demos_match_local_goldens: JKA_REF_FULL_GOLDENS is not set");
        return;
    };
    if !assets_present() {
        println!("SKIP full_demos_match_local_goldens: no retail assets");
        return;
    }
    let dir = PathBuf::from(dir);
    let manifests = Manifests::load(&referee_dir()).expect("manifests");
    let mut findings: Vec<String> = Vec::new();
    for demo in DEMOS {
        let golden = dir.join(format!("{demo}.journal.gz"));
        if !golden.exists() {
            println!("  skip {demo}: no full golden at {}", golden.display());
            continue;
        }
        let run = play(demo, u32::MAX);
        findings.extend(gate(demo, &manifests, &run, &golden));
    }
    assert!(
        findings.is_empty(),
        "the full-demo check found {} differences:\n{}",
        findings.len(),
        findings.join("\n")
    );
}

// ===========================================================================
// The seam fixtures - no assets needed
// ===========================================================================

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
        Cmd_Argc(view.common),
        2,
        "the command was not tokenized"
    );
    assert_eq!(Cmd_Argv(view.common, 0), "chat");
    assert_eq!(Cmd_Argv(view.common, 1), "hello there");
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
