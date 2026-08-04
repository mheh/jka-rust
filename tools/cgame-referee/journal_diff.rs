//! The C6b journal reader and the demo-referee byte-diff (ticket gh#30).
//!
//! `crates/mp/engine/core/tests/demo_referee.rs` includes this file by `#[path]`
//! so the gate takes no dependency on the recorder shim crate, the DEC-48.5
//! sharing rule that `shapes.rs` already follows.
//!
//! # What the gate compares
//! Two journals written by the same probe module (`probe/src/probe.rs`), one
//! from our client engine and one from the oracle engine, over the same demo.
//! The bar is byte-identical, with the named exclusions below. Every exclusion
//! is host state that no engine derives from the demo.
//!
//! 1. **Pointer arg words.** The probe's own buffer addresses. The blob is the
//!    witness for what the buffer held, so the address itself proves nothing.
//!    This is the replay referee's first exclusion.
//! 2. **Scalar words compare on the low 32 bits.** The trap frame is grabbed as
//!    64-bit words and the high half is caller stack, which the engine dispatch
//!    also throws away.
//! 3. **`CG_DRAW_ACTIVE_FRAME` arg 0.** The render `serverTime` that
//!    `CL_SetCGameTime` interpolates from `cls.realtime`. Host clock, not demo
//!    state.
//! 4. **`snapshot_t.ping`.** `CL_ParseSnapshot` computes it as
//!    `cls.realtime - outPackets[..].p_realtime`. Host clock again, and a demo
//!    sends no packets, so the field is pure wall clock.
//! 5. **The leading brackets each engine drew before the other started.** The
//!    oracle engine consumes one snapshot inside `CL_SetCGameTime` on the way to
//!    `CA_ACTIVE`, so its first drawn snapshot is one later than ours, and the
//!    probe's first bracket only seats the reliable-command cursor. The gate
//!    starts one snapshot past the later of the two first brackets. Nothing is
//!    dropped in the middle: the compared range must be snapshot-consecutive on
//!    both sides.

// The record fields mirror the journal format, and the diff reads the subset it
// compares, so a field with no reader here still documents the wire record.
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::journal::{
    MAGIC, REC_MALFORMED, REC_MARKER, REC_SYSCALL_ENTER, REC_SYSCALL_EXIT, REC_VMCALL_ENTER,
    REC_VMCALL_EXIT,
};
use crate::shapes::{ArgKind, Manifests};

/// One serialized region inside a record.
pub struct Blob {
    pub arg_index: u8,
    pub kind: u8,
    pub bytes: Vec<u8>,
}

/// One parsed C6b record.
pub struct JournalRecord {
    pub rec_type: u8,
    pub seq: u64,
    /// The trap number or the `vmMain` arm. -1 for a marker.
    pub cmd: i64,
    pub ret: Option<i64>,
    pub words: Vec<i64>,
    pub blobs: Vec<Blob>,
    /// Marker text, empty on every other record type.
    pub text: String,
}

/// Reads a gzipped C6b journal into its records.
pub fn read_journal(path: &Path) -> Result<Vec<JournalRecord>, String> {
    let raw = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut buf = Vec::new();
    flate2::read::GzDecoder::new(&raw[..])
        .read_to_end(&mut buf)
        .map_err(|e| format!("{} is not one gzip stream: {e}", path.display()))?;
    if buf.len() < 12 || &buf[0..8] != MAGIC {
        return Err(format!("{}: bad journal magic", path.display()));
    }
    let mut out = Vec::new();
    let mut at = 12usize;
    while at + 4 <= buf.len() {
        let len = u32::from_le_bytes(buf[at..at + 4].try_into().unwrap()) as usize;
        at += 4;
        if at + len > buf.len() {
            return Err(format!("{}: record runs past the end", path.display()));
        }
        out.push(parse_record(&buf[at..at + len])?);
        at += len;
    }
    Ok(out)
}

fn take_i64(body: &[u8], at: &mut usize) -> Result<i64, String> {
    if *at + 8 > body.len() {
        return Err("record body ran short of an i64".to_string());
    }
    let v = i64::from_le_bytes(body[*at..*at + 8].try_into().unwrap());
    *at += 8;
    Ok(v)
}

