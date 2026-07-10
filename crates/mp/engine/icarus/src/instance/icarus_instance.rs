//! Raven `ICARUS_Instance` — the top ICARUS singleton and its owned arenas.

use std::collections::BTreeMap;

use crate::sequence::csequence::Sequence;
use crate::sequencer::csequencer::Sequencer;

/// Handle into [`IcarusInstance::sequences`] carrying Raven's monotonic
/// never-reused `m_GUID` (ICARUS-D3 / ruling 39d — declared beside its owning
/// arena, RMG `AreaId` §B5 precedent).
/// Source: `oracle/codemp/icarus/Instance.cpp:26,228`
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SequenceId(pub i32);

/// Handle into [`IcarusInstance::sequencers`] carrying Raven's monotonic
/// never-reused `m_GUID` (ICARUS-D3 / ruling 39d).
/// Source: `oracle/codemp/icarus/Instance.cpp:26,228`
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SequencerId(pub i32);

/// Raven `ICARUS_Instance` → `IcarusInstance` (§F idiomatic, ICARUS-D1 naming).
///
/// The top singleton. **Owns** its sequence/sequencer arenas as faithful `Vec`
/// arenas keyed by `SequenceId`/`SequencerId` (ICARUS-D3 / ruling 27); the
/// per-entity `gSequencers`/`gTaskManagers` arrays live on `Icarus`, not here.
/// The `interface_export_t *m_interface` back-ref is dropped (reached through
/// `Icarus.interface_export`, ruling 24). `Save`/`Load` are inert in MP
/// dedicated (Divergences).
/// Type definition source: `oracle/codemp/icarus/instance.h:12-79`
#[derive(Default)]
pub struct IcarusInstance {
    /// Raven `int m_GUID` — monotonic id source for the arenas.
    pub m_guid: i32,
    /// Raven `sequence_l m_sequences` — owned sequence arena.
    pub sequences: Vec<Sequence>,
    /// Raven `sequencer_l m_sequencers` — owned sequencer arena.
    pub sequencers: Vec<Sequencer>,
    /// Raven `signal_m m_signals` (`map<string, unsigned char>`).
    pub m_signals: BTreeMap<String, u8>,
    /// Parallel `m_guid`-drawn ids for `sequencers` — unlike `CSequence`,
    /// `CSequencer` carries no id field of its own in Raven (identity there
    /// is pointer-only, `STL_INSERT`, `Instance.cpp:174`), so this gives
    /// `SequencerId` a stable, never-reused handle to scan/remove by.
    sequencer_ids: Vec<i32>,
}

impl IcarusInstance {
    /// Raven `ICARUS_Instance::Create` — allocate the singleton and seed it.
    /// Source: `oracle/codemp/icarus/Instance.cpp:56-64` (`instance.h:26`)
    pub fn create() -> IcarusInstance {
        // Raven stores the `interface_export_t *ie` arg into `m_interface`;
        // that back-ref is dropped (ruling 24, reached via
        // `Icarus.interface_export` instead), so `Create` reduces to the
        // ctor's zero-init (`m_GUID = 0`, empty lists/map), which `Default`
        // already gives. The `OutputDebugString` banner has no Rust analog.
        IcarusInstance::default()
    }

    /// Raven `ICARUS_Instance::Free` (protected helper behind `Delete`).
    /// Source: `oracle/codemp/icarus/Instance.cpp:72-113` (`instance.h:57`)
    fn free(&mut self) -> i32 {
        // Raven also `memset`s the per-entity `gSequencers`/`gTaskManagers`
        // globals here; those live on `Icarus`, not this class (State
        // ownership table), so that reset is the caller's job, not this one.
        self.sequencers.clear();
        self.sequencer_ids.clear();
        self.m_signals.clear();
        self.sequences.clear();
        true as i32
    }

    /// Raven `ICARUS_Instance::Delete`.
    /// Source: `oracle/codemp/icarus/Instance.cpp:121-157` (`instance.h:27`)
    pub fn delete(&mut self) -> i32 {
        self.free();
        // The `#ifdef _DEBUG` counter dump has no analog in this release
        // build; `delete this` has no analog either — the caller owns this
        // object's lifetime through its `Option<IcarusInstance>` slot.
        true as i32
    }

