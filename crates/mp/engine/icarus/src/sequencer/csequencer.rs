//! Raven `CSequencer` — the per-entity script driver.

use std::collections::BTreeMap;

use mp_host_interface::EngineHost;
use mp_qshared::shared::limits::MAX_GENTITIES;

use crate::blockstream::cblock::{
    member_c_string, member_float, peek_member_c_string, peek_member_float, Block,
};
use crate::blockstream::cblock_stream::BlockStream;
use crate::instance::icarus_instance::SequenceId;
use crate::sequence::csequence::Sequence;
use crate::sequencer::bstream_s::Bstream;
use crate::taskmanager::ctask_manager::{TaskGroupId, TaskManager};
use crate::Icarus;

/// Raven anonymous `enum { SEQ_OK, SEQ_FAILED }` — sequencer result codes.
/// Source: `oracle/codemp/icarus/sequencer.h:50-54`
pub const SEQ_OK: i32 = 0;
pub const SEQ_FAILED: i32 = 1;

/// Raven `enum { TASK_RETURN_COMPLETE, TASK_RETURN_FAILED }`.
/// Source: `oracle/codemp/icarus/taskmanager.h:17-21`
const TASK_RETURN_COMPLETE: i32 = 0;

// Sequence flags. Source: `oracle/codemp/icarus/sequencer.h:25-34`.
const SQ_COMMON: i32 = 0x0000_0000;
const SQ_RETAIN: i32 = 0x0000_0002;
const SQ_AFFECT: i32 = 0x0000_0004;
const SQ_RUN: i32 = 0x0000_0008;
const SQ_PENDING: i32 = 0x0000_0010;
const SQ_CONDITIONAL: i32 = 0x0000_0020;
const SQ_TASK: i32 = 0x0000_0040;
const SQ_LOOP: i32 = 0x0000_0001;
/// Raven `#define BF_ELSE 0x00000001` — block has an else id.
const BF_ELSE: u8 = 0x01;

// Push/pop side flags. Source: `oracle/codemp/icarus/blockstream.h:26-32`.
const POP_BACK: i32 = 1;
const PUSH_FRONT: i32 = 2;
const PUSH_BACK: i32 = 3;

// Block / member ids (interpreter.h enum resolved from `NUM_USER_TOKENS = 19`).
// Source: `oracle/codemp/icarus/interpreter.h:35-67`.
const ID_AFFECT: i32 = 19;
const ID_SOUND: i32 = 20;
const ID_MOVE: i32 = 21;
const ID_ROTATE: i32 = 22;
const ID_WAIT: i32 = 23;
const ID_BLOCK_END: i32 = 25;
const ID_SET: i32 = 26;
const ID_LOOP: i32 = 27;
const ID_PRINT: i32 = 29;
const ID_USE: i32 = 30;
const ID_FLUSH: i32 = 31;
const ID_RUN: i32 = 32;
const ID_KILL: i32 = 33;
const ID_REMOVE: i32 = 34;
const ID_CAMERA: i32 = 35;
const ID_GET: i32 = 36;
const ID_RANDOM: i32 = 37;
const ID_IF: i32 = 38;
const ID_ELSE: i32 = 39;
const ID_TASK: i32 = 41;
const ID_DO: i32 = 42;
const ID_DECLARE: i32 = 43;
const ID_FREE: i32 = 44;
const ID_SIGNAL: i32 = 46;
const ID_WAITSIGNAL: i32 = 47;
const ID_PLAY: i32 = 48;
const ID_TAG: i32 = 49;

// Token-type ids. Source: `oracle/codemp/icarus/tokenizer.h:63-75`,
// `interpreter.h:16-27`.
const TK_STRING: i32 = 4;
const TK_INT: i32 = 5;
const TK_FLOAT: i32 = 6;
const TK_IDENTIFIER: i32 = 7;
const TK_CHAR: i32 = 3;
const TK_VECTOR: i32 = 14;
const TK_GREATER_THAN: i32 = 15;
const TK_LESS_THAN: i32 = 16;
const TK_EQUALS: i32 = 17;
const TK_NOT: i32 = 18;

// Affect sub-type ids (`interpreter.h` type enum resolved from `NUM_IDS = 51`).
// Source: `oracle/codemp/icarus/interpreter.h:83-84`.
const TYPE_INSERT: i32 = 55;
const TYPE_FLUSH: i32 = 56;

/// Raven `WL_ERROR`/`WL_WARNING` print levels. The developer-gated `WL_DEBUG`
/// trace in `CheckRun` (`"%4d run(...)"`) is dropped — diagnostic-only, gated on
/// the `developer` cvar (0 in the goldens), so it reaches no state (Divergences).
/// Source: `oracle/codemp/game/q_shared.h:428-433`
const WL_ERROR: i32 = 1;
const WL_WARNING: i32 = 2;

/// Raven `#define MAX_STRING_SIZE 256`.
/// Source: `oracle/codemp/icarus/interpreter.h:8`
const _MAX_STRING_SIZE: usize = 256;

/// Raven `CSequencer` → `Sequencer` (§F idiomatic, ICARUS-D1 naming).
///
/// The per-entity script driver: parses IBI blocks into sequences. Per ICARUS-D3
/// (ruling 24) it holds **no** `m_ie`/`m_owner`/`m_taskManager` back-refs — the
/// `m_ie->I_*` dispatch sites become free fns re-indexing disjoint `Icarus`
/// field borrows per call (the owner instance and task manager are threaded in
/// explicitly during a drive). Member-storage per ruling 27: `m_sequences` →
/// `Vec<SequenceId>`, `m_taskSequences` → `BTreeMap<TaskGroupId, SequenceId>`,
/// `m_streamsCreated` → owned `Vec<Bstream>`.
/// Type definition source: `oracle/codemp/icarus/sequencer.h:68-187`
#[derive(Default)]
pub struct Sequencer {
    /// Raven `int m_ownerID`.
    pub m_owner_id: i32,
    /// Raven `int m_numCommands`.
    pub m_num_commands: i32,
    /// Raven `sequence_l m_sequences` — non-owning handles into the instance arena.
    pub m_sequences: Vec<SequenceId>,
    /// Raven `taskSequence_m m_taskSequences` (`map<CTaskGroup*, CSequence*>`).
    pub m_task_sequences: BTreeMap<TaskGroupId, SequenceId>,
    /// Raven `CSequence *m_curSequence`.
    pub m_cur_sequence: Option<SequenceId>,
    /// Raven `CTaskGroup *m_curGroup`.
    pub m_cur_group: Option<TaskGroupId>,
    /// Raven `bstream_t *m_curStream` — index into `m_streams_created`.
    pub m_cur_stream: Option<usize>,
    /// Raven `int m_elseValid`.
    pub m_else_valid: i32,
    /// Raven `CBlock *m_elseOwner` — owned here.
    pub m_else_owner: Option<Block>,
    /// Raven `vector<bstream_t*> m_streamsCreated` — owned stream-stack nodes.
    pub m_streams_created: Vec<Bstream>,
}

impl Sequencer {
    /// Raven `CSequencer::Create`.
    /// Source: `oracle/codemp/icarus/Sequencer.cpp:42-47` (`sequencer.h:77`)
    pub fn create() -> Sequencer {
        // Raven `new CSequencer` zero-inits every member (`operator new` =
        // `Z_Malloc(..., qtrue)`); `#[derive(Default)]` reproduces that.
        Sequencer::default()
    }

    /// Raven `CSequencer::Save` — inert in MP dedicated (Divergences).
    /// Source: `oracle/codemp/icarus/Sequencer.cpp:2343-2398` (`sequencer.h:90`)
    pub fn save(&self) -> i32 {
        // Raven's Save body is `#if 0`-compiled-out in this build; it falls
        // through to `return false;` — no save I/O in MP dedicated (Divergences).
        0
    }

