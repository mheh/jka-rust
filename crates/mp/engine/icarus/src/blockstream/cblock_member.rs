//! Raven `CBlockMember` — one ID/size/data record in an ICARUS block stream.

use mp_qshared::shared::vec3_t;

/// Raven `interpreter.h` `ID_RANDOM` — the special member-ID sentinel
/// `ReadMember` checks for. `Interpreter.cpp`/`tokenizer.h` (where the enum
/// chain `TK_USERDEF`(8)..`NUM_USER_TOKENS`(19)..`ID_RANDOM` resolves to `37`)
/// are out-of-scope for this port (§ Out of scope), so the resolved value is
/// pinned here as a local constant rather than reached through that module.
/// Source: `oracle/codemp/icarus/interpreter.h:33-53` (`ID_RANDOM` is the 19th
/// entry after `NUM_USER_TOKENS = 19`, i.e. `19 + 18 = 37`).
const ID_RANDOM: i32 = 37;

/// Raven `Q3_INFINITE` — used by `ReadMember`'s `ID_RANDOM` special case.
/// Source: `oracle/codemp/game/g_public.h:9`
const Q3_INFINITE: f32 = 16777216.0;

/// Raven `CBlockMember` → `BlockMember` (§F idiomatic, ICARUS-D1 naming).
///
/// One ID/size/data record. The `void *m_data` blob becomes an owned `Vec<u8>`
/// (ICARUS-D3/ruling 20 drops the ICARUS arena, so this `TAG_ICARUS5`
/// allocation is owned here). `ReadMember` (reader) and the in-memory builders
/// `SetData`/`WriteData`/`WriteDataPointer` are live and port (ICARUS-D1); the
/// dead `WriteMember`/`Duplicate` half is §20-dropped (see `blockstream/mod.rs`).
/// Type definition source: `oracle/codemp/icarus/blockstream.h:38-105`
pub struct BlockMember {
    /// ID of the value contained in `m_data`.
    pub m_id: i32,
    /// Size of the `m_data` member variable.
    pub m_size: i32,
    /// Data for this member (Raven's `void *m_data`, owned here).
    pub m_data: Vec<u8>,
}

impl BlockMember {
    /// Raven `CBlockMember::ReadMember` — read the member's data, in block
    /// format, from the stream (advancing `pos`).
    ///
    /// `stream`/`pos` replace Raven's `char **stream, long *streamPos` (an
    /// owned byte slice + cursor, §C7); `m_id`/`m_size` are read as raw
    /// little-endian 4-byte fields — Raven's `*(int *)`/`*(long *)` casts are
    /// 4 bytes wide on its LLP64 target (`sizeof(long) == sizeof(int) == 4`),
    /// and the `.IBI` bytes are the on-disk wire format, so an explicit fixed
    /// endianness (not native-endian) is required for parse parity.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:102`
    pub fn read_member(&mut self, stream: &[u8], pos: &mut i64) -> i32 {
        let p = *pos as usize;
        self.m_id = i32::from_le_bytes(stream[p..p + 4].try_into().unwrap());
        *pos += 4;

        if self.m_id == ID_RANDOM {
            // Special case, need to initialize this member's data to
            // Q3_INFINITE so we can randomize the number only the first time
            // random is checked when inside a wait.
            self.m_size = 4; // sizeof( float )
            *pos += 4; // sizeof( long ) — on-disk size field skipped, not read
            self.m_data = Q3_INFINITE.to_le_bytes().to_vec();
        } else {
            let p = *pos as usize;
            self.m_size = i32::from_le_bytes(stream[p..p + 4].try_into().unwrap());
            *pos += 4;
            let p = *pos as usize;
            let size = self.m_size as usize;
            self.m_data = stream[p..p + size].to_vec();
        }
        *pos += self.m_size as i64;

        true as i32
    }

    /// Raven `CBlockMember::SetData( const char * )`.
    /// Source: `oracle/codemp/icarus/blockstream.h:56`
    pub fn set_data_str(&mut self, data: &str) {
        // `strlen(data)+1` — the C-string bytes plus the NUL terminator.
        let mut bytes = data.as_bytes().to_vec();
        bytes.push(0);
        self.write_data_pointer(&bytes);
    }

    /// Raven `CBlockMember::SetData( vector_t )`.
    /// Source: `oracle/codemp/icarus/blockstream.h:57`
    pub fn set_data_vector(&mut self, data: vec3_t) {
        let mut bytes = Vec::with_capacity(12);
        for component in data {
            bytes.extend_from_slice(&component.to_ne_bytes());
        }
        self.write_data_pointer(&bytes);
    }

    /// Raven `CBlockMember::SetData( void *data, int size )`.
    /// Source: `oracle/codemp/icarus/blockstream.h:58`
    pub fn set_data(&mut self, data: &[u8]) {
        self.m_data = data.to_vec();
        self.m_size = data.len() as i32;
    }

