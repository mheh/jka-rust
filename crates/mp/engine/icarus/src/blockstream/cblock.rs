//! Raven `CBlock` — a block of `CBlockMember`s in an ICARUS block stream.

use mp_qshared::shared::vec3_t;

use crate::blockstream::cblock_member::BlockMember;

/// Raven `CBlock` → `Block` (§F idiomatic, ICARUS-D1 naming).
///
/// Owns its members in a `Vec` (Raven's `vector<CBlockMember*> m_members`).
/// `Create(int)`/`Init`/`Free`/`AddMember`/`GetMember` and the `Write` overloads
/// (live in-memory member builders on the parse path, ICARUS-D1) all port; only
/// `Duplicate` is §20-dropped (see `blockstream/mod.rs`).
/// Type definition source: `oracle/codemp/icarus/blockstream.h:109-154`
pub struct Block {
    /// All `BlockMember`s owned by this block.
    pub m_members: Vec<BlockMember>,
    /// ID of the block.
    pub m_id: i32,
    /// Block flags.
    pub m_flags: u8,
}

impl Block {
    /// Raven `CBlock::Init`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:187`
    pub fn init(&mut self) -> i32 {
        self.m_flags = 0;
        self.m_id = 0;
        1
    }

    /// Raven `CBlock::Create( int )` — reader/`ReadBlock` path.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:201`
    pub fn create(&mut self, block_id: i32) -> i32 {
        self.init();
        self.m_id = block_id;
        1
    }

    /// Raven `CBlock::Free`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:216`
    ///
    /// Raven walks `m_members` back-to-front `delete`ing each, bailing with
    /// `false` if a slot is ever `NULL` (never true — every index in range is
    /// populated), then clears the list. `Vec::clear` drops every owned
    /// `BlockMember` in one step and is behavior-identical (§C10).
    pub fn free(&mut self) -> i32 {
        self.m_members.clear();
        1
    }

    /// Raven `CBlock::Write( int, vector_t )` — live in-memory member builder.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:258`
    pub fn write_vector(&mut self, id: i32, data: vec3_t) -> i32 {
        let mut member = BlockMember {
            m_id: id,
            m_size: 0,
            m_data: Vec::new(),
        };
        member.set_data_vector(data);
        member.m_size = core::mem::size_of::<vec3_t>() as i32;
        self.add_member(member);
        1
    }

    /// Raven `CBlock::Write( int, float )` — live in-memory member builder.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:273`
    pub fn write_float(&mut self, id: i32, data: f32) -> i32 {
        let mut member = BlockMember {
            m_id: id,
            m_size: 0,
            m_data: Vec::new(),
        };
        member.write_data(&data.to_ne_bytes());
        member.m_size = core::mem::size_of::<f32>() as i32;
        self.add_member(member);
        1
    }

    /// Raven `CBlock::Write( int, const char * )` — live in-memory member builder.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:244`
    pub fn write_str(&mut self, id: i32, data: &str) -> i32 {
        let mut member = BlockMember {
            m_id: id,
            m_size: 0,
            m_data: Vec::new(),
        };
        member.set_data_str(data);
        member.m_size = data.len() as i32 + 1;
        self.add_member(member);
        1
    }

    /// Raven `CBlock::Write( int, int )` — live in-memory member builder.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:286`
    pub fn write_int(&mut self, id: i32, data: i32) -> i32 {
        let mut member = BlockMember {
            m_id: id,
            m_size: 0,
            m_data: Vec::new(),
        };
        member.write_data(&data.to_ne_bytes());
        member.m_size = core::mem::size_of::<i32>() as i32;
        self.add_member(member);
        1
    }

    /// Raven `CBlock::Write( CBlockMember * )` — live in-memory member builder.
    ///
    /// Raven's own comment flags its dead `SetSize` line as wrong
    /// (`// findme: this is wrong`, `BlockStream.cpp:302`); that line is
    /// already commented out in the oracle, so it contributes no behavior.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:300`
    pub fn write_member(&mut self, member: BlockMember) -> i32 {
        self.add_member(member);
        1
    }

    /// Raven `CBlock::AddMember`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:317`
    pub fn add_member(&mut self, member: BlockMember) -> i32 {
        self.m_members.push(member);
        1
    }

    /// Raven `CBlock::GetMember`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:329`
    ///
    /// Raven guards only the upper bound (`memberNum > count-1`); a negative
    /// index is UB in C (`operator[]` underflow). `Vec::get` returns `None`
    /// for the equivalent out-of-range `usize`, the defined behavior (§19).
    pub fn get_member(&self, member_num: i32) -> Option<&BlockMember> {
        if member_num > self.get_num_members() - 1 {
            return None;
        }
        self.m_members.get(member_num as usize)
    }

    /// Raven `CBlock::GetMemberData`.
    /// Source: `oracle/codemp/icarus/BlockStream.cpp:344`
    pub fn get_member_data(&self, member_num: i32) -> Option<&[u8]> {
        self.get_member(member_num).map(|m| m.m_data.as_slice())
    }