    /// Raven `CSequencer::Load` — inert in MP dedicated (Divergences).
    /// Source: `oracle/codemp/icarus/Sequencer.cpp:2406-2483` (`sequencer.h:91`)
    pub fn load(&mut self) -> i32 {
        // Raven's Load body is `#if 0`-compiled-out; it falls through to
        // `return false;` — no load I/O in MP dedicated (Divergences).
        0
    }
}

// ===========================================================================
// Arena helpers — the sequences live in `IcarusInstance.sequences`; scalar
// field access is done through small scan helpers so no `&mut Sequence` is held
// across another arena borrow (ICARUS-D3 / ruling 27's linear-scan model).
// ===========================================================================

fn seq_ref(icarus: &Icarus, id: SequenceId) -> Option<&Sequence> {
    icarus
        .instance
        .as_ref()?
        .sequences
        .iter()
        .find(|s| s.m_id == id.0)
}

fn seq_mut(icarus: &mut Icarus, id: SequenceId) -> Option<&mut Sequence> {
    icarus
        .instance
        .as_mut()?
        .sequences
        .iter_mut()
        .find(|s| s.m_id == id.0)
}

/// `HasFlag(flag)` — masked flag word.
fn seq_flags(icarus: &Icarus, id: SequenceId, flag: i32) -> i32 {
    seq_ref(icarus, id).map_or(0, |s| s.m_flags & flag)
}

/// `GetFlags()` — the whole flag word.
fn seq_all_flags(icarus: &Icarus, id: SequenceId) -> i32 {
    seq_ref(icarus, id).map_or(0, |s| s.m_flags)
}

fn seq_or_flag(icarus: &mut Icarus, id: SequenceId, flag: i32) {
    if let Some(s) = seq_mut(icarus, id) {
        s.m_flags |= flag;
    }
}

fn seq_num_commands(icarus: &Icarus, id: SequenceId) -> i32 {
    seq_ref(icarus, id).map_or(0, |s| s.m_num_commands)
}

fn seq_num_children(icarus: &Icarus, id: SequenceId) -> i32 {
    seq_ref(icarus, id).map_or(0, |s| s.m_children.len() as i32)
}

fn seq_child_by_index(icarus: &Icarus, id: SequenceId, index: i32) -> Option<SequenceId> {
    seq_ref(icarus, id).and_then(|s| s.get_child_by_index(index))
}

fn seq_return(icarus: &Icarus, id: SequenceId) -> Option<SequenceId> {
    seq_ref(icarus, id).and_then(|s| s.m_return)
}

fn seq_parent(icarus: &Icarus, id: SequenceId) -> Option<SequenceId> {
    seq_ref(icarus, id).and_then(|s| s.m_parent)
}

fn seq_set_return(icarus: &mut Icarus, id: SequenceId, ret: Option<SequenceId>) {
    if let Some(s) = seq_mut(icarus, id) {
        s.m_return = ret;
    }
}

fn seq_iterations(icarus: &Icarus, id: SequenceId) -> i32 {
    seq_ref(icarus, id).map_or(0, |s| s.m_iterations)
}

fn seq_set_iterations(icarus: &mut Icarus, id: SequenceId, it: i32) {
    if let Some(s) = seq_mut(icarus, id) {
        s.m_iterations = it;
    }
}

fn seq_add_child(icarus: &mut Icarus, parent: SequenceId, child: SequenceId) {
    if let Some(s) = seq_mut(icarus, parent) {
        s.add_child(child);
    }
}

fn seq_remove_child(icarus: &mut Icarus, parent: SequenceId, child: SequenceId) {
    if let Some(s) = seq_mut(icarus, parent) {
        s.remove_child(child);
    }
}

/// Raven `CSequence::SetParent` — set the parent AND inherit its live
/// `SQ_RETAIN`/`SQ_PENDING` flags (`Sequence.cpp:149-161`). Done here, arena-aware,
/// because the `Sequence` node holds only ids (it cannot read a sibling's flags);
/// this restores the inheritance the arena-blind `Sequence::set_parent` drops.
fn seq_set_parent_inherit(icarus: &mut Icarus, id: SequenceId, parent: Option<SequenceId>) {
    if let Some(s) = seq_mut(icarus, id) {
        s.m_parent = parent;
    }
    let parent = match parent {
        Some(p) => p,
        None => return,
    };
    let pflags = seq_all_flags(icarus, parent);
    if pflags & SQ_RETAIN != 0 {
        seq_or_flag(icarus, id, SQ_RETAIN);
    }
    if pflags & SQ_PENDING != 0 {
        seq_or_flag(icarus, id, SQ_PENDING);
    }
}

/// Raven `CSequence::RemoveFlag( flag, children )` — arena-aware so the
/// `children` reflect can recurse into the subtree.
/// Source: `oracle/codemp/icarus/Sequence.cpp:257-269`
fn seq_remove_flag(icarus: &mut Icarus, id: SequenceId, flag: i32, children: bool) {
    if let Some(s) = seq_mut(icarus, id) {
        s.m_flags &= !flag;
    }
    if children {
        let kids = seq_ref(icarus, id)
            .map(|s| s.m_children.clone())
            .unwrap_or_default();
        for c in kids {
            seq_remove_flag(icarus, c, flag, true);
        }
    }
}

/// Raven `CSequence::HasChild` — direct-child membership plus recursion into
/// each child's own subtree (`Sequence.cpp:127-141`). Arena-aware, restoring the
/// descendant walk the arena-blind `Sequence::has_child` drops.
fn seq_has_child(icarus: &Icarus, root: SequenceId, target: SequenceId) -> bool {
    let children = match seq_ref(icarus, root) {
        Some(s) => s.m_children.clone(),
        None => return false,
    };
    for c in children {
        if c == target {
            return true;
        }
        if seq_has_child(icarus, c, target) {
            return true;
        }
    }
    false
}

// I_* dispatch shims (copy the fn ptr, then call with `&mut Icarus`).

fn i_dprintf(icarus: &mut Icarus, host: &mut dyn EngineHost, level: i32, msg: &str) {
    let f = icarus.interface_export.i_dprintf;
    f(icarus, host, level, msg);
}

// ===========================================================================
// Command stack (delegates to the current sequence in the arena + the
// sequencer's own running command count).
// ===========================================================================

/// Raven `CSequencer::PushCommand`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2170-2182`
fn push_command(seqr: &mut Sequencer, icarus: &mut Icarus, command: Block, flag: i32) -> i32 {
    let cur = match seqr.m_cur_sequence {
        Some(c) => c,
        None => return SEQ_FAILED,
    };
    if let Some(s) = seq_mut(icarus, cur) {
        s.push_command(command, flag);
    }
    seqr.m_num_commands += 1;
    SEQ_OK
}

/// Raven `CSequencer::PopCommand`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2192-2205`
fn pop_command(seqr: &mut Sequencer, icarus: &mut Icarus, flag: i32) -> Option<Block> {
    let cur = seqr.m_cur_sequence?;
    let block = seq_mut(icarus, cur).and_then(|s| s.pop_command(flag));
    if block.is_some() {
        seqr.m_num_commands -= 1;
    }
    block
}

// ===========================================================================
// Streams
// ===========================================================================

/// Raven `CSequencer::AddStream` — push a fresh stream node, linking `last` to
/// the current stream (`Sequencer.cpp:147-158`).
fn add_stream(seqr: &mut Sequencer) -> usize {
    let last = seqr.m_cur_stream;
    seqr.m_streams_created.push(Bstream {
        stream: crate::blockstream::cblock_stream::BlockStream::default(),
        last,
    });
    seqr.m_streams_created.len() - 1
}

