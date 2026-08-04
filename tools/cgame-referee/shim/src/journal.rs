//! The length-prefixed little-endian journal format (the contract the headless
//! replay harness is written against - documented in ../README.md 'Journal
//! format'). One `Journal` owns a buffered file; records are built in memory
//! then flushed with a length prefix so a reader can skip any record it does
//! not understand.

// The blob kinds and the marker/flush pair are the journal format's full
// surface. The shim writes the subset its traps reach, and the readers name the
// rest, so an unconstructed variant here still documents the wire format.
#![allow(dead_code)]

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;

pub const MAGIC: &[u8; 8] = b"CGSHIMJ1";
pub const FORMAT_VERSION: u32 = 1;

// record types.
pub const REC_VMCALL_ENTER: u8 = 1;
pub const REC_VMCALL_EXIT: u8 = 2;
pub const REC_SYSCALL_ENTER: u8 = 3;
pub const REC_SYSCALL_EXIT: u8 = 4;
pub const REC_MALFORMED: u8 = 5;
pub const REC_MARKER: u8 = 6;

/// blob kinds - what a serialized region is.
#[derive(Clone, Copy)]
pub enum BlobKind {
    InStr = 1,
    InBuf = 2,
    OutBuf = 3,
    InoutBuf = 4,
    DoublePtrSlot = 5,
    SharedBuffer = 6,
    RetDeref = 7,
    OutStr = 8,
}

/// Anything the serializers can append a blob to. Keeps serialize.rs free of the
/// concrete `Record`.
pub trait BlobSink {
    fn blob(&mut self, arg_index: u8, kind: BlobKind, bytes: &[u8]);
}

/// One in-flight record. `head` is the fixed body after the seq; `blobs` is the
/// accumulated blob section. Assembled and length-prefixed at write time.
pub struct Record {
    rec_type: u8,
    seq: u64,
    head: Vec<u8>,
    blobs: Vec<u8>,
    blob_count: u16,
}

impl Record {
    pub fn new(rec_type: u8, seq: u64) -> Self {
        Record {
            rec_type,
            seq,
            head: Vec::new(),
            blobs: Vec::new(),
            blob_count: 0,
        }
    }

    pub fn push_i64(&mut self, v: i64) {
        self.head.extend_from_slice(&v.to_le_bytes());
    }

    /// Appends N raw arg words (i64 LE) preceded by a u8 count.
    pub fn push_words(&mut self, words: &[isize]) {
        self.head.push(words.len() as u8);
        for w in words {
            self.head.extend_from_slice(&(*w as i64).to_le_bytes());
        }
    }
}

impl BlobSink for Record {
    fn blob(&mut self, arg_index: u8, kind: BlobKind, bytes: &[u8]) {
        self.blobs.push(arg_index);
        self.blobs.push(kind as u8);
        self.blobs
            .extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        self.blobs.extend_from_slice(bytes);
        self.blob_count += 1;
    }
}

// the journal file is one gzip stream; the CGSHIMJ1 format lives inside it.
// Compression::fast() so deflate never stalls the trap path at record time.
pub struct Journal {
    w: BufWriter<GzEncoder<File>>,
}

impl Journal {
    pub fn create(path: &Path) -> io::Result<Journal> {
        let f = File::create(path)?;
        let mut w = BufWriter::new(GzEncoder::new(f, Compression::fast()));
        w.write_all(MAGIC)?;
        w.write_all(&FORMAT_VERSION.to_le_bytes())?;
        Ok(Journal { w })
    }

    pub fn write(&mut self, rec: &Record) {
        // payload = rec_type(1) + seq(8) + head + blob_count(2) + blobs.
        let payload_len = 1 + 8 + rec.head.len() + 2 + rec.blobs.len();
        let _ = self.w.write_all(&(payload_len as u32).to_le_bytes());
        let _ = self.w.write_all(&[rec.rec_type]);
        let _ = self.w.write_all(&rec.seq.to_le_bytes());
        let _ = self.w.write_all(&rec.head);
        let _ = self.w.write_all(&rec.blob_count.to_le_bytes());
        let _ = self.w.write_all(&rec.blobs);
    }

    /// A free-text bracket marker (module loaded, fatal setup failure, ...).
    pub fn marker(&mut self, seq: u64, text: &str) {
        let mut rec = Record::new(REC_MARKER, seq);
        rec.head
            .extend_from_slice(&(text.len() as u32).to_le_bytes());
        rec.head.extend_from_slice(text.as_bytes());
        self.write(&rec);
        let _ = self.w.flush();
    }

    pub fn flush(&mut self) {
        let _ = self.w.flush();
    }

    /// Ends the gzip stream (writes the trailer). Statics never run Drop at
    /// process exit, so the recorder calls this at CG_SHUTDOWN - the end of a
    /// recording session.
    pub fn finish(self) {
        match self.w.into_inner() {
            Ok(enc) => {
                if let Err(e) = enc.finish() {
                    eprintln!("cgame-shim: journal finish failed: {e}");
                }
            }
            Err(e) => eprintln!("cgame-shim: journal buffer flush failed: {e}"),
        }
    }
}