fn parse_record(body: &[u8]) -> Result<JournalRecord, String> {
    if body.len() < 9 {
        return Err("record body shorter than its header".to_string());
    }
    let rec_type = body[0];
    let seq = u64::from_le_bytes(body[1..9].try_into().unwrap());
    let mut at = 9usize;
    let mut rec = JournalRecord {
        rec_type,
        seq,
        cmd: -1,
        ret: None,
        words: Vec::new(),
        blobs: Vec::new(),
        text: String::new(),
    };

    if rec_type == REC_MARKER {
        if at + 4 > body.len() {
            return Err("marker record ran short".to_string());
        }
        let n = u32::from_le_bytes(body[at..at + 4].try_into().unwrap()) as usize;
        at += 4;
        rec.text = String::from_utf8_lossy(&body[at..(at + n).min(body.len())]).into_owned();
        return Ok(rec);
    }

    rec.cmd = take_i64(body, &mut at)?;
    match rec_type {
        REC_VMCALL_ENTER | REC_SYSCALL_ENTER | REC_MALFORMED => {
            let count = body[at] as usize;
            at += 1;
            for _ in 0..count {
                rec.words.push(take_i64(body, &mut at)?);
            }
        }
        REC_VMCALL_EXIT | REC_SYSCALL_EXIT => {
            rec.ret = Some(take_i64(body, &mut at)?);
        }
        other => return Err(format!("unknown record type {other}")),
    }
    if rec_type == REC_MALFORMED {
        return Ok(rec);
    }

    if at + 2 > body.len() {
        return Err("record ran short of its blob count".to_string());
    }
    let blob_count = u16::from_le_bytes(body[at..at + 2].try_into().unwrap()) as usize;
    at += 2;
    for _ in 0..blob_count {
        if at + 6 > body.len() {
            return Err("blob header ran short".to_string());
        }
        let arg_index = body[at];
        let kind = body[at + 1];
        let n = u32::from_le_bytes(body[at + 2..at + 6].try_into().unwrap()) as usize;
        at += 6;
        if at + n > body.len() {
            return Err("blob body ran short".to_string());
        }
        rec.blobs.push(Blob {
            arg_index,
            kind,
            bytes: body[at..at + n].to_vec(),
        });
        at += n;
    }
    Ok(rec)
}

// ===========================================================================
// The exclusion table
// ===========================================================================

/// One masked byte range inside a blob, keyed by the trap and the arg index.
struct BlobMask {
    trap: i64,
    arg_index: u8,
    at: usize,
    len: usize,
    why: &'static str,
}

/// `snapshot_t.ping` sits at offset 4 and holds `cls.realtime` during demo
/// playback, because a demo sends no packets for the ping loop to date.
/// Source: `oracle/codemp/cgame/cg_public.h:20-36`,
/// `oracle/codemp/client/cl_parse.cpp` `CL_ParseSnapshot`
const BLOB_MASKS: &[BlobMask] = &[BlobMask {
    trap: 230, // CG_GETSNAPSHOT
    arg_index: 1,
    at: 4,
    len: 4,
    why: "snapshot_t.ping is cls.realtime under a demo",
}];

/// `CG_DRAW_ACTIVE_FRAME` arg 0 is the interpolated render time.
/// Source: `oracle/codemp/client/cl_cgame.cpp` `CL_SetCGameTime`
const MASKED_EXPORT_WORD: (i64, usize) = (3, 0);

// ===========================================================================
// The diff
// ===========================================================================

/// One difference between the two journals, in report order.
pub struct Finding {
    pub index: usize,
    pub what: String,
}

/// What one gate run compared.
pub struct DiffReport {
    pub findings: Vec<Finding>,
    /// Brackets compared after the two lists were aligned.
    pub compared: usize,
    /// Leading brackets each side held before the first common snapshot.
    pub skipped_ours: usize,
    pub skipped_golden: usize,
}

/// `CG_DRAW_ACTIVE_FRAME`, the arm that opens one snapshot bracket.
const EXPORT_DRAW_ACTIVE_FRAME: i64 = 3;

/// `CG_GETCURRENTSNAPSHOTNUMBER`, whose out-arg 0 names the bracket.
const TRAP_CURRENT_SNAPSHOT_NUMBER: i64 = 229;