/// Raven `CSequencer::DeleteStream` — free and remove a stream node
/// (`Sequencer.cpp:167-181`). The Vec-index fold shifts higher handles down on
/// removal, so `m_cur_stream` and every node's `last` are fixed up to stay valid
/// (Raven's heap pointers needed no such fixup).
fn delete_stream(seqr: &mut Sequencer, idx: usize) {
    if idx >= seqr.m_streams_created.len() {
        return;
    }
    seqr.m_streams_created[idx].stream.free();
    seqr.m_streams_created.remove(idx);

    let fix = |o: &mut Option<usize>| {
        if let Some(i) = o {
            if *i == idx {
                *o = None;
            } else if *i > idx {
                *i -= 1;
            }
        }
    };
    fix(&mut seqr.m_cur_stream);
    for b in seqr.m_streams_created.iter_mut() {
        fix(&mut b.last);
    }
}

// ===========================================================================
// Sequence allocation / lookup
// ===========================================================================

/// Raven `CSequencer::AddSequence()` — the no-arg allocator used by AddAffect.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:220-235`
fn add_sequence(seqr: &mut Sequencer, icarus: &mut Icarus) -> SequenceId {
    let id = icarus.instance.as_mut().unwrap().get_sequence();
    seqr.m_sequences.push(id);
    // Raven's own "temp fix" note here: flag the sequence pending.
    seq_or_flag(icarus, id, SQ_PENDING);
    id
}

/// Raven `CSequencer::AddSequence( parent, returnSeq, flags )`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:237-253`
fn add_sequence_full(
    seqr: &mut Sequencer,
    icarus: &mut Icarus,
    parent: Option<SequenceId>,
    return_seq: Option<SequenceId>,
    flags: i32,
) -> SequenceId {
    let id = icarus.instance.as_mut().unwrap().get_sequence();
    seqr.m_sequences.push(id);

    if let Some(s) = seq_mut(icarus, id) {
        s.m_flags = flags; // SetFlags
    }
    seq_set_parent_inherit(icarus, id, parent); // SetParent (+ flag inherit)
    seq_set_return(icarus, id, return_seq); // SetReturn
    id
}

/// Raven `CSequencer::GetSequence( int id )` — scan this sequencer's own list.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:263-282`
fn get_sequence(seqr: &Sequencer, id: i32) -> Option<SequenceId> {
    // `SequenceId.0 == m_id`, so the linear scan reduces to a membership test.
    seqr.m_sequences.iter().copied().find(|s| s.0 == id)
}

/// Raven `CSequencer::AddTaskSequence`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:189-192`
fn add_task_sequence(seqr: &mut Sequencer, sequence: SequenceId, group: TaskGroupId) {
    seqr.m_task_sequences.insert(group, sequence);
}

/// Raven `CSequencer::GetTaskSequence`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:200-210`
fn get_task_sequence(seqr: &Sequencer, group: TaskGroupId) -> Option<SequenceId> {
    seqr.m_task_sequences.get(&group).copied()
}

/// Raven `CSequencer::RemoveSequence` — drop references only (not the memory).
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2240-2266`
fn remove_sequence(icarus: &mut Icarus, host: &mut dyn EngineHost, sequence: SequenceId) -> i32 {
    let num_children = seq_num_children(icarus, sequence);
    for i in 0..num_children {
        match seq_child_by_index(icarus, sequence, i) {
            Some(temp) => {
                seq_set_parent_inherit(icarus, temp, None);
                seq_set_return(icarus, temp, None);
            }
            None => {
                i_dprintf(
                    icarus,
                    host,
                    WL_WARNING,
                    "Unable to find child sequence on RemoveSequence call!\n",
                );
            }
        }
    }
    SEQ_OK
}

/// Raven `CSequencer::DestroySequence` — remove all references and free.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2268-2311`
fn destroy_sequence(seqr: &mut Sequencer, icarus: &mut Icarus, sequence: SequenceId) -> i32 {
    seqr.m_sequences.retain(|&s| s != sequence);
    seqr.m_task_sequences.retain(|_, &mut v| v != sequence);

    if let Some(parent) = seq_parent(icarus, sequence) {
        seq_remove_child(icarus, parent, sequence);
    }

    let mut cur_child = seq_num_children(icarus, sequence);
    while cur_child > 0 {
        cur_child -= 1;
        if let Some(child) = seq_child_by_index(icarus, sequence, cur_child) {
            destroy_sequence(seqr, icarus, child);
        }
    }

    icarus.instance.as_mut().unwrap().delete_sequence(sequence);
    SEQ_OK
}

/// Raven `CSequencer::ReturnSequence` — climb `m_return` links to the next
/// sequence that still has pending commands.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2319-2333`
fn return_sequence(icarus: &Icarus, mut sequence: SequenceId) -> Option<SequenceId> {
    while let Some(ret) = seq_return(icarus, sequence) {
        if sequence == ret {
            return None;
        }
        sequence = ret;
        if seq_num_commands(icarus, sequence) > 0 {
            return Some(sequence);
        }
    }
    None
}

// ===========================================================================
// Public seam entry: Run
// ===========================================================================

/// Raven `CSequencer::Run` — runs a script (the `G_ICARUS_RUNSCRIPT` callee path).
/// Detaches the sequencer/task manager into locals so `I_*` dispatch can hold
/// `&mut Icarus` (ruling 24), then restores them.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:308-333` (`sequencer.h:80`)
pub fn run(icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, buffer: &[u8]) -> i32 {
    if owner_id < 0 || owner_id as usize >= MAX_GENTITIES {
        return SEQ_FAILED;
    }
    let sequencer_id = match icarus.sequencers[owner_id as usize] {
        Some(id) => id,
        None => return SEQ_FAILED,
    };
    let mut seqr = match icarus
        .instance
        .as_mut()
        .and_then(|inst| inst.take_sequencer(sequencer_id))
    {
        Some(s) => s,
        None => return SEQ_FAILED,
    };
    let mut tm = match icarus.task_managers[owner_id as usize].take() {
        Some(tm) => tm,
        None => {
            if let Some(inst) = icarus.instance.as_mut() {
                inst.restore_sequencer(sequencer_id, seqr);
            }
            return SEQ_FAILED;
        }
    };

    let ret = run_inner(&mut seqr, &mut tm, icarus, host, owner_id, buffer);

    icarus.task_managers[owner_id as usize] = Some(tm);
    if let Some(inst) = icarus.instance.as_mut() {
        inst.restore_sequencer(sequencer_id, seqr);
    }
    ret
}

fn run_inner(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    buffer: &[u8],
) -> i32 {
    recall(seqr, tm, icarus);

    // Create a new stream and open it as an IBI stream.
    let block_stream = add_stream(seqr);
    if seqr.m_streams_created[block_stream].stream.open(buffer) == 0 {
        i_dprintf(icarus, host, WL_ERROR, "invalid stream");
        return SEQ_FAILED;
    }

    let ret_seq = seqr.m_cur_sequence;
    let sequence = add_sequence_full(seqr, icarus, None, ret_seq, SQ_COMMON);

    // Interpret the command blocks and route them properly.
    if route(seqr, tm, icarus, host, owner_id, sequence, block_stream) != SEQ_OK {
        return SEQ_FAILED;
    }

    SEQ_OK
}

// ===========================================================================
// Route — the block dispatcher
// ===========================================================================

