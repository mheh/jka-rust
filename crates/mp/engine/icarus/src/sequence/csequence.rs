//! Raven `CSequence` — a command/child tree node.

use crate::blockstream::cblock::Block;
use crate::instance::icarus_instance::SequenceId;

/// Raven's anonymous `enum { POP_FRONT, POP_BACK, PUSH_FRONT, PUSH_BACK }` —
/// the `side` argument to `PopCommand`/`PushCommand`.
/// Source: `oracle/codemp/icarus/blockstream.h:26-32`
const POP_FRONT: i32 = 0;
const POP_BACK: i32 = 1;
const PUSH_FRONT: i32 = 2;
const PUSH_BACK: i32 = 3;

/// Raven `#define SQ_COMMON 0x00000000` — common one-pass sequence flag.
/// Source: `oracle/codemp/icarus/sequencer.h:25`
const SQ_COMMON: i32 = 0x0000_0000;

/// Raven `CSequence` → `Sequence` (§F idiomatic, ICARUS-D1 naming).
///
/// A command/child tree node. Its pointer graph becomes id newtypes into the
/// `IcarusInstance.sequences` arena (ICARUS-D3 / ruling 27): `m_children`,
/// `m_parent`, `m_return` → `SequenceId` handles; `m_commands` → an owned
/// `Vec<Block>` (no cross-object aliasing, literal transcription). The
/// `ICARUS_Instance *m_owner` back-ref is dropped (ruling 24). `Save`/`Load`
/// are inert in MP dedicated (Divergences).
/// Type definition source: `oracle/codemp/icarus/sequence.h:12-96`
#[derive(Default)]
pub struct Sequence {
    /// Raven `sequence_l m_children`.
    pub m_children: Vec<SequenceId>,
    /// Raven `CSequence *m_parent`.
    pub m_parent: Option<SequenceId>,
    /// Raven `CSequence *m_return`.
    pub m_return: Option<SequenceId>,
    /// Raven `block_l m_commands` — owned command blocks.
    pub m_commands: Vec<Block>,
    /// Raven `int m_flags`.
    pub m_flags: i32,
    /// Raven `int m_iterations`.
    pub m_iterations: i32,
    /// Raven `int m_id`.
    pub m_id: i32,
    /// Raven `int m_numCommands`.
    pub m_num_commands: i32,
}

impl Sequence {
    /// Raven `CSequence::Create`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:34-46` (`sequence.h:20`)
    pub fn create() -> Sequence {
        // The ctor sets `m_iterations = 1` (`m_numCommands`/`m_flags` default
        // to 0, `m_parent`/`m_return` to `NULL`, all matching `Default`).
        // The `assert(seq); if (seq == NULL) return NULL;` OOM guard has no
        // Rust analog (allocation failure aborts, not a null return), so it
        // is dropped rather than transcribed.
        let mut sequence = Sequence {
            m_iterations: 1,
            ..Default::default()
        };
        // `SetFlag`/`RemoveFlag` aren't part of this file's pinned API (all
        // fields are `pub`), so `seq->SetFlag(SQ_COMMON)` inlines as an OR.
        sequence.m_flags |= SQ_COMMON;
        sequence
    }

    /// Raven `CSequence::Delete`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:54-87` (`sequence.h:21`)
    pub fn delete(&mut self) {
        // Raven notifies `m_parent->RemoveChild(this)` and reparents each
        // child (`child->SetParent(NULL)`) using the *live* objects; this
        // arena-blind node only holds ids, so that graph fixup is the job of
        // the owning `IcarusInstance::delete_sequence`, not this method.
        self.m_children.clear();
        // `m_commands` are owned `Block`s (no separate `delete`/`Free` call
        // needed — dropping the `Vec` frees them).
        self.m_commands.clear();
    }

    /// Raven `CSequence::AddChild`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:96-103` (`sequence.h:24`)
    pub fn add_child(&mut self, child: SequenceId) {
        // Raven asserts/guards on a NULL `CSequence*`; `SequenceId` can't be
        // null, so that guard has no analog here.
        self.m_children.push(child);
    }

    /// Raven `CSequence::RemoveChild`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:111-119` (`sequence.h:25`)
    pub fn remove_child(&mut self, child: SequenceId) {
        // `list::remove` drops every equal element; `retain` matches that.
        self.m_children.retain(|&id| id != child);
    }

    /// Raven `CSequence::SetParent`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:149-162` (`sequence.h:27`)
    pub fn set_parent(&mut self, parent: Option<SequenceId>) {
        self.m_parent = parent;
        // Raven also inherits `SQ_RETAIN`/`SQ_PENDING` off the parent's *live*
        // `m_flags` (`Sequence.cpp:156-161`). This arena-blind node holds only
        // the parent's id, not its flags, so that inheritance is done by the
        // arena-aware `sequencer::seq_set_parent_inherit`, which the sequencer
        // routes every `SetParent` through (restoring the full behavior).
    }

    /// Raven `CSequence::PopCommand`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:170-203` (`sequence.h:31`)
    pub fn pop_command(&mut self, side: i32) -> Option<Block> {
        if self.m_commands.is_empty() {
            return None;
        }
        match side {
            POP_FRONT => {
                let command = self.m_commands.remove(0);
                self.m_num_commands -= 1;
                Some(command)
            }
            POP_BACK => {
                let command = self.m_commands.pop();
                self.m_num_commands -= 1;
                command
            }
            // Invalid flag.
            _ => None,
        }
    }

