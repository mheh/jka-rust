//! The C6b REPLAY harness (DEC-48 rulings 1,2,5 + the .4 amendment).
//!
//! Drives ONE cgame module dylib headlessly from a recorded trace and byte-diffs
//! the module's outgoing trap stream against the recording. The recording came
//! from the ORACLE cgame under the live engine, so the recorded module-side
//! stream IS the oracle reference stream (DEC-48 ruling 1). Replaying the oracle
//! dylib against its own recording must be byte-identical (the self-check);
//! replaying the Rust cgame against the same recording gives the verdict.
//!
//! Journal format contract: tools/cgame-referee/README.md 'Journal format'.
//! Shape tables: tools/cgame-referee/{trap,export}-shapes.json, parsed at runtime
//! through the shared shapes.rs (included below via #[path]).

#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::fs::File;
use std::io::{BufReader, Read};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use native_platform::entrypoints::RawVmMain;

// The shared shape tables, included as a file so crates/cgame takes no
// dependency on the shim crate (DEC-48.5). Path is relative to THIS file's dir
// (crates/cgame/tests/replay_support -> up 4 -> repo root).
#[path = "../../../../tools/cgame-referee/shapes.rs"]
pub mod shapes;

use shapes::{
    arg_len, ArgKind, ExportRet, Manifests, SharedKind, TrapShape, ARG_RET_DEREF, ARG_SHARED,
    BLOB_DOUBLE_PTR_SLOT, BLOB_INOUT_BUF, BLOB_IN_BUF, BLOB_IN_STR, BLOB_OUT_BUF, BLOB_OUT_STR,
    CG_SET_SHARED_BUFFER, SHARED_BUFFER_SIZE,
};

// record types (README 'Journal format').
pub const REC_VMCALL_ENTER: u8 = 1;
pub const REC_VMCALL_EXIT: u8 = 2;
pub const REC_SYSCALL_ENTER: u8 = 3;
pub const REC_SYSCALL_EXIT: u8 = 4;
pub const REC_MALFORMED: u8 = 5;
pub const REC_MARKER: u8 = 6;

pub const MAGIC: &[u8; 8] = b"CGSHIMJ1";

/// Cap on stored findings; the replay keeps counting past it.
const MAX_FINDINGS: usize = 200;

// ============================ journal reader ================================

/// One serialized region read back from the journal.
pub struct Blob {
    pub arg_index: u8,
    pub kind: u8,
    pub bytes: Vec<u8>,
}

/// One parsed journal record. Fields present depend on `rec_type`.
pub struct Rec {
    pub rec_type: u8,
    pub seq: u64,
    pub cmd: i64,
    pub ret: i64,
    pub words: Vec<i64>,
    pub blobs: Vec<Blob>,
    pub text: String,
}

impl Rec {
    /// The blob for arg position `idx` of `kind`, if serialized.
    fn blob(&self, idx: u8, kind: u8) -> Option<&Blob> {
        self.blobs
            .iter()
            .find(|b| b.arg_index == idx && b.kind == kind)
    }

    /// Any blob at the sentinel arg index (shared 0xFF, ret-deref 0xFE).
    fn blob_at(&self, idx: u8) -> Option<&Blob> {
        self.blobs.iter().find(|b| b.arg_index == idx)
    }
}