/// Raven `CSequencer::Route`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:787-949`
#[allow(clippy::too_many_arguments)]
fn route(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    sequence: SequenceId,
    bstream: usize,
) -> i32 {
    seqr.m_cur_stream = Some(bstream);
    seqr.m_cur_sequence = Some(sequence);

    // Obtain all blocks.
    while seqr.m_streams_created[bstream].stream.block_available() != 0 {
        let mut block = Block {
            m_members: Vec::new(),
            m_id: 0,
            m_flags: 0,
        };
        seqr.m_streams_created[bstream]
            .stream
            .read_block(&mut block);

        // TEMP: HACK! (Raven)
        if seqr.m_else_valid != 0 {
            seqr.m_else_valid -= 1;
        }

        match block.get_block_id() {
            // Marks the end of a blocked section.
            ID_BLOCK_END => {
                push_command(seqr, icarus, block, PUSH_FRONT);
                let cur = seqr.m_cur_sequence.unwrap();

                if seq_flags(icarus, cur, SQ_RUN) != 0 || seq_flags(icarus, cur, SQ_AFFECT) != 0 {
                    seqr.m_cur_stream = seqr.m_streams_created[bstream].last;
                }

                if seq_flags(icarus, cur, SQ_TASK) != 0 {
                    seqr.m_cur_stream = seqr.m_streams_created[bstream].last;
                    seqr.m_cur_group = seqr.m_cur_group.and_then(|g| {
                        tm.m_task_groups
                            .get(g.0 as usize)
                            .and_then(|grp| grp.m_parent)
                    });
                }

                seqr.m_cur_sequence = seq_return(icarus, cur);
                return SEQ_OK;
            }

            ID_AFFECT => {
                if parse_affect(seqr, tm, icarus, host, owner_id, block, bstream) != SEQ_OK {
                    return SEQ_FAILED;
                }
            }
            ID_RUN => {
                if parse_run(seqr, tm, icarus, host, owner_id, block) != SEQ_OK {
                    return SEQ_FAILED;
                }
            }
            ID_LOOP => {
                if parse_loop(seqr, tm, icarus, host, owner_id, block, bstream) != SEQ_OK {
                    return SEQ_FAILED;
                }
            }
            ID_IF => {
                if parse_if(seqr, tm, icarus, host, owner_id, block, bstream) != SEQ_OK {
                    return SEQ_FAILED;
                }
            }
            ID_ELSE => {
                if seqr.m_else_valid == 0 {
                    i_dprintf(icarus, host, WL_ERROR, "Invalid 'else' found!\n");
                    return SEQ_FAILED;
                }
                if parse_else(seqr, tm, icarus, host, owner_id, block, bstream) != SEQ_OK {
                    return SEQ_FAILED;
                }
            }
            ID_TASK => {
                if parse_task(seqr, tm, icarus, host, owner_id, block, bstream) != SEQ_OK {
                    return SEQ_FAILED;
                }
            }

            // Commands go directly into the sequence without pre-process.
            ID_WAIT | ID_PRINT | ID_SOUND | ID_MOVE | ID_ROTATE | ID_SET | ID_USE | ID_REMOVE
            | ID_KILL | ID_FLUSH | ID_CAMERA | ID_DO | ID_DECLARE | ID_FREE | ID_SIGNAL
            | ID_WAITSIGNAL | ID_PLAY => {
                push_command(seqr, icarus, block, PUSH_FRONT);
            }

            other => {
                i_dprintf(
                    icarus,
                    host,
                    WL_ERROR,
                    &format!("'{}' : invalid block ID", other),
                );
                return SEQ_FAILED;
            }
        }
    }

    let cur = seqr.m_cur_sequence.unwrap();

    // Check for a run sequence — it must be marked.
    if seq_flags(icarus, cur, SQ_RUN) != 0 {
        let mut block = Block {
            m_members: Vec::new(),
            m_id: 0,
            m_flags: 0,
        };
        block.create(ID_BLOCK_END);
        push_command(seqr, icarus, block, PUSH_FRONT); // mark the end of the run
        return SEQ_OK;
    }

    // Check to start the communication.
    if seqr.m_streams_created[bstream].last.is_none() && seqr.m_num_commands > 0 {
        let cmd = pop_command(seqr, icarus, POP_BACK);
        prime(seqr, tm, icarus, host, owner_id, cmd);
    }

    seqr.m_cur_stream = seqr.m_streams_created[bstream].last;

    // Free the stream.
    delete_stream(seqr, bstream);

    SEQ_OK
}

// ===========================================================================
// Parse* pre-processors
// ===========================================================================

/// Raven `CSequencer::ParseRun`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:344-400`
fn parse_run(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    mut block: Block,
) -> i32 {
    // Get the name and format it.
    let name0 = peek_member_c_string(&block, 0);
    let newname = BlockStream::strip_extension(&name0);

    // Get the file from the game engine (I_LoadFile).
    let load = icarus.interface_export.i_load_file;
    let buffer = load(icarus, host, &newname);
    let buffer = match buffer {
        Some(b) if !b.is_empty() => b,
        _ => {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                &format!("'{}' : could not open file\n", name0),
            );
            return SEQ_FAILED;
        }
    };

    // Create a new stream for this file and begin streaming.
    let new_stream = add_stream(seqr);
    if seqr.m_streams_created[new_stream].stream.open(&buffer) == 0 {
        i_dprintf(icarus, host, WL_ERROR, "invalid stream");
        return SEQ_FAILED;
    }

    let cur = seqr.m_cur_sequence.unwrap();
    let new_sequence = add_sequence_full(seqr, icarus, Some(cur), Some(cur), SQ_RUN | SQ_PENDING);
    seq_add_child(icarus, cur, new_sequence);

    if route(seqr, tm, icarus, host, owner_id, new_sequence, new_stream) != SEQ_OK {
        return SEQ_FAILED;
    }

    seqr.m_cur_sequence = seq_return(icarus, cur);

    block.write_float(TK_FLOAT, new_sequence.0 as f32);
    push_command(seqr, icarus, block, PUSH_FRONT);

    SEQ_OK
}

/// Raven `CSequencer::ParseIf`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:410-441`
#[allow(clippy::too_many_arguments)]
fn parse_if(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    mut block: Block,
    bstream: usize,
) -> i32 {
    let cur = seqr.m_cur_sequence.unwrap();
    let sequence = add_sequence_full(seqr, icarus, Some(cur), Some(cur), SQ_CONDITIONAL);
    seq_add_child(icarus, cur, sequence);

    // Add a unique conditional identifier for reference later.
    block.write_float(TK_FLOAT, sequence.0 as f32);

    // Push this to mark the conditional entrance — then it becomes m_elseOwner.
    push_command_capture_else(seqr, icarus, block);

    // Recursively obtain the conditional body.
    route(seqr, tm, icarus, host, owner_id, sequence, bstream);

    seqr.m_else_valid = 2;
    // m_elseOwner is the block we just pushed; the owner id was captured.
    SEQ_OK
}

/// Push a block onto the current sequence and record it as `m_elseOwner`.
///
/// Raven keeps a raw `CBlock *m_elseOwner` alias to the just-pushed block so a
/// later `ParseElse` can `Write`/`SetFlag(BF_ELSE)` it in place. With owned
/// blocks there is no alias, so ParseElse instead patches the block *in the
/// current sequence's command list* (see `parse_else`); this records only that
/// an else is expected. The `m_else_owner` field holds no block in this fold.
fn push_command_capture_else(seqr: &mut Sequencer, icarus: &mut Icarus, block: Block) {
    push_command(seqr, icarus, block, PUSH_FRONT);
}

/// Raven `CSequencer::ParseElse`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:451-490`
#[allow(clippy::too_many_arguments)]
fn parse_else(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    _block: Block, // the else block is not retained (Raven `delete block`)
    bstream: usize,
) -> i32 {
    let cur = seqr.m_cur_sequence.unwrap();
    let sequence = add_sequence_full(seqr, icarus, Some(cur), Some(cur), SQ_CONDITIONAL);
    seq_add_child(icarus, cur, sequence);

    // Patch the pending if-block (the front command of the current sequence)
    // with the else's success id + BF_ELSE flag (Raven's `m_elseOwner` writes).
    if let Some(s) = seq_mut(icarus, cur) {
        if let Some(owner_block) = s.m_commands.first_mut() {
            owner_block.write_float(TK_FLOAT, sequence.0 as f32);
            owner_block.set_flag(BF_ELSE);
        } else {
            i_dprintf(icarus, host, WL_ERROR, "Invalid 'else' found!\n");
            return SEQ_FAILED;
        }
    }

    route(seqr, tm, icarus, host, owner_id, sequence, bstream);

    seqr.m_else_valid = 0;
    SEQ_OK
}

