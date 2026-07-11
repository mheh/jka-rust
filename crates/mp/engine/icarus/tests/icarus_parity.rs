//! Differential parity: the Rust `mp_engine_icarus` port must reproduce, byte
//! for byte, the dumps produced by the UNMODIFIED Raven C++ ICARUS TUs compiled
//! by `tools/icarus-oracle/build.sh` (goldens under `tools/icarus-oracle/goldens/`).
//!
//! The three units mirror `docs/subsystems/icarus.md` § Verification strategy and
//! the three oracle dumpers exactly:
//! * [`blockstream_matches_oracle_goldens`] mirrors `dump_blockstream.cpp` (unit 1).
//! * [`q3_registers_matches_oracle_golden`] mirrors `dump_registers.cpp` (unit 2).
//! * [`endtoend_matches_oracle_golden`] mirrors `dump_endtoend.cpp` + `mockhost.cpp`
//!   (unit 3), driving the port through `mp_host_interface::mock::MockHost`.
//!
//! Fixtures/goldens are read from `tools/icarus-oracle/` and are never edited; the
//! Rust dump format matches each dumper's `printf`s character for character.

use std::ffi::CString;
use std::fmt::Write as _;
use std::os::raw::c_char;
use std::path::PathBuf;

use mp_engine_icarus::blockstream::cblock::Block;
use mp_engine_icarus::blockstream::cblock_stream::BlockStream;
use mp_engine_icarus::game_interface::{
    icarus_init, icarus_init_ent, icarus_maintain_task_manager, icarus_run_script,
};
use mp_engine_icarus::q3_registers::{
    q3_declare_variable, q3_free_variable, q3_get_float_variable, q3_get_string_variable,
    q3_get_vector_variable, q3_variable_declared, Q3_InitVariables, Q3_SetFloatVariable,
    Q3_SetStringVariable, Q3_SetVectorVariable,
};
use mp_engine_icarus::Icarus;
use mp_host_interface::mock::MockHost;
use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::game_export_t::gameExport_t;

/// Repo-relative `tools/icarus-oracle` root (this crate is `crates/mp/engine/icarus`).
fn oracle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/icarus-oracle")
}

// ---------------------------------------------------------------------------
// Unit 1: BlockStream reader (mirrors dump_blockstream.cpp)
// ---------------------------------------------------------------------------

/// Reproduce `dump_blockstream.cpp`'s parsed record-stream dump for one `.IBI`
/// blob: `Open` → `BlockAvailable` → `ReadBlock` → per-member `GetInfo`.
fn dump_blockstream(buf: &[u8]) -> String {
    let mut out = String::new();
    // `printf("== blockstream %ld bytes ==\n", size)`.
    writeln!(out, "== blockstream {} bytes ==", buf.len()).unwrap();

    let mut stream = BlockStream::default();
    if stream.open(buf) == 0 {
        // `printf("open error\n== end ==\n")`.
        out.push_str("open error\n== end ==\n");
        return out;
    }
    out.push_str("open ok\n");

    let mut block_idx = 0;
    while stream.block_available() != 0 {
        let mut block = Block {
            m_members: Vec::new(),
            m_id: 0,
            m_flags: 0,
        };
        if stream.read_block(&mut block) == 0 {
            out.push_str("readblock error\n");
            break;
        }
        let n_members = block.get_num_members();
        // `printf("block %d id=%d flags=%u members=%d\n", ...)` — flags is the
        // `(unsigned)GetFlags()` cast (u8 → unsigned decimal).
        writeln!(
            out,
            "block {} id={} flags={} members={}",
            block_idx,
            block.get_block_id(),
            block.get_flags(),
            n_members
        )
        .unwrap();
        for j in 0..n_members {
            // GetMember(j) is always in range here; GetInfo folds the three
            // out-params (id, size, data) to a tuple (§C7).
            let member = block.get_member(j).expect("in-range member");
            let (mid, msize, data) = member.get_info();
            write!(out, "  m{} id={} size={} bytes=", j, mid, msize).unwrap();
            // `for (k=0; k<msize; k++) printf("%02x", p ? p[k] : 0)`.
            for k in 0..msize as usize {
                write!(out, "{:02x}", data.get(k).copied().unwrap_or(0)).unwrap();
            }
            out.push('\n');
        }
        block.free();
        block_idx += 1;
    }
    out.push_str("== end ==\n");
    out
}

