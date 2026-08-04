//! The demo-referee probe module, shared by both engines (ticket gh#30).
//!
//! # Why one source file serves two hosts
//! The gate compares two journals: one written while our client engine plays a
//! demo, one written while the oracle engine plays the same demo. The two
//! journals are only comparable when the SAME module sits at the seam, so this
//! file is the single probe body. The standalone cdylib
//! (`tools/cgame-referee/probe/src/lib.rs`) loads it into the oracle engine, and
//! `crates/mp/engine/core/tests/demo_referee.rs` includes it by `#[path]` for
//! the in-process run. Each host supplies a `TrapFn` and the crate-root `shapes`
//! and `journal` modules.
//!
//! # The snapshot bracket
//! Frame cadence is host-clock state, not demo state. The oracle engine draws at
//! its own frame rate and our rig steps a fixed clock, so a per-frame journal
//! would never align. The probe therefore journals per SNAPSHOT: it reads the
//! current snapshot number on every `CG_DRAW_ACTIVE_FRAME` without journaling
//! it, and it opens a journal bracket only when that number moved. One bracket
//! holds the snapshot read, the reliable-command drain, and their blobs, so the
//! record stream is a pure function of the demo bytes on both engines.
//!
//! # What the probe never does
//! The probe answers `CG_INIT`, `CG_DRAW_ACTIVE_FRAME` and `CG_SHUTDOWN` only.
//! Every other `vmMain` arm returns 0 with no record, because the oracle engine
//! drives arms (console command, key event, crosshair player) that our headless
//! rig has no path to. The probe calls no renderer trap and no sound trap, so
//! the run stays headless.

// Two hosts include this file, and each drives a subset of the probe surface, so
// a method with no caller in one host is live in the other.
#![allow(dead_code)]

use std::path::Path;

use crate::journal::{
    BlobKind, BlobSink, Journal, Record, REC_SYSCALL_ENTER, REC_SYSCALL_EXIT, REC_VMCALL_ENTER,
    REC_VMCALL_EXIT,
};
use crate::shapes::{self, ArgKind, ExportRet, Manifests};

// ===========================================================================
// The seam numbers the probe drives
// ===========================================================================

/// `cgameExport_t` arms.
/// Source: `oracle/codemp/cgame/cg_public.h:598-640`
pub const CG_INIT: i64 = 0;
pub const CG_SHUTDOWN: i64 = 1;
pub const CG_DRAW_ACTIVE_FRAME: i64 = 3;

/// `cgameImport_t` traps.
/// Source: `oracle/codemp/cgame/cg_public.h:66-592`
pub const CG_ARGC: i64 = 10;
pub const CG_ARGV: i64 = 11;
pub const CG_SENDCONSOLECOMMAND: i64 = 18;
pub const CG_GETGAMESTATE: i64 = 228;
pub const CG_GETCURRENTSNAPSHOTNUMBER: i64 = 229;
pub const CG_GETSNAPSHOT: i64 = 230;
pub const CG_GETSERVERCOMMAND: i64 = 232;

/// `sizeof(gameState_t)`, the buffer `CG_GETGAMESTATE` fills.
/// Source: `oracle/codemp/game/q_shared.h:1183-1187`
pub const GAMESTATE_SIZE: usize = 22804;

/// `sizeof(snapshot_t)`, the buffer `CG_GETSNAPSHOT` fills.
/// Source: `oracle/codemp/cgame/cg_public.h:20-36`
pub const SNAPSHOT_SIZE: usize = 139352;

/// `MAX_STRING_CHARS`, the size the probe hands `CG_ARGV`.
/// Source: `oracle/codemp/game/q_shared.h:106`
pub const ARGV_BUF_SIZE: usize = 1024;

/// Journaled brackets one recording keeps (DEC-62.2, the committed-golden
/// bound). One bracket is one distinct snapshot.
pub const DEFAULT_BRACKET_CAP: u32 = 400;