/// Raven `CSequencer::ParseLoop`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:500-550`
#[allow(clippy::too_many_arguments)]
fn parse_loop(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    mut block: Block,
    bstream: usize,
) -> i32 {
    let cur = seqr.m_cur_sequence.unwrap();
    let sequence = add_sequence_full(seqr, icarus, Some(cur), Some(cur), SQ_LOOP | SQ_RETAIN);
    seq_add_child(icarus, cur, sequence);

    // Set the number of iterations of this sequence.
    let mut member_num = 0;
    let bm0_id = block.get_member(member_num).map(|m| m.m_id).unwrap_or(-1);
    member_num += 1;

    if bm0_id == ID_RANDOM {
        let min = member_float(&block, &mut member_num);
        let max = member_float(&block, &mut member_num);
        let f = icarus.interface_export.i_random;
        let riter = f(icarus, host, min, max) as i32;
        seq_set_iterations(icarus, sequence, riter);
    } else {
        let it = peek_member_float(&block, 0) as i32;
        seq_set_iterations(icarus, sequence, it);
    }

    block.write_float(TK_FLOAT, sequence.0 as f32);
    push_command(seqr, icarus, block, PUSH_FRONT);

    route(seqr, tm, icarus, host, owner_id, sequence, bstream);

    SEQ_OK
}

/// Raven `CSequencer::AddAffect`.
///
/// Raven aliases the caller's `CBlockStream` into a stack `bstream_t` so the
/// affected entity's sequence reads from the same cursor. With owned streams
/// there is no alias: the remaining bytes are cloned into a new stream node on
/// this sequencer (faithful for the common self-affect path; cross-entity affect
/// through a different `gSequencers[]` entry is a golden-unreachable limitation).
/// Source: `oracle/codemp/icarus/Sequencer.cpp:560-587`
fn add_affect(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    source_stream: usize,
    retain: i32,
) -> (i32, i32) {
    let sequence = add_sequence(seqr, icarus);
    seq_or_flag(icarus, sequence, SQ_AFFECT | SQ_PENDING);
    if retain != 0 {
        seq_or_flag(icarus, sequence, SQ_RETAIN);
    }

    // Restore the route state via the return link.
    let back = seqr.m_cur_sequence;
    seq_set_return(icarus, sequence, back);

    // Clone the source stream's remaining state into a temp node.
    let new_stream = add_stream(seqr);
    let cloned = {
        let src = &seqr.m_streams_created[source_stream].stream;
        crate::blockstream::cblock_stream::BlockStream {
            m_file_size: src.m_file_size,
            m_file_name: src.m_file_name.clone(),
            m_stream: src.m_stream.clone(),
            m_stream_pos: src.m_stream_pos,
        }
    };
    seqr.m_streams_created[new_stream].stream = cloned;
    seqr.m_streams_created[new_stream].last = seqr.m_cur_stream;

    if route(seqr, tm, icarus, host, owner_id, sequence, new_stream) != SEQ_OK {
        return (SEQ_FAILED, sequence.0);
    }

    seq_set_return(icarus, sequence, None);
    (SEQ_OK, sequence.0)
}

/// Raven `CSequencer::ParseAffect`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:597-726`
#[allow(clippy::too_many_arguments)]
fn parse_affect(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    mut block: Block,
    bstream: usize,
) -> i32 {
    let entname = peek_member_c_string(&block, 0);
    let get_ent = icarus.interface_export.i_get_entity_by_name;
    let mut ent = get_ent(icarus, host, &entname);

    if ent.is_null() {
        // Try to parse an embedded 'get' command.
        let bm0_id = block.get_member(0).map(|m| m.m_id).unwrap_or(-1);
        let p1: String = match bm0_id {
            TK_STRING | TK_IDENTIFIER | TK_CHAR => peek_member_c_string(&block, 0),
            ID_GET => {
                let type_ = peek_member_float(&block, 1) as i32;
                let name = peek_member_c_string(&block, 2);
                if type_ == TK_STRING {
                    let gs = icarus.interface_export.i_get_string;
                    match gs(icarus, host, owner_id, type_, &name) {
                        Some(s) => s,
                        None => return SEQ_FAILED, // Raven `return false`
                    }
                } else {
                    i_dprintf(
                        icarus,
                        host,
                        WL_ERROR,
                        "Invalid parameter type on affect _1",
                    );
                    return SEQ_FAILED;
                }
            }
            _ => {
                i_dprintf(
                    icarus,
                    host,
                    WL_ERROR,
                    "Invalid parameter type on affect _2",
                );
                return SEQ_FAILED;
            }
        };
        let ge = icarus.interface_export.i_get_entity_by_name;
        ent = ge(icarus, host, &p1);
        if ent.is_null() {
            i_dprintf(icarus, host, WL_WARNING, "invalid affect() target\n");
        }
    }

    let target_ent = if ent.is_null() {
        None
    } else {
        Some(unsafe { (*ent).s.number })
    };

    // NOTENOTE: Raven fetches `gSequencers[ent->s.number]`. If the target is this
    // same owner (self-affect) we already hold its sequencer; a different entity
    // is the golden-unreachable cross-sequencer path (see `add_affect`).
    let target_is_self = target_ent == Some(owner_id);

    if !target_is_self {
        // No reachable target sequencer: fast-forward past this affect block.
        i_dprintf(icarus, host, WL_WARNING, "invalid affect() target\n");
        let back_seq = seqr.m_cur_sequence;
        let trash = icarus.instance.as_mut().unwrap().get_sequence();
        seqr.m_cur_sequence = Some(trash);
        route(seqr, tm, icarus, host, owner_id, trash, bstream);
        recall(seqr, tm, icarus);
        destroy_sequence(seqr, icarus, trash);
        seqr.m_cur_sequence = back_seq;
        return SEQ_OK;
    }

    let retain = seq_flags(icarus, seqr.m_cur_sequence.unwrap(), SQ_RETAIN);
    let (res, ret) = add_affect(seqr, tm, icarus, host, owner_id, bstream, retain);
    if res != SEQ_OK {
        return SEQ_FAILED;
    }

    block.write_float(TK_FLOAT, ret as f32);
    push_command(seqr, icarus, block, PUSH_FRONT);
    SEQ_OK
}

/// Raven `CSequencer::ParseTask`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:734-773`
#[allow(clippy::too_many_arguments)]
fn parse_task(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    block: Block,
    bstream: usize,
) -> i32 {
    let cur = seqr.m_cur_sequence.unwrap();
    let sequence = add_sequence_full(seqr, icarus, Some(cur), Some(cur), SQ_TASK | SQ_RETAIN);
    seq_add_child(icarus, cur, sequence);

    let task_name = peek_member_c_string(&block, 0);
    let group = tm.add_task_group(&task_name);

    // The current group becomes this group; subsequent commands fall into it.
    tm.m_task_groups[group.0 as usize].m_parent = seqr.m_cur_group;
    seqr.m_cur_group = Some(group);

    add_task_sequence(seqr, sequence, group);

    // block is not retained (Raven `delete block`).
    drop(block);

    route(seqr, tm, icarus, host, owner_id, sequence, bstream);
    SEQ_OK
}

// ===========================================================================
// Check* pre-processors (Prep)
// ===========================================================================

/// Raven `CSequencer::Prep` — run all pre-processors on the popped command.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:1987-1996`
fn prep(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: &mut Option<Block>,
) {
    check_affect(seqr, tm, icarus, host, owner_id, command);
    check_flush(seqr, tm, icarus, host, owner_id, command);
    check_loop(seqr, tm, icarus, host, owner_id, command);
    check_run(seqr, tm, icarus, host, owner_id, command);
    check_if(seqr, tm, icarus, host, owner_id, command);
    check_do(seqr, tm, icarus, host, owner_id, command);
}