/// One snapshot bracket: where it starts, how long it is, what it read.
struct Bracket {
    snapshot: i32,
    at: usize,
    len: usize,
}

/// Splits a journal into its leading records (the `CG_INIT` bracket) and the
/// per-snapshot brackets that follow.
fn split(records: &[JournalRecord]) -> (usize, Vec<Bracket>) {
    let head = records
        .iter()
        .position(|r| r.rec_type == REC_VMCALL_ENTER && r.cmd == EXPORT_DRAW_ACTIVE_FRAME)
        .unwrap_or(records.len());

    let mut out: Vec<Bracket> = Vec::new();
    let mut at = head;
    while at < records.len() {
        if records[at].rec_type != REC_VMCALL_ENTER || records[at].cmd != EXPORT_DRAW_ACTIVE_FRAME {
            at += 1;
            continue;
        }
        let start = at;
        let mut end = at + 1;
        while end < records.len() {
            let r = &records[end];
            if r.rec_type == REC_VMCALL_EXIT && r.cmd == EXPORT_DRAW_ACTIVE_FRAME {
                break;
            }
            end += 1;
        }
        let mut snapshot = -1;
        for r in &records[start..end.min(records.len())] {
            if r.rec_type == REC_SYSCALL_EXIT && r.cmd == TRAP_CURRENT_SNAPSHOT_NUMBER {
                for b in &r.blobs {
                    if b.arg_index == 0 && b.bytes.len() == 4 {
                        snapshot = i32::from_le_bytes(b.bytes[..].try_into().unwrap());
                    }
                }
            }
        }
        let stop = (end + 1).min(records.len());
        out.push(Bracket {
            snapshot,
            at: start,
            len: stop - start,
        });
        at = stop;
    }
    (head, out)
}

/// The snapshot numbers a bracket list reads, for the density check.
pub fn bracket_snapshots(records: &[JournalRecord]) -> Vec<i32> {
    split(records).1.into_iter().map(|b| b.snapshot).collect()
}

/// Names a record for a finding line.
fn describe(m: &Manifests, rec: &JournalRecord) -> String {
    let kind = match rec.rec_type {
        REC_VMCALL_ENTER => "VMCALL_ENTER",
        REC_VMCALL_EXIT => "VMCALL_EXIT",
        REC_SYSCALL_ENTER => "SYSCALL_ENTER",
        REC_SYSCALL_EXIT => "SYSCALL_EXIT",
        REC_MALFORMED => "MALFORMED",
        REC_MARKER => "MARKER",
        _ => "UNKNOWN",
    };
    let name = match rec.rec_type {
        REC_VMCALL_ENTER | REC_VMCALL_EXIT => m
            .export(rec.cmd)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| format!("export {}", rec.cmd)),
        REC_SYSCALL_ENTER | REC_SYSCALL_EXIT => m
            .trap(rec.cmd)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| format!("trap {}", rec.cmd)),
        _ => String::new(),
    };
    format!("{kind} {name}")
}

/// True when this trap arg word is a host pointer rather than a value.
fn word_is_pointer(m: &Manifests, trap: i64, arg: usize) -> bool {
    let Some(shape) = m.trap(trap) else {
        return true;
    };
    let Some(a) = shape.args.get(arg) else {
        return false;
    };
    match a.kind {
        ArgKind::Scalar => a.ty.ends_with('*'),
        _ => true,
    }
}

/// The first byte range inside `(trap, arg_index)` the gate ignores.
fn blob_mask(trap: i64, arg_index: u8) -> Option<&'static BlobMask> {
    BLOB_MASKS
        .iter()
        .find(|mask| mask.trap == trap && mask.arg_index == arg_index)
}

/// Compares two blob bodies outside the masked range and returns the first
/// differing byte offset.
fn first_blob_difference(mask: Option<&BlobMask>, ours: &[u8], golden: &[u8]) -> Option<usize> {
    let n = ours.len().min(golden.len());
    for i in 0..n {
        if ours[i] == golden[i] {
            continue;
        }
        if let Some(mask) = mask {
            if i >= mask.at && i < mask.at + mask.len {
                continue;
            }
        }
        return Some(i);
    }
    None
}