#[test]
fn blockstream_matches_oracle_goldens() {
    let root = oracle_root();
    let mut checked = 0;

    for entry in std::fs::read_dir(root.join("fixtures")).expect("fixtures dir") {
        let fixture = entry.expect("dir entry").path();
        if fixture.extension().and_then(|e| e.to_str()) != Some("IBI") {
            continue;
        }
        let name = fixture.file_stem().unwrap().to_str().unwrap().to_string();
        let buf = std::fs::read(&fixture).expect("read fixture");
        let golden_path = root.join("goldens").join(format!("blockstream_{name}.txt"));
        let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
            panic!("missing golden {golden_path:?} — run tools/icarus-oracle/build.sh --regen")
        });

        assert_eq!(
            dump_blockstream(&buf),
            golden,
            "fixture {name} diverges from the BlockStream C++ oracle"
        );
        checked += 1;
    }

    assert!(
        checked >= 4,
        "expected the full .IBI fixture set, found {checked}"
    );
}

// ---------------------------------------------------------------------------
// Unit 2: Q3_Registers (mirrors dump_registers.cpp)
// ---------------------------------------------------------------------------

// `Q3_DeclareVariable`'s `switch(type)` keys off the tokenizer's token values,
// not the local `VTYPE_*` enum. Reproduced here exactly as the dumper passes
// them (`tokenizer.h:63-73`, `interpreter.h:23`).
/// Raven `TK_STRING`. Source: `oracle/codemp/icarus/tokenizer.h:69`
const TK_STRING: i32 = 4;
/// Raven `TK_INT` (unknown declare type → ignored). Source: `oracle/codemp/icarus/tokenizer.h:70`
const TK_INT: i32 = 5;
/// Raven `TK_FLOAT`. Source: `oracle/codemp/icarus/tokenizer.h:72`
const TK_FLOAT: i32 = 6;
/// Raven `TK_VECTOR`. Source: `oracle/codemp/icarus/interpreter.h:23`
const TK_VECTOR: i32 = 14;

/// Reproduce `dump_registers.cpp`'s `dumpState`: the sorted (`std::map` order)
/// float/string/vector stores plus `numVariables`.
fn dump_state(icarus: &Icarus, tag: &str, out: &mut String) {
    writeln!(out, "-- {} : numVariables={}", tag, icarus.num_variables).unwrap();

    let mut floats: Vec<_> = icarus.var_floats.iter().collect();
    floats.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in floats {
        writeln!(out, "  F |{}|={:.3}", k, v).unwrap();
    }

    let mut strings: Vec<_> = icarus.var_strings.iter().collect();
    strings.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in strings {
        writeln!(out, "  S |{}|=|{}|", k, v).unwrap();
    }

    let mut vectors: Vec<_> = icarus.var_vectors.iter().collect();
    vectors.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in vectors {
        writeln!(out, "  V |{}|=|{}|", k, v).unwrap();
    }
}

