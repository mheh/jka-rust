//! Proves the interpose chain works WITHOUT the engine: load the shim's exports
//! directly, point JKA_SHIM_REAL_CGAME at the oracle cgame dylib, install a stub
//! engine syscall (the C `test_engine_syscall`), drive vmMain with an unknown
//! command, and assert (a) the CG_ERROR syscall reached the stub THROUGH the
//! shim's trampoline and (b) the journal holds the expected
//! VMCALL_ENTER / SYSCALL_ENTER / SYSCALL_EXIT / VMCALL_EXIT bracket.
//!
//! The oracle module's default vmMain arm (cg_main.c:354-358) routes
//! CG_Error("vmMain: unknown command %i") back out as a CG_ERROR (=1) syscall
//! and returns -1 - the same drive as tools/cgame-oracle/smoke.c and
//! crates/cgame/tests/abi_smoke.rs.

#![allow(non_snake_case)]

use std::ffi::{c_void, CStr};
use std::path::PathBuf;

use cgamearm64::{dllEntry, vmMain};

extern "C" {
    fn shim_test_engine_syscall_ptr() -> *mut c_void;
    fn shim_test_saw_cg_error() -> i32;
    fn shim_test_last_msg() -> *const i8;
}

const UNKNOWN_COMMAND: i32 = 0x7fff;

// journal record types (mirror src/journal.rs).
const REC_VMCALL_ENTER: u8 = 1;
const REC_VMCALL_EXIT: u8 = 2;
const REC_SYSCALL_ENTER: u8 = 3;
const REC_SYSCALL_EXIT: u8 = 4;
const REC_MARKER: u8 = 6;

fn oracle_dylib() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("../../cgame-oracle/build/liboraclecgame.dylib")
        .canonicalize()
        .expect("oracle cgame dylib present - run tools/cgame-oracle/build.sh first")
}

/// (rec_type, cmd) for every non-marker record, in file order. The journal file
/// is one gzip stream with the CGSHIMJ1 format inside.
fn parse_journal(path: &PathBuf) -> Vec<(u8, i64)> {
    let raw = std::fs::read(path).expect("read journal");
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut flate2::read::GzDecoder::new(&raw[..]), &mut buf)
        .expect("gunzip journal");
    assert_eq!(&buf[..8], b"CGSHIMJ1", "journal magic");
    let mut pos = 12; // magic(8) + version(4)
    let mut out = Vec::new();
    while pos + 4 <= buf.len() {
        let len = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let rec = &buf[pos..pos + len];
        pos += len;
        let rec_type = rec[0];
        // body starts after rec_type(1) + seq(8); markers carry no cmd word.
        if rec_type != REC_MARKER {
            let cmd = i64::from_le_bytes(rec[9..17].try_into().unwrap());
            out.push((rec_type, cmd));
        }
    }
    out
}

#[test]
fn interpose_records_the_unknown_command_bracket() {
    let real = oracle_dylib();
    let journal = std::env::temp_dir().join(format!("cgame-shim-test-{}.bin", std::process::id()));
    std::env::set_var("JKA_SHIM_REAL_CGAME", &real);
    std::env::set_var("JKA_SHIM_JOURNAL", &journal);

    // dllEntry: store engine stub, dlopen the oracle, hand it our trampoline.
    let engine_stub = unsafe { shim_test_engine_syscall_ptr() };
    dllEntry(engine_stub);

    // drive the unknown command through the shim -> oracle -> CG_ERROR -> stub.
    let ret = vmMain(UNKNOWN_COMMAND, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    assert_eq!(ret, -1, "oracle default arm returns -1 (cg_main.c:358)");

    // (a) the CG_ERROR reached the stub through the shim's trampoline.
    assert_eq!(
        unsafe { shim_test_saw_cg_error() },
        1,
        "CG_ERROR must reach the engine stub"
    );
    let msg = unsafe { CStr::from_ptr(shim_test_last_msg()) }.to_string_lossy();
    assert!(
        msg.contains("unknown command"),
        "CG_ERROR message, got: {msg}"
    );

    // flush by driving CG_SHUTDOWN (cgameExport_t = 1). Oracle CG_Shutdown on an
    // uninitialised module is safe here; it just closes files it never opened.
    let _ = vmMain(1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

    // (b) the journal holds the bracket, correctly nested.
    let recs = parse_journal(&journal);
    let cg_error = 1i64; // cgameImport_t CG_ERROR
    let want = [
        (REC_VMCALL_ENTER, UNKNOWN_COMMAND as i64),
        (REC_SYSCALL_ENTER, cg_error),
        (REC_SYSCALL_EXIT, cg_error),
        (REC_VMCALL_EXIT, UNKNOWN_COMMAND as i64),
    ];
    let mut wi = 0;
    for r in &recs {
        if wi < want.len() && *r == want[wi] {
            wi += 1;
        }
    }
    assert_eq!(
        wi,
        want.len(),
        "journal must contain the ordered bracket {want:?}; got records {recs:?}"
    );

    let _ = std::fs::remove_file(&journal);
}