    /// Raven `CSequence::PushCommand`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:211-238` (`sequence.h:32`)
    pub fn push_command(&mut self, command: Block, side: i32) -> i32 {
        match side {
            PUSH_FRONT => {
                self.m_commands.insert(0, command);
                self.m_num_commands += 1;
                true as i32
            }
            PUSH_BACK => {
                self.m_commands.push(command);
                self.m_num_commands += 1;
                true as i32
            }
            // Invalid flag.
            _ => false as i32,
        }
    }

    /// Raven `CSequence::HasFlag`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:278-281` (`sequence.h:37`)
    pub fn has_flag(&self, flag: i32) -> i32 {
        self.m_flags & flag
    }

    /// Raven `CSequence::GetChildByIndex`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:301-312` (`sequence.h:56`)
    pub fn get_child_by_index(&self, id: i32) -> Option<SequenceId> {
        if id < 0 || id as usize >= self.m_children.len() {
            return None;
        }
        Some(self.m_children[id as usize])
    }

    /// Raven `CSequence::HasChild`.
    /// Source: `oracle/codemp/icarus/Sequence.cpp:127-141` (`sequence.h:57`)
    pub fn has_child(&self, sequence: SequenceId) -> bool {
        // Raven recurses into each child's own subtree
        // (`(*ci)->HasChild(sequence)`); this arena-blind node can only see its
        // direct `m_children` ids, so the recursive descendant walk is done by
        // the arena-aware `sequencer::seq_has_child` (which the sequencer's
        // `Flush` uses), restoring the full membership test.
        self.m_children.contains(&sequence)
    }

    /// Raven `CSequence::Save` — inert in MP dedicated (Divergences).
    /// Source: `oracle/codemp/icarus/Sequence.cpp:363-406` (`sequence.h:61`)
    pub fn save(&self) -> i32 {
        // Body is `#if 0` in retail; the live code is just `return false;`.
        false as i32
    }

    /// Raven `CSequence::Load` — inert in MP dedicated (Divergences).
    /// Source: `oracle/codemp/icarus/Sequence.cpp:414-559` (`sequence.h:62`)
    pub fn load(&mut self) -> i32 {
        // Body is `#if 0` in retail; the live code is just `return false;`.
        false as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: i32) -> Block {
        Block {
            m_members: Vec::new(),
            m_id: id,
            m_flags: 0,
        }
    }

    #[test]
    fn create_sets_one_iteration_and_common_flag() {
        let seq = Sequence::create();
        assert_eq!(seq.m_iterations, 1);
        assert_eq!(seq.m_flags, SQ_COMMON);
        assert!(seq.m_parent.is_none());
        assert!(seq.m_return.is_none());
    }

    #[test]
    fn push_pop_front_and_back_preserve_order() {
        let mut seq = Sequence::create();
        assert_eq!(seq.push_command(block(1), PUSH_BACK), 1);
        assert_eq!(seq.push_command(block(2), PUSH_BACK), 1);
        assert_eq!(seq.push_command(block(0), PUSH_FRONT), 1);
        assert_eq!(seq.m_num_commands, 3);

        // Order is now [0, 1, 2].
        assert_eq!(seq.pop_command(POP_FRONT).unwrap().m_id, 0);
        assert_eq!(seq.pop_command(POP_BACK).unwrap().m_id, 2);
        assert_eq!(seq.m_num_commands, 1);
        assert_eq!(seq.pop_command(POP_FRONT).unwrap().m_id, 1);
        assert!(seq.pop_command(POP_FRONT).is_none());
    }

    #[test]
    fn push_pop_invalid_side_is_a_no_op_failure() {
        let mut seq = Sequence::create();
        assert_eq!(seq.push_command(block(1), 99), 0);
        assert_eq!(seq.m_num_commands, 0);

        seq.push_command(block(1), PUSH_BACK);
        assert!(seq.pop_command(99).is_none());
        // The bad `side` didn't consume the command.
        assert_eq!(seq.m_num_commands, 1);
    }

    #[test]
    fn add_remove_and_index_children() {
        let mut seq = Sequence::create();
        let a = SequenceId(1);
        let b = SequenceId(2);
        seq.add_child(a);
        seq.add_child(b);

        assert_eq!(seq.get_child_by_index(0), Some(a));
        assert_eq!(seq.get_child_by_index(1), Some(b));
        assert_eq!(seq.get_child_by_index(-1), None);
        assert_eq!(seq.get_child_by_index(2), None);

        // Direct membership only — no arena to recurse into descendants.
        assert!(seq.has_child(a));
        assert!(!seq.has_child(SequenceId(3)));

        seq.remove_child(a);
        assert!(!seq.has_child(a));
        assert_eq!(seq.get_child_by_index(0), Some(b));
    }

    #[test]
    fn has_flag_masks_and_delete_clears_local_state() {
        let mut seq = Sequence::create();
        seq.m_flags = 0x3;
        assert_eq!(seq.has_flag(0x1), 0x1);
        assert_eq!(seq.has_flag(0x4), 0);

        seq.add_child(SequenceId(1));
        seq.push_command(block(1), PUSH_BACK);
        seq.delete();
        assert!(seq.m_children.is_empty());
        assert!(seq.m_commands.is_empty());
    }

    #[test]
    fn save_and_load_are_inert() {
        let mut seq = Sequence::create();
        assert_eq!(seq.save(), 0);
        assert_eq!(seq.load(), 0);
    }
}
