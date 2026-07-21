//! Raven `CBlockStream` — the `.IBI` block-stream reader.

use crate::blockstream::cblock::Block;
use crate::blockstream::cblock_member::BlockMember;

/// Raven `IBI_HEADER_ID "IBI"` (`blockstream.h:19`) — includes the trailing
/// NUL, matching `id_header[sizeof(IBI_HEADER_ID)]` (`BlockStream.cpp:667`).
const IBI_HEADER_ID: &[u8; 4] = b"IBI\0";

/// Raven `IBI_VERSION 1.57f` (`blockstream.h:21`).
const IBI_VERSION: f32 = 1.57;

/// Raven `CBlockStream` → `BlockStream` (§F idiomatic, ICARUS-D1 naming).
///
/// The `.IBI` reader over an owned byte buffer (Raven's `char *m_stream` +
/// `FILE *m_fileHandle`; the writer half is §20-dropped, so no file handle is
/// kept). `Open`/`ReadBlock`/`BlockAvailable`/`Init` (live — `Open` calls
/// `Init`, closing ICARUS-Q13) and the `Get*` primitives port; the file writer
/// `Create(char *)`/`WriteBlock` are §20-dropped (see `blockstream/mod.rs`).
/// `IBI_HEADER_ID "IBI"`, `IBI_VERSION 1.57`.
/// Type definition source: `oracle/codemp/icarus/blockstream.h:158-196`
#[derive(Default)]
pub struct BlockStream {
    /// Size of the current stream.
    pub m_file_size: i64,
    /// Name of the current file.
    pub m_file_name: String,
    /// Stream of data being parsed (owned).
    pub m_stream: Vec<u8>,
    /// Current read cursor into `m_stream`.
    pub m_stream_pos: i64,
}

impl BlockStream {
    /// Raven `CBlockStream::Init` — live (`Open` calls it, ICARUS-D1).
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:558`
    pub fn init(&mut self) -> i32 {
        // Raven also clears `m_fileHandle`; no handle is kept here (the
        // writer half is §20-dropped), so only the reader state resets.
        self.m_file_name.clear();
        self.m_stream = Vec::new();
        self.m_stream_pos = 0;

        true as i32
    }

    /// Raven `CBlockStream::Free`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp` (`blockstream.h:169`)
    pub fn free(&mut self) -> i32 {
        // Raven: "It is assumed that the user will free the passed memory
        // block (m_stream) immediately after the run call" — this only
        // clears the internal cursor, not the caller's original buffer.
        self.m_stream = Vec::new();
        self.m_stream_pos = 0;

        true as i32
    }

    /// Raven `CBlockStream::BlockAvailable`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:605`
    pub fn block_available(&self) -> i32 {
        if self.m_stream_pos >= self.m_file_size {
            false as i32
        } else {
            true as i32
        }
    }

    /// Raven `CBlockStream::ReadBlock`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:619`
    pub fn read_block(&mut self, block: &mut Block) -> i32 {
        if self.block_available() == 0 {
            return false as i32;
        }

        let b_id = self.get_integer();
        let num_members = self.get_integer();
        let flags = self.get_char() as u8;

        if num_members < 0 {
            return false as i32;
        }

        block.create(b_id);
        // Raven `block->SetFlags( flags )` (`BlockStream.cpp:630`).
        block.set_flags(flags);

        // Stream blocks are temporary, used only during the initial parse
        // (Raven's `_XBOX` zone-temp toggle is a platform-specific allocator
        // hint and out of scope here).
        for _ in 0..num_members {
            // Mirrors `CBlockMember::CBlockMember()` (`BlockStream.cpp:26-30`).
            let mut member = BlockMember {
                m_id: -1,
                m_size: -1,
                m_data: Vec::new(),
            };
            member.read_member(&self.m_stream, &mut self.m_stream_pos);
            block.add_member(member);
        }

        true as i32
    }

    /// Raven `CBlockStream::Open( char *, long )` — opens an in-memory `.IBI`
    /// buffer for reading; calls `Init` at `BlockStream.cpp:670`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:665`
    pub fn open(&mut self, buffer: &[u8]) -> i32 {
        self.init();

        self.m_file_size = buffer.len() as i64;
        // Raven assigns the caller's pointer (`m_stream = buffer;`, no copy);
        // this class owns its buffer (module doc), so it copies instead.
        self.m_stream = buffer.to_vec();

        let mut id_header = [0u8; 4];
        for slot in id_header.iter_mut() {
            *slot = self.get_char() as u8;
        }

        let version = self.get_float();

        // `strcmp( id_header, IBI_HEADER_ID )` — compare up to the first NUL.
        let mut header_matches = true;
        for i in 0..id_header.len() {
            if id_header[i] != IBI_HEADER_ID[i] {
                header_matches = false;
                break;
            }
            if id_header[i] == 0 {
                break;
            }
        }

        if !header_matches {
            self.free();
            return false as i32;
        }

        if version != IBI_VERSION {
            self.free();
            return false as i32;
        }

        true as i32
    }