/// Raven `CSequencer::Prime`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2006-2016`
fn prime(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: Option<Block>,
) -> i32 {
    let mut command = command;
    prep(seqr, tm, icarus, host, owner_id, &mut command);
    if let Some(cmd) = command {
        tm.set_command(cmd, PUSH_BACK);
    }
    SEQ_OK
}

/// Raven `CSequencer::CheckRun`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:961-1035`
fn check_run(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: &mut Option<Block>,
) {
    let block_id = match command.as_ref() {
        Some(b) => b.get_block_id(),
        None => return,
    };

    if block_id == ID_RUN {
        let id = peek_member_float(command.as_ref().unwrap(), 1) as i32;
        let cur = seqr.m_cur_sequence.unwrap();

        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None; // delete block
        }

        seqr.m_cur_sequence = get_sequence(seqr, id);
        let cur = match seqr.m_cur_sequence {
            Some(c) => c,
            None => {
                i_dprintf(icarus, host, WL_ERROR, "Unable to find 'run' sequence!\n");
                *command = None;
                return;
            }
        };

        if seq_num_commands(icarus, cur) > 0 {
            *command = pop_command(seqr, icarus, POP_BACK);
            prep(seqr, tm, icarus, host, owner_id, command);
        }
        return;
    }

    let cur = seqr.m_cur_sequence.unwrap();
    if block_id == ID_BLOCK_END && seq_flags(icarus, cur, SQ_RUN) != 0 {
        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        seqr.m_cur_sequence = return_sequence(icarus, cur);

        if let Some(new_cur) = seqr.m_cur_sequence {
            if seq_num_commands(icarus, new_cur) > 0 {
                *command = pop_command(seqr, icarus, POP_BACK);
                prep(seqr, tm, icarus, host, owner_id, command);
            }
        }
    }
}

/// Raven `CSequencer::CheckLoop`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:1539-1667`
fn check_loop(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: &mut Option<Block>,
) {
    let block_id = match command.as_ref() {
        Some(b) => b.get_block_id(),
        None => return,
    };

    if block_id == ID_LOOP {
        let blk = command.as_ref().unwrap();
        let mut member_num = 0;
        let bm0_id = blk.get_member(member_num).map(|m| m.m_id).unwrap_or(-1);
        member_num += 1;

        let iterations = if bm0_id == ID_RANDOM {
            let min = member_float(blk, &mut member_num);
            let max = member_float(blk, &mut member_num);
            let f = icarus.interface_export.i_random;
            f(icarus, host, min, max) as i32
        } else {
            peek_member_float(blk, 0) as i32
        };

        let loop_id = member_float(command.as_ref().unwrap(), &mut member_num) as i32;
        let loop_seq = match get_sequence(seqr, loop_id) {
            Some(l) => l,
            None => {
                i_dprintf(icarus, host, WL_ERROR, "Unable to find 'loop' sequence!\n");
                *command = None;
                return;
            }
        };

        if seq_parent(icarus, loop_seq).is_none() {
            *command = None;
            return;
        }

        seq_set_iterations(icarus, loop_seq, iterations);

        let cur = seqr.m_cur_sequence.unwrap();
        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        seqr.m_cur_sequence = Some(loop_seq);
        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);
        return;
    }

    let cur = seqr.m_cur_sequence.unwrap();
    if block_id == ID_BLOCK_END && seq_flags(icarus, cur, SQ_LOOP) != 0 {
        if seq_iterations(icarus, cur) > 0 {
            seq_set_iterations(icarus, cur, seq_iterations(icarus, cur) - 1);
        }

        if seq_iterations(icarus, cur) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
            *command = pop_command(seqr, icarus, POP_BACK);
            prep(seqr, tm, icarus, host, owner_id, command);
        } else {
            if seq_return(icarus, cur).is_none() {
                *command = None;
                return;
            }
            let parent = seq_parent(icarus, cur);
            let parent_retain = parent.map_or(0, |p| seq_flags(icarus, p, SQ_RETAIN));
            if parent_retain != 0 {
                let block = command.take().unwrap();
                push_command(seqr, icarus, block, PUSH_FRONT);
            } else {
                *command = None;
            }

            seqr.m_cur_sequence = return_sequence(icarus, cur);
            if seqr.m_cur_sequence.is_none() {
                *command = None;
                return;
            }
            *command = pop_command(seqr, icarus, POP_BACK);
            prep(seqr, tm, icarus, host, owner_id, command);
        }
    }
}

/// Raven `CSequencer::CheckFlush`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:1677-1707`
fn check_flush(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: &mut Option<Block>,
) {
    let block_id = match command.as_ref() {
        Some(b) => b.get_block_id(),
        None => return,
    };

    if block_id == ID_FLUSH {
        let cur = seqr.m_cur_sequence.unwrap();
        flush(seqr, tm, icarus, host, owner_id, cur);

        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);
    }
}

/// Raven `CSequencer::CheckAffect`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:1717-1877`
fn check_affect(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: &mut Option<Block>,
) {
    let block_id = match command.as_ref() {
        Some(b) => b.get_block_id(),
        None => return,
    };

    if block_id == ID_AFFECT {
        let blk = command.as_ref().unwrap();
        let mut member_num = 0;
        let entname = peek_member_c_string(blk, member_num);
        member_num += 1;
        let ge = icarus.interface_export.i_get_entity_by_name;
        let mut ent = ge(icarus, host, &entname);

        if ent.is_null() {
            let bm0_id = command
                .as_ref()
                .unwrap()
                .get_member(0)
                .map(|m| m.m_id)
                .unwrap_or(-1);
            let p1: String = match bm0_id {
                TK_STRING | TK_IDENTIFIER | TK_CHAR => {
                    peek_member_c_string(command.as_ref().unwrap(), 0)
                }
                ID_GET => {
                    let type_ = peek_member_float(command.as_ref().unwrap(), member_num) as i32;
                    let name = peek_member_c_string(command.as_ref().unwrap(), member_num + 1);
                    member_num += 2;
                    if type_ == TK_STRING {
                        let gs = icarus.interface_export.i_get_string;
                        match gs(icarus, host, owner_id, type_, &name) {
                            Some(s) => s,
                            None => return,
                        }
                    } else {
                        i_dprintf(
                            icarus,
                            host,
                            WL_ERROR,
                            "Invalid parameter type on affect _1",
                        );
                        return;
                    }
                }
                _ => {
                    i_dprintf(
                        icarus,
                        host,
                        WL_ERROR,
                        "Invalid parameter type on affect _2",
                    );
                    return;
                }
            };
            let ge2 = icarus.interface_export.i_get_entity_by_name;
            ent = ge2(icarus, host, &p1);
            if ent.is_null() {
                i_dprintf(icarus, host, WL_WARNING, "invalid affect() target\n");
            }
        }

        let target_ent = if ent.is_null() {
            None
        } else {
            Some(unsafe { (*ent).s.number })
        };
        if member_num == 0 {
            member_num += 1;
        }
        let type_ = peek_member_float(command.as_ref().unwrap(), member_num) as i32;
        let id = peek_member_float(command.as_ref().unwrap(), member_num + 1) as i32;

        let cur = seqr.m_cur_sequence.unwrap();
        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        // Only self-affect reaches the running sequencer (see `add_affect`).
        if target_ent != Some(owner_id) {
            *command = pop_command(seqr, icarus, POP_BACK);
            prep(seqr, tm, icarus, host, owner_id, command);
            return;
        }

        affect(seqr, tm, icarus, host, owner_id, id, type_);

        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);

        // `gTaskManagers[ent->s.number]->Update()` on self is the same manager we
        // are inside — re-entrant Update is skipped (Raven would recurse into it).
        return;
    }

    let cur = seqr.m_cur_sequence.unwrap();
    if block_id == ID_BLOCK_END && seq_flags(icarus, cur, SQ_AFFECT) != 0 {
        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        seqr.m_cur_sequence = return_sequence(icarus, cur);
        if seqr.m_cur_sequence.is_none() {
            *command = None;
            return;
        }

        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);
    }
}