#[test]
fn q3_registers_matches_oracle_golden() {
    let golden_path = oracle_root().join("goldens").join("q3_registers.txt");
    let golden = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|_| panic!("missing golden {golden_path:?}"));

    let mut icarus = Icarus::default();
    let mut host = MockHost::new();
    let mut out = String::new();

    writeln!(out, "== q3_registers ==").unwrap();
    Q3_InitVariables(&mut icarus, &mut host);

    // Declare each type; duplicate + bad-type are no-ops.
    q3_declare_variable(&mut icarus, &mut host, TK_FLOAT, "health");
    q3_declare_variable(&mut icarus, &mut host, TK_STRING, "name");
    q3_declare_variable(&mut icarus, &mut host, TK_VECTOR, "spot");
    q3_declare_variable(&mut icarus, &mut host, TK_FLOAT, "health"); // duplicate -> ignored
    q3_declare_variable(&mut icarus, &mut host, TK_INT, "bogus"); // unknown type -> ignored
    dump_state(&icarus, "declared", &mut out);

    // VariableDeclared queries (VTYPE_NONE/FLOAT/STRING/VECTOR).
    writeln!(
        out,
        "decl health={} name={} spot={} ghost={}",
        q3_variable_declared(&mut icarus, &mut host, "health"),
        q3_variable_declared(&mut icarus, &mut host, "name"),
        q3_variable_declared(&mut icarus, &mut host, "spot"),
        q3_variable_declared(&mut icarus, &mut host, "ghost"),
    )
    .unwrap();

    // Set values; setting an undeclared name fails.
    Q3_SetFloatVariable(&mut icarus, "health", 42.5);
    Q3_SetStringVariable(&mut icarus, "name", "kyle");
    Q3_SetVectorVariable(&mut icarus, "spot", "1.0 2.0 3.0");
    writeln!(
        out,
        "set ghost={}",
        Q3_SetStringVariable(&mut icarus, "ghost", "x")
    )
    .unwrap();
    dump_state(&icarus, "set", &mut out);

    // Gets (out-param → Option, §C7). The C++ seeds fv=-1.0 before the call.
    let gf = q3_get_float_variable(&mut icarus, &mut host, "health");
    writeln!(
        out,
        "get health ok={} val={:.3}",
        gf.is_some() as i32,
        gf.unwrap_or(-1.0)
    )
    .unwrap();

    let gs = q3_get_string_variable(&mut icarus, &mut host, "name");
    writeln!(
        out,
        "get name ok={} val=|{}|",
        gs.is_some() as i32,
        gs.as_deref().unwrap_or("(null)")
    )
    .unwrap();

    let gv = q3_get_vector_variable(&mut icarus, &mut host, "spot");
    let vv = gv.unwrap_or([0.0, 0.0, 0.0]);
    writeln!(
        out,
        "get spot ok={} val={:.3} {:.3} {:.3}",
        gv.is_some() as i32,
        vv[0],
        vv[1],
        vv[2]
    )
    .unwrap();

    writeln!(
        out,
        "get ghost ok={}",
        q3_get_float_variable(&mut icarus, &mut host, "ghost").is_some() as i32
    )
    .unwrap();

    // Free one, then re-query.
    q3_free_variable(&mut icarus, &mut host, "name");
    dump_state(&icarus, "freed name", &mut out);

    // Cap test: declare past MAX_VARIABLES (32) and confirm the guard.
    for i in 0..40 {
        let n = format!("v{:02}", i);
        q3_declare_variable(&mut icarus, &mut host, TK_FLOAT, &n);
    }
    writeln!(
        out,
        "after flood numVariables={} floats={}",
        icarus.num_variables,
        icarus.var_floats.len()
    )
    .unwrap();

    writeln!(out, "== end ==").unwrap();

    assert_eq!(out, golden, "Q3_Registers diverges from the C++ oracle");
}

// ---------------------------------------------------------------------------
// Unit 3: end-to-end sequencer (mirrors dump_endtoend.cpp + mockhost.cpp)
// ---------------------------------------------------------------------------

/// `GAME_ICARUS_*` callnum → name, mirroring `dump_endtoend.cpp`'s `callName`
/// (`kIcarusCalls`). The callnum is the game-export index the icarus seam passes
/// to `VM_Call(gvm, GAME_ICARUS_*)`.
fn call_name(v: i32) -> &'static str {
    match v {
        x if x == gameExport_t::GAME_ICARUS_PLAYSOUND as i32 => "PLAYSOUND",
        x if x == gameExport_t::GAME_ICARUS_SET as i32 => "SET",
        x if x == gameExport_t::GAME_ICARUS_LERP2POS as i32 => "LERP2POS",
        x if x == gameExport_t::GAME_ICARUS_LERP2ORIGIN as i32 => "LERP2ORIGIN",
        x if x == gameExport_t::GAME_ICARUS_LERP2ANGLES as i32 => "LERP2ANGLES",
        x if x == gameExport_t::GAME_ICARUS_GETTAG as i32 => "GETTAG",
        x if x == gameExport_t::GAME_ICARUS_LERP2START as i32 => "LERP2START",
        x if x == gameExport_t::GAME_ICARUS_LERP2END as i32 => "LERP2END",
        x if x == gameExport_t::GAME_ICARUS_USE as i32 => "USE",
        x if x == gameExport_t::GAME_ICARUS_KILL as i32 => "KILL",
        x if x == gameExport_t::GAME_ICARUS_REMOVE as i32 => "REMOVE",
        x if x == gameExport_t::GAME_ICARUS_PLAY as i32 => "PLAY",
        x if x == gameExport_t::GAME_ICARUS_GETFLOAT as i32 => "GETFLOAT",
        x if x == gameExport_t::GAME_ICARUS_GETVECTOR as i32 => "GETVECTOR",
        x if x == gameExport_t::GAME_ICARUS_GETSTRING as i32 => "GETSTRING",
        x if x == gameExport_t::GAME_ICARUS_SOUNDINDEX as i32 => "SOUNDINDEX",
        x if x == gameExport_t::GAME_ICARUS_GETSETIDFORSTRING as i32 => "GETSETIDFORSTRING",
        _ => "?",
    }
}