    /// Raven `CBlockStream::GetUnsignedInteger`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp` (`blockstream.h:183`)
    pub fn get_unsigned_integer(&mut self) -> u32 {
        let pos = self.m_stream_pos as usize;
        let data = u32::from_le_bytes(self.m_stream[pos..pos + 4].try_into().unwrap());
        self.m_stream_pos += 4;
        data
    }

    /// Raven `CBlockStream::GetInteger`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp` (`blockstream.h:184`)
    pub fn get_integer(&mut self) -> i32 {
        let pos = self.m_stream_pos as usize;
        let data = i32::from_le_bytes(self.m_stream[pos..pos + 4].try_into().unwrap());
        self.m_stream_pos += 4;
        data
    }

    /// Raven `CBlockStream::GetChar`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp` (`blockstream.h:186`)
    pub fn get_char(&mut self) -> i8 {
        let pos = self.m_stream_pos as usize;
        let data = self.m_stream[pos] as i8;
        self.m_stream_pos += 1;
        data
    }

    /// Raven `CBlockStream::GetLong`.
    ///
    /// Raven's `long` is the retail Win32 4-byte width (LLP64), not an 8-byte
    /// host `long`; read 4 bytes and sign-extend to the pinned `i64` return.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp` (`blockstream.h:187`)
    pub fn get_long(&mut self) -> i64 {
        let pos = self.m_stream_pos as usize;
        let data = i32::from_le_bytes(self.m_stream[pos..pos + 4].try_into().unwrap());
        self.m_stream_pos += 4;
        data as i64
    }

    /// Raven `CBlockStream::GetFloat`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp` (`blockstream.h:188`)
    pub fn get_float(&mut self) -> f32 {
        let pos = self.m_stream_pos as usize;
        let data = f32::from_le_bytes(self.m_stream[pos..pos + 4].try_into().unwrap());
        self.m_stream_pos += 4;
        data
    }

    /// Raven `CBlockStream::StripExtension` — backward-scans to the LAST `.`
    /// (distinct from `COM_StripExtension`'s forward scan to the first `.`).
    /// Shared as the one icarus-crate backward-scan copy (`CSequencer::Run`
    /// routes here); it reads none of `CBlockStream`'s state, so it is an
    /// associated fn rather than a method.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp` (`blockstream.h:190`)
    pub(crate) fn strip_extension(input: &str) -> String {
        // Raven scans backward from `strlen(in)` (whose first read is the
        // implicit NUL, never '.') for the last '.'; scanning from
        // `len - 1` is equivalent and skips that redundant first step.
        let bytes = input.as_bytes();
        let mut i = bytes.len() as isize - 1;
        while i >= 0 && bytes[i as usize] != b'.' {
            i -= 1;
        }

        if i < 0 {
            input.to_string()
        } else {
            input[..i as usize].to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_bytes(version: f32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"IBI\0");
        buf.extend_from_slice(&version.to_le_bytes());
        buf
    }

    #[test]
    fn open_accepts_valid_header_and_version() {
        let mut stream = BlockStream::default();
        let buf = header_bytes(IBI_VERSION);
        assert_eq!(stream.open(&buf), 1);
        assert_eq!(stream.m_stream_pos, 8);
    }

    #[test]
    fn open_rejects_bad_header() {
        let mut stream = BlockStream::default();
        let mut buf = Vec::new();
        buf.extend_from_slice(b"XYZ\0");
        buf.extend_from_slice(&IBI_VERSION.to_le_bytes());
        assert_eq!(stream.open(&buf), 0);
        // `Free` runs on the reject path, clearing the stream.
        assert!(stream.m_stream.is_empty());
    }

    #[test]
    fn open_rejects_bad_version() {
        let mut stream = BlockStream::default();
        let buf = header_bytes(1.0);
        assert_eq!(stream.open(&buf), 0);
    }

    #[test]
    fn block_available_matches_position_vs_size() {
        let mut stream = BlockStream::default();
        stream.m_file_size = 4;
        stream.m_stream_pos = 0;
        assert_eq!(stream.block_available(), 1);
        stream.m_stream_pos = 4;
        assert_eq!(stream.block_available(), 0);
        stream.m_stream_pos = 5;
        assert_eq!(stream.block_available(), 0);
    }

    #[test]
    fn get_primitives_advance_the_cursor_little_endian() {
        let mut stream = BlockStream::default();
        stream.m_stream = vec![0x2a, 0xff, 0xff, 0xff, 0xff];
        stream.m_file_size = stream.m_stream.len() as i64;

        assert_eq!(stream.get_char(), 0x2a);
        assert_eq!(stream.m_stream_pos, 1);

        stream.m_stream_pos = 1;
        assert_eq!(stream.get_integer(), -1);
        assert_eq!(stream.m_stream_pos, 5);
    }

    #[test]
    fn strip_extension_finds_last_dot() {
        assert_eq!(BlockStream::strip_extension("foo.bar.ibi"), "foo.bar");
        assert_eq!(BlockStream::strip_extension("noext"), "noext");
        assert_eq!(BlockStream::strip_extension(""), "");
    }
}
