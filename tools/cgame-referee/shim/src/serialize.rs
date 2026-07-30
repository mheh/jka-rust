//! Table-driven blob serializers. The tables (`TRAP_TABLE`, `EXPORT_TABLE`) are
//! generated from the two committed manifests by build.rs; the enums here are
//! the shapes those literals reference. Everything reads foreign engine memory
//! through raw pointers, so every reader null-checks and caps its length - a
//! recorder never aborts the session (DEC-48 ruling 1).

// `name`/`ret`/`SharedKind::Out` etc. are differ-facing metadata the recorder
// does not itself branch on yet.
#![allow(dead_code)]

use crate::journal::{BlobKind, BlobSink};

/// One argument's serialization shape (trap side). `len_arg` is the 0-based
/// `args[]` index holding the element count, or -1 for none; `elem_size` is the
/// per-element stride for those.
pub struct ArgShape {
    pub kind: ArgKind,
    pub size_of: u32,
    pub len_arg: i32,
    pub elem_size: u32,
}

/// One argument's shape on the export (vmMain) side. Export arg index N is the
/// raw word `argN` directly (no trap-number prefix).
pub struct ExportArg {
    pub kind: ArgKind,
    pub size_of: u32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ArgKind {
    Scalar,
    InStr,
    InBuf,
    OutBuf,
    InoutBuf,
    DoublePtr,
    RetainedPtr,
}

#[derive(Clone, Copy)]
pub enum RetKind {
    Void,
    Scalar,
    Handle,
    Float,
}

#[derive(Clone, Copy)]
pub enum ExportRet {
    Void,
    Scalar,
    PtrOpaque,
    PtrDeref,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SharedKind {
    None,
    In,
    Out,
    Inout,
}

pub struct TrapShape {
    pub num: i64,
    pub name: &'static str,
    pub ret: RetKind,
    pub dumps_shared: bool,
    pub args: &'static [ArgShape],
}

pub struct ExportShape {
    pub num: i64,
    pub name: &'static str,
    pub ret: ExportRet,
    pub ret_size_of: u32,
    pub shared: SharedKind,
    pub args: &'static [ExportArg],
}

include!(concat!(env!("OUT_DIR"), "/manifest_tables.rs"));

/// The engine-retained shared region size (`MAX_CG_SHARED_BUFFER_SIZE`,
/// cg_public.h:593).
pub const SHARED_BUFFER_SIZE: usize = 2048;

/// CG_SET_SHARED_BUFFER trap number (cgameImport_t) - registers the region.
pub const CG_SET_SHARED_BUFFER: i64 = 344;

/// Cap on any single serialized blob - a bad length from foreign memory never
/// makes us read gigabytes.
const MAX_BLOB: u64 = 1 << 20;

pub fn trap_shape(num: i64) -> Option<&'static TrapShape> {
    TRAP_TABLE.iter().find(|t| t.num == num)
}

pub fn export_shape(num: i64) -> Option<&'static ExportShape> {
    EXPORT_TABLE.iter().find(|e| e.num == num)
}