/// Reproduce `dump_endtoend.cpp`'s variable dump (sorted float/string/vector).
fn dump_e2e_vars(icarus: &Icarus, out: &mut String) {
    let mut floats: Vec<_> = icarus.var_floats.iter().collect();
    floats.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in floats {
        writeln!(out, "  F |{}|={:.3}", k, v).unwrap();
    }
    let mut strings: Vec<_> = icarus.var_strings.iter().collect();
    strings.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in strings {
        writeln!(out, "  S |{}|=|{}|", k, v).unwrap();
    }
    let mut vectors: Vec<_> = icarus.var_vectors.iter().collect();
    vectors.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in vectors {
        writeln!(out, "  V |{}|=|{}|", k, v).unwrap();
    }
}

#[test]
fn endtoend_matches_oracle_golden() {
    let root = oracle_root();
    let golden = std::fs::read_to_string(root.join("goldens").join("endtoend_e2e.txt"))
        .expect("read endtoend golden");

    let mut host = MockHost::new();

    // ICARUS_RegisterScript reads `name + IBI_EXT` (".IBI"); the driver runs
    // "fixtures/e2e", so the port requests "fixtures/e2e.IBI".
    let ibi = std::fs::read(root.join("fixtures/e2e.IBI")).expect("read e2e.IBI");
    host.files.insert("fixtures/e2e.IBI".to_string(), ibi);

    // Mock entity 0: a valid, unfrozen script user with stable name strings.
    // The CStrings must outlive every icarus call that reads the pointers.
    let classname = CString::new("func_test").unwrap();
    let targetname = CString::new("test1").unwrap();
    let script_targetname = CString::new("test1").unwrap();
    {
        let ent = host.gentity_mut(0);
        ent.s.number = 0;
        ent.r.svFlags = 0; // no SVF_ICARUS_FREEZE
        ent.classname = classname.as_ptr() as *mut c_char;
        ent.targetname = targetname.as_ptr() as *mut c_char;
        ent.script_targetname = script_targetname.as_ptr() as *mut c_char;
    }

    let mut icarus = Icarus::default();
    let mut out = String::new();

    writeln!(out, "== icarus_endtoend fixtures/e2e ==").unwrap();

    icarus_init(&mut icarus, &mut host);
    let ent_ptr = host.gentity(0);
    icarus_init_ent(&mut icarus, &mut host, ent_ptr);
    writeln!(out, "init ok").unwrap();

    host.sv_time = 0;
    let ran = icarus_run_script(&mut icarus, &mut host, ent_ptr, "fixtures/e2e");
    writeln!(out, "runscript ret={}", ran as i32).unwrap();

    // Advance the mock clock and beat the task manager (200ms/frame, 30 frames)
    // — the per-entity heartbeat the engine dispatches (sv_game.cpp:769).
    for _ in 0..30 {
        host.sv_time += 200;
        icarus_maintain_task_manager(&mut icarus, &mut host, 0);
    }

    // The ordered outbound VM_Call(gvm, GAME_ICARUS_*) trace (the golden's core).
    writeln!(out, "-- vm_call trace ({}) --", host.vm_calls.len()).unwrap();
    for (i, (_vm, callnum, _args)) in host.vm_calls.iter().enumerate() {
        writeln!(out, "  {} {}", i, call_name(*callnum)).unwrap();
    }

    writeln!(out, "-- variables --").unwrap();
    dump_e2e_vars(&icarus, &mut out);

    writeln!(out, "-- signals --").unwrap();
    for sig in ["go", "ready"] {
        let v = icarus
            .instance
            .as_ref()
            .map_or(-1, |i| i.check_signal(sig) as i32);
        writeln!(out, "  |{}|={}", sig, v).unwrap();
    }

    writeln!(out, "== end ==").unwrap();

    assert_eq!(out, golden, "end-to-end trace diverges from the C++ oracle");
}