    /// Raven `ICARUS_Instance::GetSequencer` — insert/return the sequencer for
    /// an entity id (`STL_INSERT( m_sequencers, … )`).
    /// Source: `oracle/codemp/icarus/Instance.cpp:166-182`
    pub fn get_sequencer(&mut self, owner_id: i32) -> SequencerId {
        let mut sequencer = Sequencer::create();
        sequencer.m_owner_id = owner_id;

        // `CSequencer` carries no id field of its own in Raven; draw one
        // from the shared `m_guid` counter (the same source `GetSequence`
        // draws from) to hand back a stable `SequencerId`.
        let id = self.m_guid;
        self.m_guid += 1;

        self.sequencers.push(sequencer);
        self.sequencer_ids.push(id);

        // Raven also builds and links a `CTaskManager` here
        // (`taskManager->Init(sequencer)`); per the State-ownership table,
        // task managers live on `Icarus.task_managers` (ent-indexed), not
        // this arena, so that construction is the caller's job (`ICARUS_InitEnt`).
        SequencerId(id)
    }

    /// Raven `ICARUS_Instance::DeleteSequencer`.
    /// Source: `oracle/codemp/icarus/Instance.cpp:191-215` (`instance.h:30`)
    pub fn delete_sequencer(&mut self, sequencer: SequencerId) {
        // Raven's `Recall()` needs `&mut Icarus`/`&mut dyn EngineHost`
        // (ruling 24) and its task-manager teardown needs the ent-indexed
        // `Icarus.task_managers` arena (ruling 27) — both outside this type,
        // so the caller performs them before removing the sequencer here.
        if let Some(pos) = self.sequencer_ids.iter().position(|&id| id == sequencer.0) {
            self.sequencer_ids.remove(pos);
            self.sequencers.remove(pos);
        }
    }

    /// Position of a `SequencerId` in the owning `sequencers` arena, or `None`.
    /// Used by the sequencer driver to `take`/`restore` a sequencer around a
    /// run/update (so `I_*` dispatch can hold `&mut Icarus` while the sequencer
    /// is a detached local — ruling 24's disjoint-borrow discipline).
    fn sequencer_pos(&self, sequencer: SequencerId) -> Option<usize> {
        self.sequencer_ids.iter().position(|&id| id == sequencer.0)
    }

    /// Detach a sequencer from the arena into an owned value, leaving a
    /// `Default` placeholder in its slot (the arena position is stable for the
    /// duration of a drive — no sequencers are added/removed mid-drive).
    pub fn take_sequencer(&mut self, sequencer: SequencerId) -> Option<Sequencer> {
        let pos = self.sequencer_pos(sequencer)?;
        Some(std::mem::take(&mut self.sequencers[pos]))
    }

    /// Restore a detached sequencer into its arena slot.
    pub fn restore_sequencer(&mut self, sequencer: SequencerId, value: Sequencer) {
        if let Some(pos) = self.sequencer_pos(sequencer) {
            self.sequencers[pos] = value;
        }
    }

    /// Raven `ICARUS_Instance::GetSequence()` — allocate a fresh sequence.
    /// Source: `oracle/codemp/icarus/Instance.cpp:223-240` (`instance.h:32`)
    pub fn get_sequence(&mut self) -> SequenceId {
        let mut sequence = Sequence::create();

        // Assign the GUID.
        let id = self.m_guid;
        self.m_guid += 1;
        sequence.m_id = id;
        // `SetOwner(this)` back-ref is dropped (ruling 24).

        self.sequences.push(sequence);

        SequenceId(id)
    }

    /// Raven `ICARUS_Instance::GetSequence( int id )` — faithful linear scan
    /// (insertion-ordered, NOT a keyed map — §A2 change declined, ruling 27).
    /// Source: `oracle/codemp/icarus/Instance.cpp:248-258`
    pub fn get_sequence_by_id(&self, id: SequenceId) -> Option<&Sequence> {
        self.sequences.iter().find(|sequence| sequence.m_id == id.0)
    }

    /// Raven `ICARUS_Instance::DeleteSequence`.
    /// Source: `oracle/codemp/icarus/Instance.cpp:266-277` (`instance.h:34`)
    pub fn delete_sequence(&mut self, sequence: SequenceId) {
        if let Some(pos) = self
            .sequences
            .iter()
            .position(|candidate| candidate.m_id == sequence.0)
        {
            self.sequences.remove(pos);
        }
    }