/// A little-endian cursor over one record's payload bytes.
struct Slice<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Slice<'a> {
    fn new(b: &'a [u8]) -> Self {
        Slice { b, pos: 0 }
    }
    fn u8(&mut self) -> u8 {
        let v = self.b[self.pos];
        self.pos += 1;
        v
    }
    fn u16(&mut self) -> u16 {
        let v = u16::from_le_bytes(self.b[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;
        v
    }
    fn u32(&mut self) -> u32 {
        let v = u32::from_le_bytes(self.b[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        v
    }
    fn i64(&mut self) -> i64 {
        let v = i64::from_le_bytes(self.b[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        v
    }
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        let v = self.b[self.pos..self.pos + n].to_vec();
        self.pos += n;
        v
    }
}

/// Streaming journal reader over the single gzip stream. Forward-only with a
/// one-record peek; the drive consumes records in exactly the order the module
/// produces calls, so no random access is needed (keeps memory flat over a
/// 34M-record trace).
pub struct Reader {
    inner: GzDecoder<BufReader<File>>,
    peeked: Option<Rec>,
    pub total: u64,
}

impl Reader {
    pub fn open(path: &Path) -> Result<Reader, String> {
        let f = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
        let mut inner = GzDecoder::new(BufReader::with_capacity(1 << 20, f));
        let mut hdr = [0u8; 12];
        read_exact_or(&mut inner, &mut hdr).map_err(|e| format!("read header: {e}"))?;
        if &hdr[..8] != MAGIC {
            return Err(format!("bad magic {:?}", &hdr[..8]));
        }
        Ok(Reader {
            inner,
            peeked: None,
            total: 0,
        })
    }

    /// Reads one raw record, or None at clean end of stream.
    fn next_raw(&mut self) -> Option<Rec> {
        let mut lenb = [0u8; 4];
        match read_exact_or(&mut self.inner, &mut lenb) {
            Ok(true) => {}
            _ => return None, // clean EOF (or a torn tail from a crashed recording)
        }
        let payload_len = u32::from_le_bytes(lenb) as usize;
        let mut payload = vec![0u8; payload_len];
        if read_exact_or(&mut self.inner, &mut payload).ok() != Some(true) {
            return None;
        }
        self.total += 1;
        Some(parse_record(&payload))
    }

    pub fn peek(&mut self) -> Option<&Rec> {
        if self.peeked.is_none() {
            self.peeked = self.next_raw();
        }
        self.peeked.as_ref()
    }

    pub fn take(&mut self) -> Option<Rec> {
        if let Some(r) = self.peeked.take() {
            return Some(r);
        }
        self.next_raw()
    }
}

/// Reads exactly `buf.len()` bytes; Ok(true) full, Ok(false) clean EOF at the
/// start, Err on a torn read. A crashed recording leaves a torn tail; we treat
/// it as end-of-stream rather than panic (README: never abort on bad data).
fn read_exact_or<R: Read>(r: &mut R, buf: &mut [u8]) -> std::io::Result<bool> {
    let mut filled = 0;
    while filled < buf.len() {
        match r.read(&mut buf[filled..]) {
            // EOF: full only if the buffer was empty; a partial fill is a torn
            // tail from a crashed recording - treat it as clean end.
            Ok(0) => return Ok(filled == buf.len()),
            Ok(n) => filled += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(true)
}

fn parse_record(payload: &[u8]) -> Rec {
    let mut s = Slice::new(payload);
    let rec_type = s.u8();
    let seq = {
        // seq is u64 LE right after rec_type.
        let v = u64::from_le_bytes(payload[1..9].try_into().unwrap());
        s.pos = 9;
        v
    };
    let mut rec = Rec {
        rec_type,
        seq,
        cmd: 0,
        ret: 0,
        words: Vec::new(),
        blobs: Vec::new(),
        text: String::new(),
    };
    match rec_type {
        REC_VMCALL_ENTER | REC_SYSCALL_ENTER => {
            rec.cmd = s.i64();
            read_words(&mut s, &mut rec.words);
            read_blobs(&mut s, &mut rec.blobs);
        }
        REC_VMCALL_EXIT | REC_SYSCALL_EXIT => {
            rec.cmd = s.i64();
            rec.ret = s.i64();
            read_blobs(&mut s, &mut rec.blobs);
        }
        REC_MALFORMED => {
            rec.cmd = s.i64();
            read_words(&mut s, &mut rec.words);
        }
        REC_MARKER => {
            let n = s.u32() as usize;
            rec.text = String::from_utf8_lossy(&s.bytes(n)).into_owned();
        }
        _ => {}
    }
    rec
}

fn read_words(s: &mut Slice, out: &mut Vec<i64>) {
    let count = s.u8() as usize;
    out.reserve(count);
    for _ in 0..count {
        out.push(s.i64());
    }
}

fn read_blobs(s: &mut Slice, out: &mut Vec<Blob>) {
    let count = s.u16() as usize;
    out.reserve(count);
    for _ in 0..count {
        let arg_index = s.u8();
        let kind = s.u8();
        let len = s.u32() as usize;
        let bytes = s.bytes(len);
        out.push(Blob {
            arg_index,
            kind,
            bytes,
        });
    }
}

// ============================ findings ======================================

/// One byte-level divergence between the module and the recording.
pub struct Finding {
    pub seq: u64,
    pub name: String,
    pub what: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "seq {} {} - {}", self.seq, self.name, self.what)
    }
}

/// Byte ranges that are comparable for structs whose remaining bytes are
/// UNINITIALIZED caller stack (the engine reads them per a flags field, the
/// module never zeroes them, so they differ run to run and can never be part
/// of a byte-identical bar).
///
/// CG_FX_ADDPRIMITIVE (279) effectTrailArgStruct_t (q_shared.h:2615-2620):
/// mVerts[4] at 84-byte stride - only each vert's origin (first 12 bytes) is
/// unconditionally meaningful; rgb/alpha/ST fields are mSetFlags-gated. The
/// tail (mShader/mSetFlags/mKillTime, bytes 336..348) always compares.
fn masked_ranges(num: i64, arg: usize) -> Option<&'static [(usize, usize)]> {
    const ADDPRIMITIVE: &[(usize, usize)] =
        &[(0, 12), (84, 96), (168, 180), (252, 264), (336, 348)];
    match (num, arg) {
        (279, 0) => Some(ADDPRIMITIVE),
        _ => None,
    }
}

/// Diff only the masked ranges of two equal-length buffers.
fn diff_masked(module: &[u8], recorded: &[u8], ranges: &[(usize, usize)]) -> Option<String> {
    for &(a, b) in ranges {
        let b = b.min(module.len()).min(recorded.len());
        if a >= b {
            continue;
        }
        if module[a..b] != recorded[a..b] {
            let off = (a..b).find(|&i| module[i] != recorded[i]).unwrap();
            return Some(format!(
                "masked range {a}..{b}, first diff @{off}: mod[{:02x}] rec[{:02x}]",
                module[off], recorded[off]
            ));
        }
    }
    None
}

/// First differing offset + short hex context on each side, or None if equal.
fn diff_bytes(module: &[u8], recorded: &[u8]) -> Option<String> {
    if module == recorded {
        return None;
    }
    let n = module.len().min(recorded.len());
    let off = (0..n).find(|&i| module[i] != recorded[i]).unwrap_or(n);
    let ctx = |b: &[u8]| {
        let end = (off + 16).min(b.len());
        let hex: Vec<String> = b[off..end].iter().map(|x| format!("{x:02x}")).collect();
        hex.join(" ")
    };
    Some(format!(
        "len mod={} rec={}, first diff @{off}: mod[{}] rec[{}]",
        module.len(),
        recorded.len(),
        ctx(module),
        ctx(recorded)
    ))
}

// ============================ replay state ==================================

/// Drives one module dylib from the recording and collects findings. All fields
/// are interior-mutable so the syscall handler reaches the state through a shared
/// `&ReplayState` reconstructed from `ctx` - the reentrant trap->vmMain->trap
/// chain would alias a `&mut`, so we never form one (shared refs may alias).
pub struct ReplayState {
    manifests: Manifests,
    reader: RefCell<Reader>,
    module_vm: RawVmMain,
    /// The 2048-byte region the module registered via CG_SET_SHARED_BUFFER.
    module_shared_ptr: Cell<usize>,
    findings: RefCell<Vec<Finding>>,
    finding_total: Cell<u64>,
    syscall_count: Cell<u64>,
    vmcall_count: Cell<u64>,
    desync: Cell<bool>,
    desync_msg: RefCell<Option<String>>,
}

/// Small Copy summary of the next meaningful record (peek without holding a
/// borrow across the reentrant drive).
#[derive(Clone, Copy)]
struct PeekInfo {
    rec_type: u8,
    seq: u64,
    cmd: i64,
}

/// The outcome the tests assert on.
pub struct RunOutcome {
    pub syscalls: u64,
    pub vmcalls: u64,
    pub records: u64,
    pub finding_total: u64,
    pub findings: Vec<Finding>,
    pub desync: Option<String>,
}

impl ReplayState {
    pub fn new(manifests: Manifests, reader: Reader, module_vm: RawVmMain) -> Box<ReplayState> {
        Box::new(ReplayState {
            manifests,
            reader: RefCell::new(reader),
            module_vm,
            module_shared_ptr: Cell::new(0),
            findings: RefCell::new(Vec::new()),
            finding_total: Cell::new(0),
            syscall_count: Cell::new(0),
            vmcall_count: Cell::new(0),
            desync: Cell::new(false),
            desync_msg: RefCell::new(None),
        })
    }

    // ---- record cursor (skips markers + malformed, which carry no drive
    // meaning; a malformed always trails its own ENTER) -----------------------

    fn peek_meaningful(&self) -> Option<PeekInfo> {
        loop {
            let mut r = self.reader.borrow_mut();
            let rt = match r.peek() {
                Some(rec) => rec.rec_type,
                None => return None,
            };
            if rt == REC_MARKER || rt == REC_MALFORMED {
                r.take();
                continue;
            }
            let rec = r.peek().unwrap();
            return Some(PeekInfo {
                rec_type: rec.rec_type,
                seq: rec.seq,
                cmd: rec.cmd,
            });
        }
    }

    fn take_meaningful(&self) -> Option<Rec> {
        // peek_meaningful has already discarded any leading marker/malformed.
        self.peek_meaningful();
        self.reader.borrow_mut().take()
    }

    fn record_total(&self) -> u64 {
        self.reader.borrow().total
    }

    // ---- findings + desync --------------------------------------------------

    fn finding(&self, seq: u64, name: &str, what: String) {
        self.finding_total.set(self.finding_total.get() + 1);
        let mut v = self.findings.borrow_mut();
        if v.len() < MAX_FINDINGS {
            v.push(Finding {
                seq,
                name: name.to_string(),
                what,
            });
        }
    }

    /// Hard desync: mark it and unwind out of the module (extern "C-unwind"
    /// lets the panic traverse the module's live C frames back to the top-level
    /// catch in `run`). A different trap number than recorded is the only thing
    /// that fires this.
    fn desync(&self, msg: String) -> ! {
        self.desync.set(true);
        *self.desync_msg.borrow_mut() = Some(msg.clone());
        panic!("cgame-replay hard desync: {msg}");
    }

    // ---- the top-level drive ------------------------------------------------

    pub fn run(&self) -> RunOutcome {
        loop {
            if self.desync.get() {
                break;
            }
            let info = match self.peek_meaningful() {
                Some(i) => i,
                None => break, // clean end of recording
            };
            if info.rec_type != REC_VMCALL_ENTER {
                self.finding(
                    info.seq,
                    "top-level",
                    format!(
                        "expected VMCALL_ENTER, recording had rec_type {}",
                        info.rec_type
                    ),
                );
                // not a trap desync; drop the stray record and continue.
                self.reader.borrow_mut().take();
                continue;
            }
            let caught = std::panic::catch_unwind(AssertUnwindSafe(|| self.drive_vmcall()));
            if caught.is_err() && !self.desync.get() {
                // a non-desync panic (module fault / harness bug) - stop cleanly.
                self.desync.set(true);
                *self.desync_msg.borrow_mut() =
                    Some("panic during vmMain drive (not a trap-number desync)".to_string());
            }
            if self.desync.get() {
                break;
            }
        }
        let msg = self.desync_msg.borrow().clone();
        RunOutcome {
            syscalls: self.syscall_count.get(),
            vmcalls: self.vmcall_count.get(),
            records: self.record_total(),
            finding_total: self.finding_total.get(),
            findings: std::mem::take(&mut self.findings.borrow_mut()),
            desync: msg,
        }
    }

    /// Drive one VMCALL bracket: consume its ENTER, call the module's vmMain with
    /// the recorded words (substituting live buffers for pointer args and priming
    /// the shared region), then consume + diff its EXIT.
    fn drive_vmcall(&self) {
        let enter = self.take_meaningful().expect("VMCALL_ENTER present");
        self.vmcall_count.set(self.vmcall_count.get() + 1);
        let cmd = enter.cmd;
        let shape = self.manifests.export(cmd);

        // build the 12 arg words; keep any substituted buffers alive across the
        // call, and remember out/inout ones so the EXIT can diff them.
        let mut words: [isize; 12] = [0; 12];
        for i in 0..12 {
            words[i] = enter.words.get(i).copied().unwrap_or(0) as isize;
        }
        // owned buffers: (arg_index, buffer, expected_len) for EXIT diffing.
        let mut out_bufs: Vec<(usize, Vec<u8>)> = Vec::new();
        // hold in-buffers alive until after the call.
        let mut _live: Vec<Vec<u8>> = Vec::new();

        if let Some(sh) = shape {
            for (i, a) in sh.args.iter().enumerate() {
                if i >= 12 {
                    break;
                }
                match a.kind {
                    ArgKind::InStr => {
                        let mut buf = enter
                            .blob(i as u8, BLOB_IN_STR)
                            .map(|b| b.bytes.clone())
                            .unwrap_or_default();
                        buf.push(0);
                        words[i] = buf.as_ptr() as isize;
                        _live.push(buf);
                    }
                    ArgKind::InBuf => {
                        let buf = enter
                            .blob(i as u8, BLOB_IN_BUF)
                            .map(|b| b.bytes.clone())
                            .unwrap_or_else(|| vec![0u8; a.size_of as usize]);
                        words[i] = buf.as_ptr() as isize;
                        _live.push(buf);
                    }
                    ArgKind::OutBuf => {
                        let buf = vec![0u8; a.size_of as usize];
                        out_bufs.push((i, buf));
                        words[i] = out_bufs.last().unwrap().1.as_ptr() as isize;
                    }
                    ArgKind::InoutBuf => {
                        let buf = enter
                            .blob(i as u8, BLOB_INOUT_BUF)
                            .map(|b| b.bytes.clone())
                            .unwrap_or_else(|| vec![0u8; a.size_of as usize]);
                        out_bufs.push((i, buf));
                        words[i] = out_bufs.last().unwrap().1.as_ptr() as isize;
                    }
                    _ => {} // scalar - the recorded word already stands
                }
            }
            // prime the shared region for in/inout arms.
            if matches!(sh.shared, SharedKind::In | SharedKind::Inout) {
                if let Some(b) = enter.blob_at(ARG_SHARED) {
                    self.write_shared(&b.bytes);
                }
            }
        }

        // call the module - traps re-enter this state through the handler.
        let vm = self.module_vm;
        let ret = vm(
            cmd as i32, words[0], words[1], words[2], words[3], words[4], words[5], words[6],
            words[7], words[8], words[9], words[10], words[11],
        );
        if self.desync.get() {
            return;
        }

        // consume the matching EXIT (skipping any recorded inner syscalls the
        // module did not make).
        let exit = match self.expect_vmcall_exit(cmd) {
            Some(e) => e,
            None => return,
        };

        // diff the return word by ret kind.
        if let Some(sh) = shape {
            match sh.ret {
                ExportRet::Scalar | ExportRet::Void => {
                    if ret as i64 != exit.ret {
                        self.finding(
                            exit.seq,
                            &sh.name,
                            format!("ret word: module {ret} != recorded {}", exit.ret),
                        );
                    }
                }
                ExportRet::PtrOpaque => {
                    // host-specific token: only presence/nullness is comparable.
                    if (ret == 0) != (exit.ret == 0) {
                        self.finding(
                            exit.seq,
                            &sh.name,
                            format!("ret ptr nullness: module {ret} vs recorded {}", exit.ret),
                        );
                    }
                }
                ExportRet::PtrDeref => {
                    // diff the pointed-to bytes, not the (host-specific) pointer.
                    // KNOWN GAP: RoffSystem writes through this trajectory_t*
                    // AFTER the vmcall returns; the shim/replay cannot see that
                    // engine write, so ROFF fixtures diff here by design
                    // (README 'The ROFF SetLerp gap').
                    if let Some(b) = exit.blob_at(ARG_RET_DEREF) {
                        let got = read_module(ret as i64, sh.ret_size_of as usize);
                        if let Some(d) = diff_bytes(&got, &b.bytes) {
                            self.finding(exit.seq, &sh.name, format!("ret deref: {d}"));
                        }
                    }
                }
            }

            // diff out/inout arg buffers the module wrote.
            for (i, buf) in &out_bufs {
                let kind = if sh.args[*i].kind == ArgKind::OutBuf {
                    BLOB_OUT_BUF
                } else {
                    BLOB_INOUT_BUF
                };
                if let Some(b) = exit.blob(*i as u8, kind) {
                    if let Some(d) = diff_bytes(buf, &b.bytes) {
                        self.finding(exit.seq, &sh.name, format!("arg{i} out buffer: {d}"));
                    }
                }
            }

            // diff the shared region for out/inout arms.
            if matches!(sh.shared, SharedKind::Out | SharedKind::Inout) {
                if let Some(b) = exit.blob_at(ARG_SHARED) {
                    let got = self.read_shared();
                    if let Some(d) = diff_bytes(&got, &b.bytes) {
                        self.finding(exit.seq, &sh.name, format!("shared buffer: {d}"));
                    }
                }
            }
        }
    }

    /// After vmMain returns, consume forward to this bracket's VMCALL_EXIT. Any
    /// recorded inner SYSCALL/VMCALL the module skipped is a finding + skip.
    fn expect_vmcall_exit(&self, cmd: i64) -> Option<Rec> {
        loop {
            let info = self.peek_meaningful()?;
            match info.rec_type {
                REC_VMCALL_EXIT => {
                    let e = self.take_meaningful().unwrap();
                    if e.cmd != cmd {
                        self.finding(
                            e.seq,
                            "vmcall",
                            format!("EXIT cmd {} != ENTER cmd {cmd}", e.cmd),
                        );
                    }
                    return Some(e);
                }
                REC_SYSCALL_ENTER => {
                    self.finding(
                        info.seq,
                        "vmcall",
                        format!("recorded trap {} not issued by module", info.cmd),
                    );
                    self.skip_syscall_bracket();
                }
                REC_VMCALL_ENTER => {
                    self.finding(
                        info.seq,
                        "vmcall",
                        format!("recorded nested vmcall {} not issued by module", info.cmd),
                    );
                    self.skip_vmcall_bracket();
                }
                other => {
                    self.finding(
                        info.seq,
                        "vmcall",
                        format!("unexpected rec_type {other} awaiting EXIT"),
                    );
                    self.reader.borrow_mut().take();
                }
            }
        }
    }

    /// Serve one module syscall: match it against the recorded SYSCALL_ENTER,
    /// drive any nested vmcalls, then serve the recorded EXIT (out-blobs written
    /// back through the module's pointers, recorded ret returned).
    fn serve_syscall(&self, args_ptr: *const isize) -> isize {
        if self.desync.get() {
            return 0;
        }
        self.syscall_count.set(self.syscall_count.get() + 1);

        // SAFETY: the trampoline always hands us its full 16-word frame.
        let frame: [i64; 16] = {
            let raw = unsafe { std::slice::from_raw_parts(args_ptr, 16) };
            let mut f = [0i64; 16];
            for i in 0..16 {
                f[i] = raw[i] as i64;
            }
            f
        };
        let number = frame[0];
        if std::env::var_os("JKA_REPLAY_TRACE").is_some() {
            eprintln!("trap {number} seq~{}", self.syscall_count.get());
        }

        // register the module's shared region (host-owned; re-pointed each run).
        if number == CG_SET_SHARED_BUFFER {
            self.module_shared_ptr.set(frame[1] as usize);
        }

        // the next meaningful record must be this trap's SYSCALL_ENTER.
        let info = match self.peek_meaningful() {
            Some(i) => i,
            None => self.desync(format!(
                "module issued trap {number} but the recording ended"
            )),
        };
        if info.rec_type != REC_SYSCALL_ENTER {
            // the module made an extra syscall the recording did not have here
            // (the bracket was about to close). Report and serve a safe 0 without
            // consuming, so the outer drive still finds its structure.
            self.finding(
                info.seq,
                "syscall",
                format!(
                    "module issued extra trap {number}; recording had rec_type {} next",
                    info.rec_type
                ),
            );
            return 0;
        }
        if info.cmd != number {
            // DIFFERENT TRAP NUMBER = hard desync. If the module bailed with
            // CG_ERROR (trap 1), its message tells us exactly what it choked on.
            let extra = if number == 1 {
                let msg = String::from_utf8_lossy(&read_module_cstr(frame[1])).into_owned();
                format!(" [module CG_ERROR: {msg}]")
            } else {
                String::new()
            };
            self.desync(format!(
                "trap number: module issued {number}, recording expected {} (seq {}){extra}",
                info.cmd, info.seq
            ));
        }
        let enter = self.take_meaningful().unwrap();
        let shape = self.manifests.trap(number);

        // compare the module's call against the recorded ENTER.
        if let Some(sh) = shape {
            self.compare_syscall_enter(sh, &enter, &frame);
        }

        // drive nested vmcalls recorded inside this trap's bracket (e.g.
        // trap_UpdateScreen re-enters CG_DRAW_ACTIVE_FRAME, cl_scrn.cpp:439-442),
        // then take the matching SYSCALL_EXIT.
        let exit = loop {
            if self.desync.get() {
                return 0;
            }
            let p = match self.peek_meaningful() {
                Some(p) => p,
                None => self.desync(format!(
                    "trap {number} bracket never closed (recording ended)"
                )),
            };
            match p.rec_type {
                REC_VMCALL_ENTER => self.drive_vmcall(),
                REC_SYSCALL_EXIT => break self.take_meaningful().unwrap(),
                REC_SYSCALL_ENTER => {
                    // a sibling trap with no closing EXIT for ours first - the
                    // module made fewer inner calls; report + skip that bracket.
                    self.finding(
                        p.seq,
                        "syscall",
                        format!(
                            "recorded inner trap {} not issued before trap {number} closed",
                            p.cmd
                        ),
                    );
                    self.skip_syscall_bracket();
                }
                other => self.desync(format!(
                    "awaiting SYSCALL_EXIT for trap {number}, got rec_type {other}"
                )),
            }
        };

        // serve the recorded response back through the module's pointers.
        if let Some(sh) = shape {
            self.serve_syscall_exit(sh, &exit, &frame);
        }
        exit.ret as isize
    }

    /// Compare a trap's scalar words + in-blobs against the recording. Pointer
    /// words are ASLR-dependent and never compared - the pointee blob is the
    /// deterministic witness.
    fn compare_syscall_enter(&self, sh: &TrapShape, enter: &Rec, frame: &[i64; 16]) {
        // scalar arg words (arg i is frame[i+1]; recorded words[i+1]). The cgame
        // VM args are 32-bit int (floats ride as their int bits); the variadic
        // trampoline grabs 64-bit words, so the HIGH 32 bits are stack garbage
        // that differs run-to-run - compare only the low 32. A scalar typed `*`
        // is a 64-bit host pointer token (ghoul2 handle), host-specific, so we
        // never compare its word.
        for (i, a) in sh.args.iter().enumerate() {
            if a.kind != ArgKind::Scalar || a.ty.ends_with('*') {
                continue;
            }
            if let Some(rw) = enter.words.get(i + 1).copied() {
                if rw as i32 != frame[i + 1] as i32 {
                    self.finding(
                        enter.seq,
                        &sh.name,
                        format!(
                            "arg{i} word: module {} != recorded {}",
                            frame[i + 1] as i32,
                            rw as i32
                        ),
                    );
                }
            }
        }
        // in-blobs: read the module's pointee and byte-compare.
        for (i, a) in sh.args.iter().enumerate() {
            let ptr = frame[i + 1];
            match a.kind {
                ArgKind::InStr => {
                    let got = read_module_cstr(ptr);
                    if let Some(b) = enter.blob(i as u8, BLOB_IN_STR) {
                        if let Some(d) = diff_bytes(&got, &b.bytes) {
                            self.finding(enter.seq, &sh.name, format!("arg{i} in_str: {d}"));
                        }
                    }
                }
                ArgKind::InBuf => {
                    let len = arg_len(a, sh.num, i, frame) as usize;
                    let got = read_module(ptr, len);
                    if let Some(b) = enter.blob(i as u8, BLOB_IN_BUF) {
                        let d = if let Some(ranges) = masked_ranges(sh.num, i) {
                            diff_masked(&got, &b.bytes, ranges)
                        } else {
                            diff_bytes(&got, &b.bytes)
                        };
                        if let Some(d) = d {
                            self.finding(enter.seq, &sh.name, format!("arg{i} in_buf: {d}"));
                        }
                    }
                }
                ArgKind::InoutBuf => {
                    let len = arg_len(a, sh.num, i, frame) as usize;
                    let got = read_module(ptr, len);
                    if let Some(b) = enter.blob(i as u8, BLOB_INOUT_BUF) {
                        if let Some(d) = diff_bytes(&got, &b.bytes) {
                            self.finding(enter.seq, &sh.name, format!("arg{i} inout_buf(in): {d}"));
                        }
                    }
                }
                // double_ptr slot contents are host pointer tokens - never part
                // of the diff bar (we serve recorded tokens back, so a compare is
                // a tautology; a mis-shaped word here would deref a token and
                // fault, which is how the ANGLEOVERRIDE/CLEANMODELS manifest
                // numbering swap was caught).
                ArgKind::DoublePtr => {}
                _ => {}
            }
        }
    }

    /// Write the recorded engine-side results back through the module's pointers:
    /// out/inout buffers, and the host token into double_ptr slots. The token is
    /// host-specific but writing the recorded one keeps the module's later
    /// handle-by-value traps consistent with the recording.
    fn serve_syscall_exit(&self, sh: &TrapShape, exit: &Rec, frame: &[i64; 16]) {
        for (i, a) in sh.args.iter().enumerate() {
            let ptr = frame[i + 1];
            match a.kind {
                ArgKind::OutBuf => {
                    if let Some(b) = exit.blob(i as u8, BLOB_OUT_BUF) {
                        write_module(ptr, &b.bytes);
                    }
                }
                ArgKind::InoutBuf => {
                    if let Some(b) = exit.blob(i as u8, BLOB_INOUT_BUF) {
                        write_module(ptr, &b.bytes);
                    }
                }
                ArgKind::OutStr => {
                    if let Some(b) = exit.blob(i as u8, BLOB_OUT_STR) {
                        let mut z = b.bytes.clone();
                        z.push(0);
                        write_module(ptr, &z);
                    }
                }
                ArgKind::DoublePtr => {
                    if let Some(b) = exit.blob(i as u8, BLOB_DOUBLE_PTR_SLOT) {
                        write_module(ptr, &b.bytes);
                    }
                }
                _ => {}
            }
        }
    }

    // ---- shared region + bracket skipping ----------------------------------

    fn write_shared(&self, bytes: &[u8]) {
        let p = self.module_shared_ptr.get();
        if p == 0 {
            return;
        }
        let n = bytes.len().min(SHARED_BUFFER_SIZE);
        // SAFETY: the module registered this 2048-byte region; we only ever
        // touch its declared length.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p as *mut u8, n);
        }
    }

    fn read_shared(&self) -> Vec<u8> {
        let p = self.module_shared_ptr.get();
        if p == 0 {
            return Vec::new();
        }
        // SAFETY: same registered region.
        unsafe { std::slice::from_raw_parts(p as *const u8, SHARED_BUFFER_SIZE).to_vec() }
    }

    /// Consume a full SYSCALL bracket (its ENTER..EXIT, including nested vmcalls)
    /// without driving the module - used when the module skipped a recorded call.
    fn skip_syscall_bracket(&self) {
        let enter = match self.take_meaningful() {
            Some(e) => e,
            None => return,
        };
        let want = enter.seq;
        loop {
            let p = match self.peek_meaningful() {
                Some(p) => p,
                None => return,
            };
            match p.rec_type {
                REC_SYSCALL_EXIT if p.seq == want => {
                    self.reader.borrow_mut().take();
                    return;
                }
                REC_VMCALL_ENTER => self.skip_vmcall_bracket(),
                REC_SYSCALL_ENTER => self.skip_syscall_bracket(),
                _ => {
                    self.reader.borrow_mut().take();
                }
            }
        }
    }

    /// Consume a full VMCALL bracket without driving the module.
    fn skip_vmcall_bracket(&self) {
        let enter = match self.take_meaningful() {
            Some(e) => e,
            None => return,
        };
        let want = enter.seq;
        loop {
            let p = match self.peek_meaningful() {
                Some(p) => p,
                None => return,
            };
            match p.rec_type {
                REC_VMCALL_EXIT if p.seq == want => {
                    self.reader.borrow_mut().take();
                    return;
                }
                REC_VMCALL_ENTER => self.skip_vmcall_bracket(),
                REC_SYSCALL_ENTER => self.skip_syscall_bracket(),
                _ => {
                    self.reader.borrow_mut().take();
                }
            }
        }
    }
}

// ============================ foreign memory ================================

/// Cap on any single module read - a garbage length never makes us try to read
/// (or allocate) gigabytes. Mirrors the recorder's MAX_BLOB.
const MAX_READ: usize = 1 << 20;

// Mach-checked copies. The kernel validates the address range, so a quirk
// pointer from the module (a record-time host token passed as an address, the
// CleanModels value-passing site) returns an error instead of SIGBUS.
extern "C" {
    fn mach_task_self() -> u32;
    fn vm_read_overwrite(
        task: u32,
        addr: usize,
        size: usize,
        data: usize,
        outsize: *mut usize,
    ) -> i32;
    fn vm_write(task: u32, addr: usize, data: usize, count: u32) -> i32;
}

/// Reads `len` bytes of module memory at `ptr`. Empty on null, absurd length,
/// or an unmapped address (kernel-checked, never faults).
fn read_module(ptr: i64, len: usize) -> Vec<u8> {
    if ptr == 0 || len == 0 || len > MAX_READ {
        return Vec::new();
    }
    let mut buf = vec![0u8; len];
    let mut got: usize = 0;
    // SAFETY: kernel-validated copy into our owned buffer.
    let kr = unsafe {
        vm_read_overwrite(
            mach_task_self(),
            ptr as usize,
            len,
            buf.as_mut_ptr() as usize,
            &mut got,
        )
    };
    if kr != 0 || got != len {
        return Vec::new();
    }
    buf
}

/// Reads a NUL-terminated C string of module memory at `ptr`, capped. Walks in
/// kernel-checked pages so an unterminated or unmapped string never faults.
fn read_module_cstr(ptr: i64) -> Vec<u8> {
    let mut out = Vec::new();
    if ptr == 0 {
        return out;
    }
    let mut at = ptr as usize;
    while out.len() < MAX_READ {
        // read up to the end of the current page, then continue page by page
        let page_left = 4096 - (at & 4095);
        let chunk = read_module(at as i64, page_left);
        if chunk.is_empty() {
            break; // unmapped - stop at what we have
        }
        if let Some(z) = chunk.iter().position(|&b| b == 0) {
            out.extend_from_slice(&chunk[..z]);
            return out;
        }
        out.extend_from_slice(&chunk);
        at += page_left;
    }
    out
}

/// Writes recorded bytes back through the module's pointer. No-op on null or an
/// unmapped address (kernel-checked, never faults).
fn write_module(ptr: i64, bytes: &[u8]) {
    if ptr == 0 || bytes.is_empty() {
        return;
    }
    // SAFETY: kernel-validated copy from our owned buffer.
    unsafe {
        vm_write(
            mach_task_self(),
            ptr as usize,
            bytes.as_ptr() as usize,
            bytes.len() as u32,
        );
    }
}

// ============================ handler + entry ==============================

/// The armed engine-slot handler. `ctx` is our `*const ReplayState`; the
/// reentrant chain reconstructs a fresh shared `&ReplayState` each time (never a
/// `&mut`, so no aliasing), and a hard-desync panic unwinds through the module's
/// C-unwind frames to the top-level catch in `run`.
pub extern "C-unwind" fn replay_syscall(ctx: *mut c_void, args: *const isize) -> isize {
    // SAFETY: ctx is the Box<ReplayState> the driver armed; it outlives every
    // vmMain call. Shared ref only.
    let state = unsafe { &*(ctx as *const ReplayState) };
    state.serve_syscall(args)
}

/// Locate the manifests directory (tools/cgame-referee) from the crate manifest.
pub fn referee_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/cgame; the referee lives at ../../tools/...
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("tools/cgame-referee")
}