/// Cap on the reliable commands one bracket drains. A demo whose recording
/// started late can name a sequence far ahead of the last executed one, and the
/// bound keeps that from filling the journal.
const MAX_COMMANDS_PER_BRACKET: i32 = 64;

/// Cap on the tokens one drained command reports. `CG_ARGC` is engine state, so
/// a wild count never turns into a wild read loop.
const MAX_ARGS_PER_COMMAND: i32 = 16;

/// Cap on one serialized blob, the same bound `shim/src/serialize.rs` uses.
const MAX_BLOB: u64 = 1 << 20;

/// How the host forwards one 16-word trap frame into its engine.
pub type TrapFn = fn(&mut [isize; 16]) -> isize;

// ===========================================================================
// Foreign-memory reads
// ===========================================================================

/// Reads a NUL-terminated string at `ptr`, capped. Empty on null.
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

// ===========================================================================
// Blob serializers
// ===========================================================================
//
// These mirror `shim/src/serialize.rs` byte for byte, because the recorder shim
// and this probe must produce the same journal.

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

// ===========================================================================
// The probe
// ===========================================================================

/// One recording session: the journal, the shape tables, and the seam state the
/// bracket rule needs.
///
/// The probe owns its scratch buffers and clears each one before the trap that
/// fills it, so a blob never carries a byte from an earlier frame.
pub struct Probe {
    journal: Option<Journal>,
    manifests: Manifests,
    trap: TrapFn,
    seq: u64,
    /// Snapshot number the last journaled bracket read. -1 before the first.
    last_snapshot: i32,
    /// Next reliable command to drain, the `cgs.serverCommandSequence` rule.
    next_command: i32,
    /// True once the first bracket seated `next_command`.
    seeded: bool,
    /// Journaled brackets so far.
    brackets: u32,
    cap: u32,
    /// True once the cap closed the journal.
    done: bool,
    gamestate: Box<[u8; GAMESTATE_SIZE]>,
    snapshot: Box<[u8; SNAPSHOT_SIZE]>,
    argv: [u8; ARGV_BUF_SIZE],
    /// Trap numbers journaled, in order. The rig asserts on these.
    pub traps: Vec<i64>,
    /// vmMain arms journaled, in order.
    pub vmcalls: Vec<i64>,
}

impl Probe {
    /// Opens a recording session. `manifest_dir` holds `trap-shapes.json` and
    /// `export-shapes.json`.
    pub fn new(
        journal_path: &Path,
        manifest_dir: &Path,
        cap: u32,
        trap: TrapFn,
    ) -> Result<Probe, String> {
        let manifests = Manifests::load(manifest_dir)?;
        let journal = Journal::create(journal_path)
            .map_err(|e| format!("create {}: {e}", journal_path.display()))?;
        Ok(Probe {
            journal: Some(journal),
            manifests,
            trap,
            seq: 0,
            last_snapshot: -1,
            next_command: 0,
            seeded: false,
            brackets: 0,
            cap,
            done: false,
            gamestate: Box::new([0u8; GAMESTATE_SIZE]),
            snapshot: Box::new([0u8; SNAPSHOT_SIZE]),
            argv: [0u8; ARGV_BUF_SIZE],
            traps: Vec::new(),
            vmcalls: Vec::new(),
        })
    }

    /// Journaled brackets so far. The rig stops its playback loop on this.
    pub fn brackets(&self) -> u32 {
        self.brackets
    }

    /// True once the bracket cap closed the journal.
    pub fn done(&self) -> bool {
        self.done
    }

    /// Closes the gzip stream. A second call is a no-op, so the cap and
    /// `CG_SHUTDOWN` can both reach it.
    pub fn finish(&mut self) {
        if let Some(j) = self.journal.take() {
            j.finish();
        }
        self.done = true;
    }

    // -- journal writers ----------------------------------------------------

    fn write(&mut self, rec: &Record) {
        if let Some(j) = self.journal.as_mut() {
            j.write(rec);
        }
    }