/// Reads a NUL-terminated C string at `ptr`, capped. Empty on null.
fn read_cstr(ptr: isize) -> Vec<u8> {
    if ptr == 0 {
        return Vec::new();
    }
    let base = ptr as *const u8;
    let mut out = Vec::new();
    // SAFETY: foreign engine string; capped so a missing NUL never runs away.
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

/// Reads `len` bytes at `ptr`. Empty on null or absurd length.
fn read_bytes(ptr: isize, len: u64) -> Vec<u8> {
    if ptr == 0 || len == 0 || len > MAX_BLOB {
        return Vec::new();
    }
    // SAFETY: foreign engine buffer sized by the manifest shape; capped above.
    unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
}

/// The pointee of a `double_ptr` slot: the 8-byte host pointer the engine stores
/// there. Read as raw bytes so the token is captured verbatim.
fn read_slot(slot_ptr: isize) -> Vec<u8> {
    read_bytes(slot_ptr, 8)
}

/// Special count-by-product cases the manifest's single `len_arg` cannot express.
/// Returns the element count for arg `idx` of trap `num`, or None to fall back to
/// the table's `len_arg`. `args` is the raw 16-word frame (args[0] = trap num).
fn special_count(num: i64, idx: usize, args: &[isize]) -> Option<u64> {
    match (num, idx) {
        // CG_R_ADDPOLYSTOSCENE (204): verts = numVerts(args[2]) * numPolys(args[4]).
        (204, 2) => Some((args[2].max(0) as u64).saturating_mul(args[4].max(0) as u64)),
        _ => None,
    }
}

/// Byte count for a len_arg / special-count buffer at arg `idx`.
fn buf_len(shape_len_arg: i32, elem: u32, num: i64, idx: usize, args: &[isize]) -> u64 {
    let count = special_count(num, idx, args).unwrap_or_else(|| {
        if shape_len_arg >= 0 && (shape_len_arg as usize) < args.len() {
            args[shape_len_arg as usize].max(0) as u64
        } else {
            0
        }
    });
    count.saturating_mul(elem.max(1) as u64)
}

/// Byte length of a buffer arg. Counted (len_arg / special-count) wins over the
/// fixed size_of - traps 34/203/204 carry BOTH an element stride in size_of AND
/// a count arg; a fixed size_of alone means exactly one struct.
fn arg_len(a: &ArgShape, num: i64, idx: usize, args: &[isize]) -> u64 {
    if a.len_arg >= 0 || special_count(num, idx, args).is_some() {
        buf_len(a.len_arg, a.elem_size, num, idx, args)
    } else {
        a.size_of as u64
    }
}

/// Serialize a trap's in-args at SYSCALL_ENTER. `args` is the 16-word frame;
/// arg shape index `i` addresses `args[i+1]` (the dispatch-index convention).
pub fn trap_enter_blobs(shape: &TrapShape, args: &[isize], sink: &mut dyn BlobSink) {
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = args[i + 1];
        match a.kind {
            ArgKind::InStr => sink.blob(i as u8, BlobKind::InStr, &read_cstr(ptr)),
            ArgKind::InBuf => {
                let len = arg_len(a, shape.num, i, args);
                sink.blob(i as u8, BlobKind::InBuf, &read_bytes(ptr, len));
            }
            ArgKind::InoutBuf => {
                let len = arg_len(a, shape.num, i, args);
                sink.blob(i as u8, BlobKind::InoutBuf, &read_bytes(ptr, len));
            }
            // slot value BEFORE the engine (maybe) writes a new host ptr back.
            ArgKind::DoublePtr => sink.blob(i as u8, BlobKind::DoublePtrSlot, &read_slot(ptr)),
            ArgKind::Scalar | ArgKind::OutBuf | ArgKind::RetainedPtr => {}
        }
    }
}

/// Serialize a trap's out-args at SYSCALL_EXIT (engine has written them).
pub fn trap_exit_blobs(shape: &TrapShape, args: &[isize], sink: &mut dyn BlobSink) {
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = args[i + 1];
        match a.kind {
            ArgKind::OutBuf => {
                let len = arg_len(a, shape.num, i, args);
                sink.blob(i as u8, BlobKind::OutBuf, &read_bytes(ptr, len));
            }
            ArgKind::InoutBuf => {
                let len = arg_len(a, shape.num, i, args);
                sink.blob(i as u8, BlobKind::InoutBuf, &read_bytes(ptr, len));
            }
            // engine-written token in the slot after INIT/DUPLICATE/etc.
            ArgKind::DoublePtr => sink.blob(i as u8, BlobKind::DoublePtrSlot, &read_slot(ptr)),
            ArgKind::Scalar | ArgKind::InStr | ArgKind::InBuf | ArgKind::RetainedPtr => {}
        }
    }
}

/// Serialize an export's in-args at VMCALL_ENTER. Export arg index N is `words[N]`.
pub fn export_enter_blobs(shape: &ExportShape, words: &[isize], sink: &mut dyn BlobSink) {
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = words[i];
        match a.kind {
            ArgKind::InStr => sink.blob(i as u8, BlobKind::InStr, &read_cstr(ptr)),
            ArgKind::InBuf => {
                sink.blob(i as u8, BlobKind::InBuf, &read_bytes(ptr, a.size_of as u64))
            }
            _ => {}
        }
    }
}

/// Serialize an export's out-args + pointer-return deref at VMCALL_EXIT.
pub fn export_exit_blobs(
    shape: &ExportShape,
    words: &[isize],
    ret: isize,
    sink: &mut dyn BlobSink,
) {
    for (i, a) in shape.args.iter().enumerate() {
        let ptr = words[i];
        match a.kind {
            ArgKind::OutBuf => sink.blob(
                i as u8,
                BlobKind::OutBuf,
                &read_bytes(ptr, a.size_of as u64),
            ),
            ArgKind::InoutBuf => sink.blob(
                i as u8,
                BlobKind::InoutBuf,
                &read_bytes(ptr, a.size_of as u64),
            ),
            _ => {}
        }
    }
    // the trajectory_t* arms: dereference the returned pointer (the engine reads
    // and writes through it in RoffSystem). ptr_opaque returns are left as the
    // bare token already in the ret word.
    if let ExportRet::PtrDeref = shape.ret {
        sink.blob(
            0xFE,
            BlobKind::RetDeref,
            &read_bytes(ret, shape.ret_size_of as u64),
        );
    }
}