/// Compares one record against its twin. Returns false when the two records are
/// different calls, which means the streams have parted.
fn diff_record(
    m: &Manifests,
    index: usize,
    a: &JournalRecord,
    b: &JournalRecord,
    out: &mut Vec<Finding>,
) -> bool {
    if a.rec_type != b.rec_type || a.cmd != b.cmd {
        out.push(Finding {
            index,
            what: format!(
                "record kind: ours {} vs golden {}",
                describe(m, a),
                describe(m, b)
            ),
        });
        return false;
    }
    if let (Some(x), Some(y)) = (a.ret, b.ret) {
        if x as i32 != y as i32 {
            out.push(Finding {
                index,
                what: format!("{}: ret {} vs {}", describe(m, a), x as i32, y as i32),
            });
        }
    }
    diff_words(m, index, a, b, out);
    diff_blobs(m, index, a, b, out);
    true
}

/// Walks both journals and reports every difference, up to `max_findings`.
/// An empty finding list is the pass.
///
/// The `CG_INIT` records compare position for position. The snapshot brackets
/// after them align on their snapshot numbers first, because the oracle engine
/// reaches `CA_ACTIVE` one snapshot later than our rig does.
pub fn diff(
    m: &Manifests,
    ours: &[JournalRecord],
    golden: &[JournalRecord],
    max_findings: usize,
) -> DiffReport {
    let mut out: Vec<Finding> = Vec::new();
    let (head_a, brackets_a) = split(ours);
    let (head_b, brackets_b) = split(golden);

    if head_a != head_b {
        out.push(Finding {
            index: 0,
            what: format!("CG_INIT record count: ours {head_a} vs golden {head_b}"),
        });
    }
    for i in 0..head_a.min(head_b) {
        if out.len() >= max_findings {
            return report(out, 0, 0, 0);
        }
        if !diff_record(m, i, &ours[i], &golden[i], &mut out) {
            return report(out, 0, 0, 0);
        }
    }

    if brackets_a.is_empty() || brackets_b.is_empty() {
        out.push(Finding {
            index: head_a,
            what: "a journal holds no snapshot bracket".to_string(),
        });
        return report(out, 0, brackets_a.len(), brackets_b.len());
    }

    // Align past each side's seeding bracket. The oracle engine reaches
    // `CA_ACTIVE` one snapshot later than our rig, and the probe's first bracket
    // only seats the reliable-command cursor, so the first bracket that both
    // sides drain as a delta is one past the later of the two starts.
    let first_common = brackets_a[0].snapshot.max(brackets_b[0].snapshot) + 1;
    if !brackets_a.iter().any(|x| x.snapshot == first_common)
        || !brackets_b.iter().any(|x| x.snapshot == first_common)
    {
        out.push(Finding {
            index: head_a,
            what: format!("neither journal reaches the aligned snapshot {first_common}"),
        });
        return report(out, 0, brackets_a.len(), brackets_b.len());
    }
    let skip_a = brackets_a
        .iter()
        .position(|x| x.snapshot == first_common)
        .unwrap();
    let skip_b = brackets_b
        .iter()
        .position(|x| x.snapshot == first_common)
        .unwrap();
    let a = &brackets_a[skip_a..];
    let b = &brackets_b[skip_b..];
    let pairs = a.len().min(b.len());

    for k in 0..pairs {
        if out.len() >= max_findings {
            return report(out, k, skip_a, skip_b);
        }
        // A skipped snapshot inside the compared range would silently drop
        // coverage, so both sides must stay consecutive.
        if k > 0 && (a[k].snapshot != a[k - 1].snapshot + 1 || b[k].snapshot != b[k - 1].snapshot + 1)
        {
            out.push(Finding {
                index: a[k].at,
                what: format!(
                    "snapshot run broke: ours {} after {}, golden {} after {}",
                    a[k].snapshot,
                    a[k - 1].snapshot,
                    b[k].snapshot,
                    b[k - 1].snapshot
                ),
            });
            return report(out, k, skip_a, skip_b);
        }
        if a[k].len != b[k].len {
            out.push(Finding {
                index: a[k].at,
                what: format!(
                    "snapshot {} bracket holds {} records, golden holds {}",
                    a[k].snapshot,
                    a[k].len,
                    b[k].len
                ),
            });
            return report(out, k, skip_a, skip_b);
        }
        for j in 0..a[k].len {
            if !diff_record(
                m,
                a[k].at + j,
                &ours[a[k].at + j],
                &golden[b[k].at + j],
                &mut out,
            ) {
                return report(out, k, skip_a, skip_b);
            }
        }
    }
    report(out, pairs, skip_a, skip_b)
}