    /// Forwards one trap and journals both ends of it.
    ///
    /// The probe calls no trap that re-enters `vmMain`, so the host may hold its
    /// probe lock across this forward.
    fn journaled_trap(&mut self, args: &mut [isize; 16]) -> isize {
        let num = args[0] as i64;
        self.seq += 1;
        let seq = self.seq;
        self.traps.push(num);

        let mut rec = Record::new(REC_SYSCALL_ENTER, seq);
        rec.push_i64(num);
        rec.push_words(args);
        trap_enter_blobs(&self.manifests, num, args, &mut rec);
        self.write(&rec);

        let ret = (self.trap)(args);

        let mut rec = Record::new(REC_SYSCALL_EXIT, seq);
        rec.push_i64(num);
        rec.push_i64(ret as i64);
        trap_exit_blobs(&self.manifests, num, args, &mut rec);
        self.write(&rec);
        ret
    }

    /// Forwards one trap without a record. The bracket rule needs the current
    /// snapshot number before it can decide to journal anything.
    fn silent_trap(&mut self, args: &mut [isize; 16]) -> isize {
        (self.trap)(args)
    }

    fn vmcall_enter(&mut self, cmd: i64, words: &[isize; 12]) -> u64 {
        self.seq += 1;
        let seq = self.seq;
        self.vmcalls.push(cmd);
        let mut rec = Record::new(REC_VMCALL_ENTER, seq);
        rec.push_i64(cmd);
        rec.push_words(words);
        export_enter_blobs(&self.manifests, cmd, words, &mut rec);
        self.write(&rec);
        seq
    }

    fn vmcall_exit(&mut self, seq: u64, cmd: i64, words: &[isize; 12], ret: isize) {
        let mut rec = Record::new(REC_VMCALL_EXIT, seq);
        rec.push_i64(cmd);
        rec.push_i64(ret as i64);
        export_exit_blobs(&self.manifests, cmd, words, ret, &mut rec);
        self.write(&rec);
    }

    // -- the vmMain body ----------------------------------------------------

    /// The probe's `vmMain`. Both hosts call this and return its value.
    pub fn vm_main(&mut self, command: i64, words: &[isize; 12]) -> isize {
        match command {
            CG_INIT => self.init(words),
            CG_DRAW_ACTIVE_FRAME => self.draw_active_frame(words),
            CG_SHUTDOWN => {
                self.finish();
                0
            }
            _ => 0,
        }
    }

    /// `CG_INIT(serverMessageNum, serverCommandSequence, clientNum)`. The probe
    /// reads the game state back the way `CG_Init` does.
    ///
    /// The reliable-command cursor is NOT seated from arg 1. The backlog between
    /// `CG_INIT` and the first drawn snapshot depends on how long each engine
    /// took to reach `CA_ACTIVE`, which is host timing. The first bracket seats
    /// the cursor instead, so every journaled drain is a per-snapshot delta.
    fn init(&mut self, words: &[isize; 12]) -> isize {
        if self.done {
            return 0;
        }
        let seq = self.vmcall_enter(CG_INIT, words);
        self.gamestate.fill(0);
        let ptr = self.gamestate.as_mut_ptr() as isize;
        let mut f = frame(CG_GETGAMESTATE, &[ptr]);
        self.journaled_trap(&mut f);
        self.vmcall_exit(seq, CG_INIT, words, 0);
        0
    }