/// Raven `CSequencer::CheckDo`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:1885-1977`
fn check_do(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: &mut Option<Block>,
) {
    let block_id = match command.as_ref() {
        Some(b) => b.get_block_id(),
        None => return,
    };

    if block_id == ID_DO {
        let group_name = peek_member_c_string(command.as_ref().unwrap(), 0);
        let group = tm.get_task_group(&group_name);
        let sequence = group.and_then(|g| get_task_sequence(seqr, g));

        let group = match group {
            Some(g) => g,
            None => {
                i_dprintf(
                    icarus,
                    host,
                    WL_ERROR,
                    &format!("ICARUS Unable to find task group \"{}\"!\n", group_name),
                );
                *command = None;
                return;
            }
        };
        let sequence = match sequence {
            Some(s) => s,
            None => {
                i_dprintf(
                    icarus,
                    host,
                    WL_ERROR,
                    "ICARUS Unable to find task 'group' sequence!\n",
                );
                *command = None;
                return;
            }
        };

        let cur = seqr.m_cur_sequence.unwrap();
        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        seq_set_return(icarus, sequence, Some(cur));
        seqr.m_cur_sequence = Some(sequence);

        tm.m_task_groups[group.0 as usize].m_parent = seqr.m_cur_group;
        seqr.m_cur_group = Some(group);

        let guid = tm.m_task_groups[group.0 as usize].m_guid;
        tm.mark_task(guid, 2 /* TASK_START */);

        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);
        return;
    }

    let cur = seqr.m_cur_sequence.unwrap();
    if block_id == ID_BLOCK_END && seq_flags(icarus, cur, SQ_TASK) != 0 {
        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        if let Some(g) = seqr.m_cur_group {
            let guid = tm.m_task_groups[g.0 as usize].m_guid;
            tm.mark_task(guid, 3 /* TASK_END */);
            seqr.m_cur_group = tm.m_task_groups[g.0 as usize].m_parent;
        }

        let return_seq = return_sequence(icarus, cur);
        seq_set_return(icarus, cur, None);
        seqr.m_cur_sequence = return_seq;

        if seqr.m_cur_sequence.is_none() {
            *command = None;
            return;
        }

        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);
    }
}

/// Raven `CSequencer::CheckIf`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:1385-1529`
fn check_if(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    command: &mut Option<Block>,
) {
    let block_id = match command.as_ref() {
        Some(b) => b.get_block_id(),
        None => return,
    };

    if block_id == ID_IF {
        let ret = evaluate_conditional(icarus, host, owner_id, command.as_ref().unwrap());
        let blk = command.as_ref().unwrap();
        let num_members = blk.get_num_members();
        let has_else = blk.has_flag(BF_ELSE) != 0;

        if ret != 0 {
            let success_id = if has_else {
                peek_member_float(blk, num_members - 2) as i32
            } else {
                peek_member_float(blk, num_members - 1) as i32
            };
            let success_seq = match get_sequence(seqr, success_id) {
                Some(s) => s,
                None => {
                    i_dprintf(
                        icarus,
                        host,
                        WL_ERROR,
                        "Unable to find conditional success sequence!\n",
                    );
                    *command = None;
                    return;
                }
            };
            retain_or_drop(seqr, icarus, command);
            seqr.m_cur_sequence = Some(success_seq);
            *command = pop_command(seqr, icarus, POP_BACK);
            prep(seqr, tm, icarus, host, owner_id, command);
            return;
        }

        if ret == 0 && has_else {
            let failure_id = peek_member_float(blk, num_members - 1) as i32;
            let failure_seq = match get_sequence(seqr, failure_id) {
                Some(s) => s,
                None => {
                    i_dprintf(
                        icarus,
                        host,
                        WL_ERROR,
                        "Unable to find conditional failure sequence!\n",
                    );
                    *command = None;
                    return;
                }
            };
            retain_or_drop(seqr, icarus, command);
            seqr.m_cur_sequence = Some(failure_seq);
            *command = pop_command(seqr, icarus, POP_BACK);
            prep(seqr, tm, icarus, host, owner_id, command);
            return;
        }

        // Conditional failed with no else: move on to the next command.
        retain_or_drop(seqr, icarus, command);
        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);
        return;
    }

    let cur = seqr.m_cur_sequence.unwrap();
    if block_id == ID_BLOCK_END && seq_flags(icarus, cur, SQ_CONDITIONAL) != 0 {
        if seq_return(icarus, cur).is_none() {
            *command = None;
            return;
        }

        let parent = seq_parent(icarus, cur);
        let parent_retain = parent.map_or(0, |p| seq_flags(icarus, p, SQ_RETAIN));
        if parent_retain != 0 {
            let block = command.take().unwrap();
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            *command = None;
        }

        seqr.m_cur_sequence = return_sequence(icarus, cur);
        if seqr.m_cur_sequence.is_none() {
            *command = None;
            return;
        }

        *command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, command);
    }
}

/// Shared "retain the conditional statement iff the calling sequence is
/// retained, else drop it" tail (CheckIf's repeated block).
fn retain_or_drop(seqr: &mut Sequencer, icarus: &mut Icarus, command: &mut Option<Block>) {
    let cur = seqr.m_cur_sequence.unwrap();
    if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
        if let Some(block) = command.take() {
            push_command(seqr, icarus, block, PUSH_FRONT);
        }
    } else {
        *command = None;
    }
}

// ===========================================================================
// EvaluateConditional
// ===========================================================================

/// Raven `CSequencer::EvaluateConditional`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:1045-1375`
fn evaluate_conditional(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    block: &Block,
) -> i32 {
    let mut member_num = 0;

    let (t1, p1) = match eval_operand(icarus, host, owner_id, block, &mut member_num) {
        Some(v) => v,
        None => return 0,
    };

    // Comparison operator.
    let oper_id = block.get_member(member_num).map(|m| m.m_id).unwrap_or(-1);
    member_num += 1;
    let oper = match oper_id {
        TK_EQUALS | TK_GREATER_THAN | TK_LESS_THAN | TK_NOT => oper_id,
        _ => {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                "Invalid operator type found on conditional!\n",
            );
            return 0;
        }
    };

    let (t2, p2) = match eval_operand(icarus, host, owner_id, block, &mut member_num) {
        Some(v) => v,
        None => return 0,
    };

    let f = icarus.interface_export.i_evaluate;
    f(icarus, host, t1, &p1, t2, &p2, oper)
}