fn report(
    findings: Vec<Finding>,
    compared: usize,
    skipped_ours: usize,
    skipped_golden: usize,
) -> DiffReport {
    DiffReport {
        findings,
        compared,
        skipped_ours,
        skipped_golden,
    }
}

fn diff_words(
    m: &Manifests,
    index: usize,
    a: &JournalRecord,
    b: &JournalRecord,
    out: &mut Vec<Finding>,
) {
    if a.words.len() != b.words.len() {
        out.push(Finding {
            index,
            what: format!(
                "{}: word count {} vs {}",
                describe(m, a),
                a.words.len(),
                b.words.len()
            ),
        });
        return;
    }
    for (w, (x, y)) in a.words.iter().zip(b.words.iter()).enumerate() {
        if *x as i32 == *y as i32 {
            continue;
        }
        match a.rec_type {
            REC_SYSCALL_ENTER => {
                // Word 0 is the trap number, so arg N is word N + 1.
                if w > 0 && word_is_pointer(m, a.cmd, w - 1) {
                    continue;
                }
            }
            REC_VMCALL_ENTER => {
                if (a.cmd, w) == MASKED_EXPORT_WORD {
                    continue;
                }
            }
            _ => {}
        }
        out.push(Finding {
            index,
            what: format!(
                "{}: arg word {w} is {} vs {}",
                describe(m, a),
                *x as i32,
                *y as i32
            ),
        });
    }
}

fn diff_blobs(
    m: &Manifests,
    index: usize,
    a: &JournalRecord,
    b: &JournalRecord,
    out: &mut Vec<Finding>,
) {
    if a.blobs.len() != b.blobs.len() {
        out.push(Finding {
            index,
            what: format!(
                "{}: blob count {} vs {}",
                describe(m, a),
                a.blobs.len(),
                b.blobs.len()
            ),
        });
        return;
    }
    for (x, y) in a.blobs.iter().zip(b.blobs.iter()) {
        if x.arg_index != y.arg_index || x.kind != y.kind {
            out.push(Finding {
                index,
                what: format!(
                    "{}: blob shape (arg {}, kind {}) vs (arg {}, kind {})",
                    describe(m, a),
                    x.arg_index,
                    x.kind,
                    y.arg_index,
                    y.kind
                ),
            });
            continue;
        }
        if x.bytes.len() != y.bytes.len() {
            out.push(Finding {
                index,
                what: format!(
                    "{}: blob arg {} length {} vs {}",
                    describe(m, a),
                    x.arg_index,
                    x.bytes.len(),
                    y.bytes.len()
                ),
            });
            continue;
        }
        let mask = blob_mask(a.cmd, x.arg_index);
        if let Some(at) = first_blob_difference(mask, &x.bytes, &y.bytes) {
            out.push(Finding {
                index,
                what: format!(
                    "{}: blob arg {} differs at byte {at} ({:#04x} vs {:#04x})",
                    describe(m, a),
                    x.arg_index,
                    x.bytes[at],
                    y.bytes[at]
                ),
            });
        }
    }
}

/// Counts the records by type, for the summary line a passing run prints.
pub fn census(records: &[JournalRecord]) -> HashMap<u8, usize> {
    let mut out = HashMap::new();
    for r in records {
        *out.entry(r.rec_type).or_insert(0) += 1;
    }
    out
}

/// Names the masked ranges the gate applied, so a passing run states what it
/// did not compare.
pub fn exclusions() -> Vec<String> {
    let mut out = vec![
        "pointer arg words (host addresses)".to_string(),
        "CG_DRAW_ACTIVE_FRAME arg 0 (interpolated render time)".to_string(),
        "leading brackets before the first common snapshot".to_string(),
    ];
    for mask in BLOB_MASKS {
        out.push(format!(
            "trap {} arg {} bytes {}..{} ({})",
            mask.trap,
            mask.arg_index,
            mask.at,
            mask.at + mask.len,
            mask.why
        ));
    }
    out
}