    /// Raven `CBlock::GetBlockID`.
    /// Source: `oracle/codemp/icarus/blockstream.h:141`
    pub fn get_block_id(&self) -> i32 {
        self.m_id
    }

    /// Raven `CBlock::GetNumMembers`.
    /// Source: `oracle/codemp/icarus/blockstream.h:142`
    pub fn get_num_members(&self) -> i32 {
        self.m_members.len() as i32
    }

    /// Raven `CBlock::HasFlag`.
    /// Source: `oracle/codemp/icarus/blockstream.h:147`
    pub fn has_flag(&self, flag: u8) -> i32 {
        (self.m_flags & flag) as i32
    }

    /// Raven `CBlock::GetFlags`.
    /// Source: `oracle/codemp/icarus/blockstream.h:148`
    pub fn get_flags(&self) -> u8 {
        self.m_flags
    }

    /// Raven `CBlock::SetFlags` — replace the flag word (`ReadBlock` caller).
    /// Source: `oracle/codemp/icarus/blockstream.h:141`
    pub fn set_flags(&mut self, flags: u8) {
        self.m_flags = flags;
    }

    /// Raven `CBlock::SetFlag` — OR a flag into the flag word.
    /// Source: `oracle/codemp/icarus/blockstream.h:142`
    pub fn set_flag(&mut self, flag: u8) {
        self.m_flags |= flag;
    }
}

// Small block-member read helpers (native-endian, §19-guarded on short data).
// Shared by the sequencer/task-manager dispatch paths and the game interface.

/// Read a member's raw data as a native-endian `f32` (`*(float *)GetMemberData`),
/// **without** advancing. Short/absent data reads `0.0` (§19 guard).
pub(crate) fn peek_member_float(block: &Block, member_num: i32) -> f32 {
    let data = block.get_member_data(member_num).unwrap_or(&[]);
    let mut buf = [0u8; 4];
    let n = data.len().min(4);
    buf[..n].copy_from_slice(&data[..n]);
    f32::from_ne_bytes(buf)
}

/// Read a member's raw data as a native-endian `f32`, advancing `member_num`.
pub(crate) fn member_float(block: &Block, member_num: &mut i32) -> f32 {
    let v = peek_member_float(block, *member_num);
    *member_num += 1;
    v
}

/// Read a member's raw data as a native-endian `i32` (`*(int *)GetMemberData`),
/// advancing `member_num`. Short/absent data reads `0` (§19 guard).
pub(crate) fn member_int(block: &Block, member_num: &mut i32) -> i32 {
    let data = block.get_member_data(*member_num).unwrap_or(&[]);
    *member_num += 1;
    let mut buf = [0u8; 4];
    let n = data.len().min(4);
    buf[..n].copy_from_slice(&data[..n]);
    i32::from_ne_bytes(buf)
}

/// A raw byte field (`(char *)GetMemberData`) read as a C string: bytes up to
/// the first NUL, lossy UTF-8.
pub(crate) fn bytes_to_c_string(data: &[u8]) -> String {
    let len = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..len]).into_owned()
}

/// Read a member's data as a C string **without** advancing.
pub(crate) fn peek_member_c_string(block: &Block, member_num: i32) -> String {
    let data = block.get_member_data(member_num).unwrap_or(&[]);
    bytes_to_c_string(data)
}

/// Read a member's data as a C string, advancing `member_num`.
pub(crate) fn member_c_string(block: &Block, member_num: &mut i32) -> String {
    let s = peek_member_c_string(block, *member_num);
    *member_num += 1;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(id: i32) -> BlockMember {
        BlockMember {
            m_id: id,
            m_size: 0,
            m_data: Vec::new(),
        }
    }

    /// `GetMember`'s upper-bound-only guard (`BlockStream.cpp:331`) leaves a
    /// negative index as C `operator[]` UB; the port must pick the defined
    /// "absent" result rather than panicking on the `as usize` wrap.
    #[test]
    fn get_member_negative_index_is_none_not_panic() {
        let mut block = Block {
            m_members: Vec::new(),
            m_id: 0,
            m_flags: 0,
        };
        block.add_member(member(1));
        assert!(block.get_member(-1).is_none());
        assert!(block.get_member_data(-1).is_none());
    }

    /// `GetMember`'s guard only rejects `memberNum > count-1`; in-range reads
    /// (including the last valid index) must still succeed.
    #[test]
    fn get_member_in_range_and_out_of_range() {
        let mut block = Block {
            m_members: Vec::new(),
            m_id: 0,
            m_flags: 0,
        };
        block.add_member(member(7));
        block.add_member(member(8));
        assert_eq!(block.get_member(1).map(|m| m.m_id), Some(8));
        assert!(block.get_member(2).is_none());
    }

    /// `CBlock::Free` (`BlockStream.cpp:216`) must drop every member and
    /// leave the list empty, matching the delete-then-`clear()` walk.
    #[test]
    fn free_clears_members() {
        let mut block = Block {
            m_members: Vec::new(),
            m_id: 0,
            m_flags: 0,
        };
        block.add_member(member(1));
        block.add_member(member(2));
        assert_eq!(block.free(), 1);
        assert_eq!(block.get_num_members(), 0);
    }
}