/// Read one conditional operand — folds the two near-identical operand switches
/// in `EvaluateConditional` (the first/second parameter) into a single helper.
/// Returns `(type, formatted-string)`.
fn eval_operand(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    block: &Block,
    member_num: &mut i32,
) -> Option<(i32, String)> {
    let id = block.get_member(*member_num).map(|m| m.m_id).unwrap_or(-1);
    *member_num += 1;

    match id {
        TK_FLOAT => {
            let f = peek_member_float(block, *member_num - 1);
            Some((TK_FLOAT, format!("{:.3}", f)))
        }
        TK_VECTOR => {
            let mut vec = [0.0f32; 3];
            for v in vec.iter_mut() {
                *v = member_float(block, member_num);
            }
            Some((
                TK_VECTOR,
                format!("{:.3} {:.3} {:.3}", vec[0], vec[1], vec[2]),
            ))
        }
        TK_STRING | TK_IDENTIFIER | TK_CHAR => {
            Some((id, peek_member_c_string(block, *member_num - 1)))
        }
        ID_GET => {
            let type_ = member_float(block, member_num) as i32;
            let name = member_c_string(block, member_num);
            match type_ {
                TK_FLOAT => {
                    let mut fval = 0.0f32;
                    let g = icarus.interface_export.i_get_float;
                    if g(icarus, host, owner_id, type_, &name, &mut fval) == 0 {
                        return None;
                    }
                    Some((TK_FLOAT, format!("{:.3}", fval)))
                }
                TK_INT => {
                    let mut fval = 0.0f32;
                    let g = icarus.interface_export.i_get_float;
                    if g(icarus, host, owner_id, type_, &name, &mut fval) == 0 {
                        return None;
                    }
                    Some((TK_INT, format!("{}", fval as i32)))
                }
                TK_STRING => {
                    let g = icarus.interface_export.i_get_string;
                    let s = g(icarus, host, owner_id, type_, &name)?;
                    Some((type_, s))
                }
                TK_VECTOR => {
                    let mut vval = [0.0f32; 3];
                    let g = icarus.interface_export.i_get_vector;
                    if g(icarus, host, owner_id, type_, &name, &mut vval) == 0 {
                        return None;
                    }
                    Some((
                        type_,
                        format!("{:.3} {:.3} {:.3}", vval[0], vval[1], vval[2]),
                    ))
                }
                _ => Some((type_, String::new())),
            }
        }
        ID_RANDOM => {
            let min = member_float(block, member_num);
            let max = member_float(block, member_num);
            let g = icarus.interface_export.i_random;
            let r = g(icarus, host, min, max);
            Some((TK_FLOAT, format!("{:.3}", r)))
        }
        ID_TAG => {
            let name = member_c_string(block, member_num);
            let type_ = member_float(block, member_num);
            let mut vec = [0.0f32; 3];
            let g = icarus.interface_export.i_get_tag;
            if g(icarus, host, owner_id, &name, type_ as i32, &mut vec) == 0 {
                i_dprintf(
                    icarus,
                    host,
                    WL_ERROR,
                    &format!("Unable to find tag \"{}\"!\n", name),
                );
                return None;
            }
            Some((
                TK_VECTOR,
                format!("{:.3} {:.3} {:.3}", vec[0], vec[1], vec[2]),
            ))
        }
        _ => {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                "Invalid parameter type on conditional",
            );
            None
        }
    }
}

// ===========================================================================
// Flush / Interrupt / Affect / Callback / Recall
// ===========================================================================

/// Raven `CSequencer::Flush`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:106-137`
fn flush(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    _owner_id: i32,
    owner: SequenceId,
) -> i32 {
    recall(seqr, tm, icarus);

    let mut kept = Vec::with_capacity(seqr.m_sequences.len());
    let seqs = seqr.m_sequences.clone();
    for sli in seqs {
        if sli == owner
            || seq_has_child(icarus, owner, sli)
            || seq_flags(icarus, sli, SQ_PENDING) != 0
            || seq_flags(icarus, sli, SQ_TASK) != 0
        {
            kept.push(sli);
            continue;
        }
        remove_sequence(icarus, host, sli);
        icarus.instance.as_mut().unwrap().delete_sequence(sli);
    }
    seqr.m_sequences = kept;

    // The owner is now the root sequence.
    seq_set_parent_inherit(icarus, owner, None);
    seq_set_return(icarus, owner, None);

    SEQ_OK
}

/// Raven `CSequencer::Interrupt` — save the current task's block back onto the
/// sequence (`Sequencer.cpp:290-299`). `taskManager` is threaded in.
#[allow(dead_code)]
fn interrupt(seqr: &mut Sequencer, tm: &mut TaskManager, icarus: &mut Icarus) {
    if let Some(command) = tm.get_current_task() {
        push_command(seqr, icarus, command, PUSH_BACK);
    }
}

/// Raven `CSequencer::Affect`.
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2114-2160`
fn affect(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    id: i32,
    type_: i32,
) -> i32 {
    let sequence = match get_sequence(seqr, id) {
        Some(s) => s,
        None => return SEQ_FAILED,
    };

    match type_ {
        TYPE_FLUSH => {
            flush(seqr, tm, icarus, host, owner_id, sequence);
            seq_remove_flag(icarus, sequence, SQ_PENDING, true);
            seqr.m_cur_sequence = Some(sequence);
            let cmd = pop_command(seqr, icarus, POP_BACK);
            prime(seqr, tm, icarus, host, owner_id, cmd);
        }
        TYPE_INSERT => {
            recall(seqr, tm, icarus);
            seq_set_return(icarus, sequence, seqr.m_cur_sequence);
            seq_remove_flag(icarus, sequence, SQ_PENDING, true);
            seqr.m_cur_sequence = Some(sequence);
            let cmd = pop_command(seqr, icarus, POP_BACK);
            prime(seqr, tm, icarus, host, owner_id, cmd);
        }
        _ => {
            i_dprintf(icarus, host, WL_ERROR, "unknown affect type found");
        }
    }

    SEQ_OK
}

/// Raven `CSequencer::Callback` — handle a completed task and hand back the next.
/// Ownership-correct fold of the pinned signature (CONFIRMED problem #5/#6): the
/// block is taken **by value** so the two Raven branches — `PushCommand(block,…)`
/// (transfer) and `delete block` (drop) — both express (the borrowed `&Block`
/// the frozen doc pinned could do neither).
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2026-2074` (`sequencer.h:81`)
pub fn callback(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    block: Block,
    return_code: i32,
) -> i32 {
    if return_code == TASK_RETURN_COMPLETE {
        // There are no more pending commands.
        let cur = match seqr.m_cur_sequence {
            Some(c) => c,
            None => {
                drop(block); // delete block
                return SEQ_OK;
            }
        };

        // Check to retain the command.
        if seq_flags(icarus, cur, SQ_RETAIN) != 0 {
            push_command(seqr, icarus, block, PUSH_FRONT);
        } else {
            drop(block);
        }

        // Check for pending commands.
        if seq_num_commands(icarus, cur) <= 0 {
            match seq_return(icarus, cur) {
                None => return SEQ_OK,
                Some(r) => seqr.m_cur_sequence = Some(r),
            }
        }

        let mut command = pop_command(seqr, icarus, POP_BACK);
        prep(seqr, tm, icarus, host, owner_id, &mut command);

        if let Some(cmd) = command {
            tm.set_command(cmd, PUSH_FRONT);
        }

        return SEQ_OK;
    }

    // Raven notes this error could be more descriptive.
    i_dprintf(icarus, host, WL_ERROR, "command could not be called back\n");
    SEQ_FAILED
}

/// Raven `CSequencer::Recall` — flush the task manager's recalled tasks back onto
/// the current sequence. `taskManager` is threaded in (ruling 24).
/// Source: `oracle/codemp/icarus/Sequencer.cpp:2082-2106` (`sequencer.h:100`)
fn recall(seqr: &mut Sequencer, tm: &mut TaskManager, icarus: &mut Icarus) -> i32 {
    // Raven: `if (!m_taskManager) { assert(0); return true; }` — the manager is
    // always present here (threaded in), so that guard never fires.
    while let Some(block) = tm.recall_task() {
        if seqr.m_cur_sequence.is_some() {
            push_command(seqr, icarus, block, PUSH_BACK);
        } else {
            drop(block);
        }
    }
    1 // true
}

#[cfg(test)]
mod tests {
    use super::*;

    // `create()` mirrors `new CSequencer` zero-init.
    #[test]
    fn create_zero_inits() {
        let seq = Sequencer::create();
        assert_eq!(seq.m_owner_id, 0);
        assert_eq!(seq.m_num_commands, 0);
        assert!(seq.m_cur_sequence.is_none());
        assert!(seq.m_streams_created.is_empty());
    }

    // Semantics reasoned about: the two save/load methods are inert `#if 0`
    // bodies that fall through to `return false;` (0) in the MP dedicated build.
    #[test]
    fn save_and_load_are_inert_false() {
        let mut seq = Sequencer::create();
        assert_eq!(seq.save(), 0);
        assert_eq!(seq.load(), 0);
    }

    #[test]
    fn strip_extension_trims_trailing_dot_segment() {
        assert_eq!(BlockStream::strip_extension("scripts/foo.ibi"), "scripts/foo");
        assert_eq!(BlockStream::strip_extension("noext"), "noext");
    }
}