    /// Raven `ICARUS_Instance::Signal`.
    /// Source: `oracle/codemp/icarus/Instance.cpp:623-626` (`instance.h:42`)
    pub fn signal(&mut self, identifier: &str) {
        self.m_signals.insert(identifier.to_string(), 1);
    }

    /// Raven `ICARUS_Instance::CheckSignal`.
    /// Source: `oracle/codemp/icarus/Instance.cpp:634-644` (`instance.h:43`)
    pub fn check_signal(&self, identifier: &str) -> bool {
        self.m_signals.contains_key(identifier)
    }

    /// Raven `ICARUS_Instance::ClearSignal`.
    /// Source: `oracle/codemp/icarus/Instance.cpp:652-654` (`instance.h:44`)
    pub fn clear_signal(&mut self, identifier: &str) {
        self.m_signals.remove(identifier);
    }

    /// Raven `ICARUS_Instance::Save` — inert in MP dedicated (Divergences):
    /// its `I_WriteSaveData` targets are no-ops, and no host handle is
    /// threaded to this method, so the traversal has nothing to reach.
    /// Source: `oracle/codemp/icarus/Instance.cpp:422-443` (`instance.h:47`)
    pub fn save(&self) -> i32 {
        true as i32
    }

    /// Raven `ICARUS_Instance::Load` — inert in MP dedicated (Divergences),
    /// same as `save` above.
    /// Source: `oracle/codemp/icarus/Instance.cpp:575-615` (`instance.h:48`)
    pub fn load(&mut self) -> i32 {
        true as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_sequence_assigns_monotonic_never_reused_ids() {
        let mut instance = IcarusInstance::create();
        let a = instance.get_sequence();
        let b = instance.get_sequence();
        assert_eq!(a, SequenceId(0));
        assert_eq!(b, SequenceId(1));
        assert_eq!(instance.m_guid, 2);
    }

    #[test]
    fn get_sequence_by_id_is_a_linear_scan_by_id_not_position() {
        let mut instance = IcarusInstance::create();
        let a = instance.get_sequence();
        let b = instance.get_sequence();

        assert_eq!(instance.get_sequence_by_id(a).unwrap().m_id, a.0);
        assert_eq!(instance.get_sequence_by_id(b).unwrap().m_id, b.0);
        assert!(instance.get_sequence_by_id(SequenceId(99)).is_none());
    }

    #[test]
    fn delete_sequence_removes_only_the_matching_id() {
        let mut instance = IcarusInstance::create();
        let a = instance.get_sequence();
        let b = instance.get_sequence();

        instance.delete_sequence(a);

        assert!(instance.get_sequence_by_id(a).is_none());
        assert!(instance.get_sequence_by_id(b).is_some());
        assert_eq!(instance.sequences.len(), 1);

        // Deleting an id that isn't present is a faithful no-op (Raven's
        // `list::remove` of an absent pointer is also a no-op).
        instance.delete_sequence(a);
        assert_eq!(instance.sequences.len(), 1);
    }

    #[test]
    fn signal_check_and_clear_round_trip() {
        let mut instance = IcarusInstance::create();
        assert!(!instance.check_signal("door_open"));

        instance.signal("door_open");
        assert!(instance.check_signal("door_open"));

        instance.clear_signal("door_open");
        assert!(!instance.check_signal("door_open"));

        // Clearing an unset signal is a faithful no-op (`map::erase` of a
        // missing key).
        instance.clear_signal("door_open");
        assert!(!instance.check_signal("door_open"));
    }

    #[test]
    fn delete_clears_all_owned_arenas() {
        let mut instance = IcarusInstance::create();
        instance.get_sequence();
        instance.signal("some_signal");
        // Populate the sequencer arena directly to exercise `delete`'s clear.
        instance.sequencers.push(Sequencer::default());
        instance.sequencer_ids.push(0);

        assert_eq!(instance.delete(), 1);

        assert!(instance.sequences.is_empty());
        assert!(instance.sequencers.is_empty());
        assert!(instance.sequencer_ids.is_empty());
        assert!(instance.m_signals.is_empty());
    }

    #[test]
    fn save_and_load_are_inert_no_ops_that_report_success() {
        let mut instance = IcarusInstance::create();
        instance.get_sequence();
        assert_eq!(instance.save(), 1);
        assert_eq!(instance.load(), 1);
        // Inert: no traversal side effect on the arena.
        assert_eq!(instance.sequences.len(), 1);
    }
}