    /// Raven `CBlockMember::WriteData` (template) — in-memory member-data copy
    /// (live, ICARUS-D1).
    /// Source: `oracle/codemp/icarus/blockstream.h:76`
    pub fn write_data(&mut self, data: &[u8]) {
        self.m_data = data.to_vec();
        self.m_size = data.len() as i32;
    }

    /// Raven `CBlockMember::WriteDataPointer` (template) — exact-byte member
    /// copy, byte fidelity preserved (live, ICARUS-D1; Divergences).
    /// Source: `oracle/codemp/icarus/blockstream.h:88`
    pub fn write_data_pointer(&mut self, data: &[u8]) {
        self.m_data = data.to_vec();
        self.m_size = data.len() as i32;
    }

    /// Raven `CBlockMember::GetInfo` — id, size, and data out-params folded to a
    /// return (§C7).
    /// Source: `oracle/codemp/icarus/blockstream.h:52`
    pub fn get_info(&self) -> (i32, i32, &[u8]) {
        (self.m_id, self.m_size, &self.m_data)
    }

    /// Raven `CBlockMember::Free`.
    ///
    /// Raven guards the reset on `m_data != NULL`; the owned-`Vec` analog of
    /// "no allocation held" is `is_empty()` (an empty `Vec` never holds a
    /// live `TAG_ICARUS5` allocation to release).
    /// Source: `oracle/codemp/icarus/blockstream.h:44`
    pub fn free(&mut self) {
        if !self.m_data.is_empty() {
            self.m_data = Vec::new();
            self.m_id = -1;
            self.m_size = -1;
        }
    }
}

impl Default for BlockMember {
    /// Raven `CBlockMember::CBlockMember` — the zero-arg constructor.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:26`
    fn default() -> Self {
        BlockMember {
            m_id: -1,
            m_size: -1,
            m_data: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_member_random_sentinel_hardcodes_infinite_and_skips_stream_data() {
        // Raven's ID_RANDOM branch never reads the data field from the
        // stream — it hardcodes Q3_INFINITE — but still advances `pos` past
        // both the on-disk size field and the on-disk data field, exactly as
        // the non-random path would, so the cursor stays in sync with the
        // record layout for whatever follows in the stream.
        let mut stream = Vec::new();
        stream.extend_from_slice(&ID_RANDOM.to_le_bytes());
        stream.extend_from_slice(&0xDEAD_BEEFu32.to_le_bytes()); // on-disk size field, ignored
        stream.extend_from_slice(&0x1234_5678u32.to_le_bytes()); // on-disk data field, ignored
        let mut pos: i64 = 0;

        let mut member = BlockMember::default();
        let ok = member.read_member(&stream, &mut pos);

        assert_eq!(ok, 1);
        assert_eq!(pos, 12);
        assert_eq!(member.m_id, ID_RANDOM);
        assert_eq!(member.m_size, 4);
        assert_eq!(member.m_data, Q3_INFINITE.to_le_bytes().to_vec());
    }

    #[test]
    fn read_member_normal_record_reads_size_and_data_from_stream() {
        let mut stream = Vec::new();
        stream.extend_from_slice(&42i32.to_le_bytes()); // m_id
        stream.extend_from_slice(&4i32.to_le_bytes()); // m_size
        stream.extend_from_slice(&7i32.to_le_bytes()); // data payload
        let mut pos: i64 = 0;

        let mut member = BlockMember::default();
        member.read_member(&stream, &mut pos);

        assert_eq!(pos, 12);
        assert_eq!(member.m_id, 42);
        assert_eq!(member.m_size, 4);
        assert_eq!(member.m_data, 7i32.to_le_bytes().to_vec());
    }

    #[test]
    fn set_data_str_appends_nul_terminator() {
        let mut member = BlockMember::default();
        member.set_data_str("hi");
        assert_eq!(member.m_data, vec![b'h', b'i', 0]);
        assert_eq!(member.m_size, 3);
    }

    #[test]
    fn free_is_noop_when_data_absent() {
        // Constructed directly with `m_data: Vec::new()` (never went through
        // SetData/WriteData), mirroring a member whose data pointer is NULL
        // — Free()'s guard leaves m_id/m_size untouched.
        let mut member = BlockMember {
            m_id: 5,
            m_size: 5,
            m_data: Vec::new(),
        };
        member.free();
        assert_eq!(member.m_id, 5);
        assert_eq!(member.m_size, 5);
    }

    #[test]
    fn free_resets_id_and_size_when_data_present() {
        let mut member = BlockMember::default();
        member.set_data(&[1, 2, 3]);
        member.free();
        assert!(member.m_data.is_empty());
        assert_eq!(member.m_id, -1);
        assert_eq!(member.m_size, -1);
    }
}
