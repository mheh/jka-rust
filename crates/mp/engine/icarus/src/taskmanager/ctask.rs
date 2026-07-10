//! Raven `CTask` — a scheduled ICARUS task.

use crate::blockstream::cblock::Block;

/// Raven `CTask` → `Task` (§F idiomatic, ICARUS-D1 naming).
///
/// A scheduled task; owned in `TaskManager.m_tasks: Vec<Task>` (Raven's
/// `list<CTask*> m_tasks`, ICARUS-D3 / ruling 27). No cross-object pointer
/// aliasing (groups track completion by int GUID), so literal transcription
/// with no id newtype needed; `m_block` is an owned `Block`.
/// Type definition source: `oracle/codemp/icarus/taskmanager.h:33-58`
pub struct Task {
    /// Raven `int m_id` — the task GUID.
    pub m_id: i32,
    /// Raven `DWORD m_timeStamp`.
    pub m_time_stamp: u32,
    /// Raven `CBlock *m_block` — owned here.
    pub m_block: Block,
}

impl Task {
    /// Raven `CTask::Create` — `new CTask` never returns null here (no
    /// `operator new` overload on `CTask`, unlike `CBlock`/`CBlockMember`), so the
    /// `assert`/NULL-check guard is dead; the port constructs the value directly.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:35-49`
    pub fn create(guid: i32, block: Block) -> Task {
        Task {
            m_id: guid,
            m_time_stamp: 0,
            m_block: block,
        }
    }

    /// Raven `CTask::Free` — `delete this`.
    // Rust: the task lives by value in `TaskManager.m_tasks: Vec<Task>` (§B9), so
    // removal — and the owned `m_block`'s drop — is the caller's job; no-op here.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:57-61`
    pub fn free(&mut self) {}

    /// Raven `CTask::GetID` — the owned block's id.
    /// Source: `oracle/codemp/icarus/taskmanager.h:48`
    pub fn get_id(&self) -> i32 {
        self.m_block.get_block_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_seeds_zero_timestamp_and_carries_guid_and_block() {
        let block = Block {
            m_members: Vec::new(),
            m_id: 42,
            m_flags: 0,
        };
        let task = Task::create(7, block);
        assert_eq!(task.m_id, 7);
        assert_eq!(task.m_time_stamp, 0);
        // `GetID` reads through to the owned block, not `m_id` (the GUID).
        assert_eq!(task.get_id(), 42);
    }
}
