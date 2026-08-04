//! Shared trap/export argument-shape tables for the C6b recorder shim and the
//! headless replay referee. Parsed at runtime from the two committed manifests
//! (`trap-shapes.json`, `export-shapes.json`) so the replay can never drift from
//! what the recorder serialized. The shim generates its own tables at build time
//! (`shim/build.rs`); this file is the runtime-parsed twin the replay uses.
//! crates/cgame includes it via `#[path]` so it takes no dependency on the shim
//! crate (DEC-48.5 sharing rule).
//!
//! The length math here mirrors `shim/src/serialize.rs` one-for-one - counted
//! (len_arg / the trap-204 product) wins over a fixed size_of, and trap 204's
//! `args[2]*args[4]` is the only product special-case.

// Three consumers include this file by `#[path]` and each reads a subset of the
// tables, so an item with no reader in one consumer is live in another.
#![allow(dead_code)]

use std::path::Path;

use serde_json::Value;

/// The engine-retained shared region size (`MAX_CG_SHARED_BUFFER_SIZE`,
/// cg_public.h:593).
pub const SHARED_BUFFER_SIZE: usize = 2048;

/// CG_SET_SHARED_BUFFER trap number (cgameImport_t) - registers the region.
pub const CG_SET_SHARED_BUFFER: i64 = 344;

/// What a serialized region is, per the blob-kind byte in the journal.
pub const BLOB_IN_STR: u8 = 1;
pub const BLOB_IN_BUF: u8 = 2;
pub const BLOB_OUT_BUF: u8 = 3;
pub const BLOB_OUT_STR: u8 = 8;
pub const BLOB_INOUT_BUF: u8 = 4;
pub const BLOB_DOUBLE_PTR_SLOT: u8 = 5;
pub const BLOB_SHARED_BUFFER: u8 = 6;
pub const BLOB_RET_DEREF: u8 = 7;

/// arg-index sentinels the journal uses for the two non-positional blobs.
pub const ARG_SHARED: u8 = 0xFF;
pub const ARG_RET_DEREF: u8 = 0xFE;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ArgKind {
    Scalar,
    InStr,
    InBuf,
    OutBuf,
    OutStr,
    InoutBuf,
    DoublePtr,
    RetainedPtr,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RetKind {
    Void,
    Scalar,
    Handle,
    Float,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ExportRet {
    Void,
    Scalar,
    PtrOpaque,
    PtrDeref,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SharedKind {
    None,
    In,
    Out,
    Inout,
}

/// One argument's serialization shape (trap side). `len_arg` is the 0-based
/// `args[]` index holding the element count, or -1 for none; `elem_size` is the
/// per-element stride for those.
pub struct ArgShape {
    pub kind: ArgKind,
    /// The C type the engine casts to (`vec3_t`, `char*`, `CGhoul2Info_v*`, ...).
    /// A scalar whose type ends in `*` is a host pointer token (ghoul2 handle) -
    /// 64-bit and host-specific, so its word is never compared.
    pub ty: String,
    pub size_of: u32,
    pub len_arg: i32,
    pub elem_size: u32,
}

pub struct TrapShape {
    pub num: i64,
    pub name: String,
    pub ret: RetKind,
    pub args: Vec<ArgShape>,
}

/// One argument's shape on the export (vmMain) side. Export arg index N is the
/// raw word `argN` directly (no trap-number prefix).
pub struct ExportArg {
    pub kind: ArgKind,
    pub size_of: u32,
}

pub struct ExportShape {
    pub num: i64,
    pub name: String,
    pub ret: ExportRet,
    pub ret_size_of: u32,
    pub shared: SharedKind,
    pub args: Vec<ExportArg>,
}

/// Both manifests, parsed once at replay start.
pub struct Manifests {
    pub traps: Vec<TrapShape>,
    pub exports: Vec<ExportShape>,
}

impl Manifests {
    /// Loads and parses `trap-shapes.json` + `export-shapes.json` from `dir`.
    pub fn load(dir: &Path) -> Result<Manifests, String> {
        let traps = parse_traps(&dir.join("trap-shapes.json"))?;
        let exports = parse_exports(&dir.join("export-shapes.json"))?;
        Ok(Manifests { traps, exports })
    }

    pub fn trap(&self, num: i64) -> Option<&TrapShape> {
        self.traps.iter().find(|t| t.num == num)
    }

    pub fn export(&self, num: i64) -> Option<&ExportShape> {
        self.exports.iter().find(|e| e.num == num)
    }
}

fn arg_kind(kind: &str) -> ArgKind {
    match kind {
        "scalar" => ArgKind::Scalar,
        "in_str" => ArgKind::InStr,
        "in_buf" => ArgKind::InBuf,
        "out_buf" => ArgKind::OutBuf,
        "out_str" => ArgKind::OutStr,
        "inout_buf" => ArgKind::InoutBuf,
        "double_ptr" => ArgKind::DoublePtr,
        "retained_ptr" => ArgKind::RetainedPtr,
        other => panic!("unknown arg kind {other}"),
    }
}

fn trap_ret(ret: &str) -> RetKind {
    match ret {
        "void" => RetKind::Void,
        "scalar" => RetKind::Scalar,
        "handle" => RetKind::Handle,
        "float" => RetKind::Float,
        other => panic!("unknown trap ret {other}"),
    }
}

fn export_ret(ret: &str) -> ExportRet {
    match ret {
        "void" => ExportRet::Void,
        "scalar" => ExportRet::Scalar,
        "ptr_opaque" => ExportRet::PtrOpaque,
        "ptr_deref" => ExportRet::PtrDeref,
        other => panic!("unknown export ret {other}"),
    }
}

fn parse_traps(path: &Path) -> Result<Vec<TrapShape>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("parse trap json: {e}"))?;
    let arr = doc["traps"].as_array().ok_or("traps not an array")?;
    let mut out = Vec::with_capacity(arr.len());
    for t in arr {
        let num = t["num"].as_i64().unwrap();
        let name = t["name"].as_str().unwrap().to_string();
        let ret = trap_ret(t["ret"].as_str().unwrap());
        let mut args = Vec::new();
        for a in t["args"].as_array().unwrap() {
            let kind = arg_kind(a["kind"].as_str().unwrap());
            let ty = a["type"].as_str().unwrap_or("").to_string();
            let size_of = a["size_of"].as_u64().unwrap_or(0) as u32;
            let len_arg = a["len_arg"].as_i64().unwrap_or(-1) as i32;
            // element stride for len_arg buffers: the named size_of, else 1 byte.
            let elem_size = if size_of > 0 { size_of } else { 1 };
            args.push(ArgShape {
                kind,
                ty,
                size_of,
                len_arg,
                elem_size,
            });
        }
        out.push(TrapShape {
            num,
            name,
            ret,
            args,
        });
    }
    Ok(out)
}

fn parse_exports(path: &Path) -> Result<Vec<ExportShape>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("parse export json: {e}"))?;
    let arr = doc["exports"].as_array().ok_or("exports not an array")?;
    let mut out = Vec::with_capacity(arr.len());
    for e in arr {
        let num = e["num"].as_i64().unwrap();
        let name = e["name"].as_str().unwrap().to_string();
        let ret = export_ret(e["ret"].as_str().unwrap());
        let ret_size_of = e["ret_size_of"].as_u64().unwrap_or(0) as u32;
        let shared = match e.get("shared_buffer").and_then(|v| v.as_str()) {
            None => SharedKind::None,
            Some("in") => SharedKind::In,
            Some("out") => SharedKind::Out,
            Some("inout") => SharedKind::Inout,
            Some(other) => panic!("unknown shared_buffer {other}"),
        };
        let mut args = Vec::new();
        for a in e["args"].as_array().unwrap() {
            let kind = arg_kind(a["kind"].as_str().unwrap());
            let size_of = a["size_of"].as_u64().unwrap_or(0) as u32;
            args.push(ExportArg { kind, size_of });
        }
        out.push(ExportShape {
            num,
            name,
            ret,
            ret_size_of,
            shared,
            args,
        });
    }
    Ok(out)
}