    /// `CG_DRAW_ACTIVE_FRAME(serverTime, stereoView, demoPlayback)`. A frame
    /// whose snapshot number did not move writes no record.
    fn draw_active_frame(&mut self, words: &[isize; 12]) -> isize {
        if self.done {
            return 0;
        }

        // The bracket decision comes first, so the frames the two engines do not
        // share never reach the journal. The number is read again inside the
        // bracket below, which is what puts the trap's out-args on the byte bar.
        let mut number: i32 = 0;
        let mut server_time: i32 = 0;
        let mut f = frame(
            CG_GETCURRENTSNAPSHOTNUMBER,
            &[
                &mut number as *mut i32 as isize,
                &mut server_time as *mut i32 as isize,
            ],
        );
        self.silent_trap(&mut f);
        // A number of 0 means no snapshot has landed yet. The oracle engine
        // draws such a frame during its load screen, and the real cgame skips
        // it too, because `cg.snap` is still NULL there.
        if number <= 0 || number == self.last_snapshot {
            return 0;
        }
        self.last_snapshot = number;

        let seq = self.vmcall_enter(CG_DRAW_ACTIVE_FRAME, words);

        let mut number: i32 = 0;
        let mut server_time: i32 = 0;
        let mut f = frame(
            CG_GETCURRENTSNAPSHOTNUMBER,
            &[
                &mut number as *mut i32 as isize,
                &mut server_time as *mut i32 as isize,
            ],
        );
        self.journaled_trap(&mut f);

        // The truncation seam: `CL_GetSnapshot` copies at most
        // MAX_ENTITIES_IN_SNAPSHOT entities and leaves the rest of this buffer
        // alone, so the probe clears it first and the blob stays history-free.
        self.snapshot.fill(0);
        let ptr = self.snapshot.as_mut_ptr() as isize;
        let mut f = frame(CG_GETSNAPSHOT, &[number as isize, ptr]);
        let got = self.journaled_trap(&mut f);
        if got != 0 {
            let latest = self.snapshot_command_sequence();
            if self.seeded {
                self.drain_server_commands(latest);
            } else {
                // The first bracket only seats the cursor. See `init`.
                self.next_command = latest;
                self.seeded = true;
            }
        }

        self.vmcall_exit(seq, CG_DRAW_ACTIVE_FRAME, words, 0);

        self.brackets += 1;
        if self.brackets >= self.cap {
            self.finish();
        }
        0
    }

    /// `snapshot_t.serverCommandSequence`, read out of the filled buffer.
    /// Source: `oracle/codemp/cgame/cg_public.h:20-36`
    fn snapshot_command_sequence(&self) -> i32 {
        let at = SNAPSHOT_SIZE - 4;
        i32::from_ne_bytes(self.snapshot[at..].try_into().unwrap())
    }

    /// Drains every reliable command the snapshot names, the
    /// `CG_ExecuteNewServerCommands` rule, and journals the tokenized result.
    /// Source: `oracle/codemp/cgame/cg_servercmds.c:1338-1345`
    fn drain_server_commands(&mut self, latest: i32) {
        let mut drained = 0;
        while self.next_command < latest && drained < MAX_COMMANDS_PER_BRACKET {
            self.next_command += 1;
            drained += 1;
            let mut f = frame(CG_GETSERVERCOMMAND, &[self.next_command as isize]);
            if self.journaled_trap(&mut f) == 0 {
                continue;
            }
            let mut f = frame(CG_ARGC, &[]);
            let argc = self.journaled_trap(&mut f) as i32;
            let argc = argc.clamp(0, MAX_ARGS_PER_COMMAND);
            for i in 0..argc {
                self.argv.fill(0);
                let ptr = self.argv.as_mut_ptr() as isize;
                let mut f = frame(CG_ARGV, &[i as isize, ptr, ARGV_BUF_SIZE as isize]);
                self.journaled_trap(&mut f);
            }
        }
    }

    /// Sends a console command with no record. The standalone host uses this to
    /// quit the oracle engine once the cap closed the journal.
    pub fn send_console_command(&mut self, text: &str) {
        let mut owned: Vec<u8> = text.as_bytes().to_vec();
        owned.push(0);
        let mut f = frame(CG_SENDCONSOLECOMMAND, &[owned.as_ptr() as isize]);
        self.silent_trap(&mut f);
    }
}

/// Builds a 16-word trap frame from a trap number and its arguments.
fn frame(num: i64, args: &[isize]) -> [isize; 16] {
    let mut f = [0isize; 16];
    f[0] = num as isize;
    for (i, a) in args.iter().enumerate() {
        f[i + 1] = *a;
    }
    f
}