/// Special count-by-product cases the manifest's single `len_arg` cannot express.
/// Mirrors `shim/src/serialize.rs`. `args` is the raw 16-word frame (args[0] =
/// trap number).
pub fn special_count(num: i64, idx: usize, args: &[i64]) -> Option<u64> {
    match (num, idx) {
        // CG_R_ADDPOLYSTOSCENE (204): verts = numVerts(args[2]) * numPolys(args[4]).
        // count words are 32-bit ints - mask off the variadic-trampoline garbage
        // in the high 32 bits before multiplying (see arg_len).
        (204, 2) => {
            Some(((args[2] as i32).max(0) as u64).saturating_mul((args[4] as i32).max(0) as u64))
        }
        _ => None,
    }
}

/// Byte count for a len_arg / special-count buffer at arg `idx`.
fn buf_len(shape_len_arg: i32, elem: u32, num: i64, idx: usize, args: &[i64]) -> u64 {
    let count = special_count(num, idx, args).unwrap_or_else(|| {
        if shape_len_arg >= 0 && (shape_len_arg as usize) < args.len() {
            // count args are 32-bit ints; the variadic trampoline grabs 64-bit
            // words whose high 32 bits are stack garbage. Masking to i32 is what
            // the engine dispatch does (it casts args[n] to int) - reading the
            // full 64-bit word blows past any sane length.
            (args[shape_len_arg as usize] as i32).max(0) as u64
        } else {
            0
        }
    });
    count.saturating_mul(elem.max(1) as u64)
}

/// Byte length of a buffer arg. Counted (len_arg / special-count) wins over the
/// fixed size_of - traps 34/203/204 carry BOTH an element stride in size_of AND
/// a count arg; a fixed size_of alone means exactly one struct.
pub fn arg_len(a: &ArgShape, num: i64, idx: usize, args: &[i64]) -> u64 {
    if a.len_arg >= 0 || special_count(num, idx, args).is_some() {
        buf_len(a.len_arg, a.elem_size, num, idx, args)
    } else {
        a.size_of as u64
    }
}
