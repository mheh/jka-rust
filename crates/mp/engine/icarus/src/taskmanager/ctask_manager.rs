//! Raven `CTaskManager` — the per-entity task scheduler.

use std::collections::BTreeMap;

use mp_host_interface::EngineHost;
use mp_qshared::shared::vec3_t;
use mp_qshared::shared::limits::MAX_GENTITIES;

use crate::blockstream::cblock::Block;
use crate::sequencer::csequencer::{self, Sequencer};
use crate::taskmanager::ctask::Task;
use crate::taskmanager::ctask_group::TaskGroup;
use crate::Icarus;

/// Raven's anonymous `enum { TASK_OK, TASK_FAILED, TASK_START, TASK_END }`.
/// Source: `oracle/codemp/icarus/taskmanager.h:23-29`
const TASK_OK: i32 = 0;
const TASK_FAILED: i32 = 1;
const TASK_START: i32 = 2;
const TASK_END: i32 = 3;

/// Raven's anonymous `enum { TASK_RETURN_COMPLETE, TASK_RETURN_FAILED }`.
/// Source: `oracle/codemp/icarus/taskmanager.h:17-21`
const TASK_RETURN_COMPLETE: i32 = 0;

/// Raven `const int RUNAWAY_LIMIT = 256`.
/// Source: `oracle/codemp/icarus/taskmanager.h:15`
const RUNAWAY_LIMIT: i32 = 256;

/// Raven anonymous `enum { POP_FRONT, POP_BACK, PUSH_FRONT, PUSH_BACK }`.
/// Source: `oracle/codemp/icarus/blockstream.h:26-32`
const POP_FRONT: i32 = 0;
const POP_BACK: i32 = 1;
const PUSH_FRONT: i32 = 2;
const PUSH_BACK: i32 = 3;

// Block / member ids the scheduler dispatches on (interpreter.h enum resolved
// from `NUM_USER_TOKENS = 19`). Tokenizer/interpreter are out-of-scope
// skeletons (§ Out of scope), so the resolved values are pinned locally.
// Source: `oracle/codemp/icarus/interpreter.h:35-67`
const ID_SOUND: i32 = 20;
const ID_MOVE: i32 = 21;
const ID_ROTATE: i32 = 22;
const ID_WAIT: i32 = 23;
const ID_SET: i32 = 26;
const ID_PRINT: i32 = 29;
const ID_USE: i32 = 30;
const ID_KILL: i32 = 33;
const ID_REMOVE: i32 = 34;
const ID_CAMERA: i32 = 35;
const ID_GET: i32 = 36;
const ID_RANDOM: i32 = 37;
const ID_DECLARE: i32 = 43;
const ID_FREE: i32 = 44;
const ID_SIGNAL: i32 = 46;
const ID_WAITSIGNAL: i32 = 47;
const ID_PLAY: i32 = 48;
const ID_TAG: i32 = 49;

// Token-type ids (`tokenizer.h`/`interpreter.h`). Source:
// `oracle/codemp/icarus/tokenizer.h:63-75`, `interpreter.h:16-23`.
const TK_STRING: i32 = 4;
const TK_INT: i32 = 5;
const TK_FLOAT: i32 = 6;
const TK_IDENTIFIER: i32 = 7;
const TK_VECTOR: i32 = 14;

// Camera sub-type ids (`interpreter.h` type enum resolved from `NUM_IDS = 51`).
// Source: `oracle/codemp/icarus/interpreter.h:87-98`.
const TYPE_PAN: i32 = 57;
const TYPE_ZOOM: i32 = 58;
const TYPE_MOVE: i32 = 59;
const TYPE_FADE: i32 = 60;
const TYPE_PATH: i32 = 61;
const TYPE_ENABLE: i32 = 62;
const TYPE_DISABLE: i32 = 63;
const TYPE_SHAKE: i32 = 64;
const TYPE_ROLL: i32 = 65;
const TYPE_TRACK: i32 = 66;
const TYPE_DISTANCE: i32 = 67;
const TYPE_FOLLOW: i32 = 68;

/// Raven `Q3_INFINITE`. Source: `oracle/codemp/game/g_public.h:9`
const Q3_INFINITE: f32 = 16777216.0;

/// Raven `WL_ERROR`/`WL_WARNING` print levels (as `i32`). The developer-gated
/// `WL_DEBUG` per-command trace lines (`"%4d cmd(...); [%d]"`) are dropped —
/// diagnostic-only, gated on the `developer` cvar (0 in the goldens), so they
/// reach no observable state (Divergences).
/// Source: `oracle/codemp/game/q_shared.h:428-433`
const WL_ERROR: i32 = 1;
const WL_WARNING: i32 = 2;

/// Raven `SVF_ICARUS_FREEZE` — while set the entity does not execute ICARUS
/// commands. Redeclared here (it lives in `mp_game`'s `g_public` consts).
/// Source: `oracle/codemp/game/g_public.h:35`
const SVF_ICARUS_FREEZE: i32 = 0x0000_8000;

/// Handle into [`TaskManager::m_task_groups`] (ICARUS-D3 / ruling 39d — declared
/// beside its owning arena, RMG `AreaId` §B5 precedent).
/// Source: `oracle/codemp/icarus/taskmanager.h:177`
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct TaskGroupId(pub i32);

/// Raven `CTaskManager` → `TaskManager` (§F idiomatic, ICARUS-D1 naming).
///
/// The per-entity scheduler. The three parallel `CTaskGroup*` indexes collapse
/// to one owner `m_task_groups: Vec<TaskGroup>` + `TaskGroupId`, with
/// name/id side-indexes (ICARUS-D3 / ruling 27); `m_tasks: Vec<Task>` owns the
/// tasks. The `CSequencer *m_owner` back-ref is dropped — the dispatch
/// sites become free fns `(&mut Sequencer, &mut TaskManager, &mut Icarus,
/// &mut dyn EngineHost, …)` (ruling 24). `Update` is the per-frame heartbeat,
/// gated by `SVF_ICARUS_FREEZE` via `host.gentity(owner_id)`.
/// Type definition source: `oracle/codemp/icarus/taskmanager.h:97-189`
#[derive(Default)]
pub struct TaskManager {
    /// Raven `int m_ownerID`.
    pub m_owner_id: i32,
    /// Raven `CTaskGroup *m_curGroup`.
    pub m_cur_group: Option<TaskGroupId>,
    /// Raven `taskGroup_v m_taskGroups` — owning arena.
    pub m_task_groups: Vec<TaskGroup>,
    /// Raven `tasks_l m_tasks` — owned tasks.
    pub m_tasks: Vec<Task>,
    /// Raven `int m_GUID`.
    pub m_guid: i32,
    /// Raven `int m_count`.
    pub m_count: i32,
    /// Raven `taskGroupName_m m_taskGroupNameMap` (`map<string, CTaskGroup*>`).
    pub m_task_group_name_map: BTreeMap<String, TaskGroupId>,
    /// Raven `taskGroupID_m m_taskGroupIDMap` (`map<int, CTaskGroup*>`).
    pub m_task_group_id_map: BTreeMap<i32, TaskGroupId>,
    /// Raven `bool m_resident`.
    pub m_resident: bool,
}

impl TaskManager {
    /// Raven `CTaskManager::Create` — `new CTaskManager` (empty ctor; fields are
    /// seeded later by `Init`), so the port hands back a `Default` value.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:162-165`
    pub fn create() -> TaskManager {
        TaskManager::default()
    }

    /// Raven `CTaskManager::Flush` — an unimplemented stub: it clears nothing and
    /// simply `return true;` (Raven marks it for a rewrite). Faithfully returns
    /// `1` (`true` widened to `int`), *not* `TASK_OK`.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:227-232`
    pub fn flush(&mut self) -> i32 {
        1
    }

    /// Raven `CTaskManager::IsRunning` — true iff pending tasks remain.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:346-349`
    pub fn is_running(&self) -> bool {
        !self.m_tasks.is_empty()
    }

    /// Raven `CTaskManager::AddTaskGroup`.
    ///
    /// If a group of this name already exists, it is reset via `Init` and its
    /// handle returned; otherwise a new group is allocated, stamped with the next
    /// GUID (`m_GUID++`), pushed to the arena, and registered in both side-indexes.
    /// Raven's `new`-returned-NULL guard/`I_DPrintf` branch is dead (`new` never
    /// returns null here) and is dropped, matching the `CTask`/`CBlock` precedents.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:240-278`
    pub fn add_task_group(&mut self, name: &str) -> TaskGroupId {
        // Collect any garbage: an existing group of this name is cleared and reused.
        if let Some(&id) = self.m_task_group_name_map.get(name) {
            self.m_task_groups[id.0 as usize].init();
            return id;
        }

        // Allocate a new one — its handle is its slot in the owning arena.
        let id = TaskGroupId(self.m_task_groups.len() as i32);
        let mut group = TaskGroup::default();

        // Setup the internal information: `group->SetGUID( m_GUID++ )`.
        let guid = self.m_guid;
        self.m_guid += 1;
        group.set_guid(guid);

        // Add it to the arena and associate it for retrieval later.
        self.m_task_groups.push(group);
        self.m_task_group_name_map.insert(name.to_string(), id);
        self.m_task_group_id_map.insert(guid, id);

        id
    }

    /// Raven `CTaskManager::GetTaskGroup( const char * )` — name lookup.
    /// The `I_DPrintf(WL_WARNING,…)` "not found" note is developer-gated and
    /// reaches no state here, so it is dropped; the lookup result is faithful.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:286-299`
    pub fn get_task_group(&self, name: &str) -> Option<TaskGroupId> {
        self.m_task_group_name_map.get(name).copied()
    }

    /// Raven `CTaskManager::GetTaskGroup( int )` — id lookup.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:301-314`
    pub fn get_task_group_by_id(&self, id: i32) -> Option<TaskGroupId> {
        self.m_task_group_id_map.get(&id).copied()
    }

    /// Raven `CTaskManager::MarkTask`.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:870-904`
    pub fn mark_task(&mut self, id: i32, operation: i32) -> i32 {
        let group = match self.get_task_group_by_id(id) {
            Some(g) => g,
            None => return TASK_FAILED,
        };

        if operation == TASK_START {
            // Reset all the completion information.
            self.m_task_groups[group.0 as usize].init();
            self.m_task_groups[group.0 as usize].m_parent = self.m_cur_group;
            self.m_cur_group = Some(group);
        } else if operation == TASK_END {
            let cur = match self.m_cur_group {
                Some(g) => g,
                None => return TASK_FAILED,
            };
            self.m_cur_group = self.m_task_groups[cur.0 as usize].m_parent;
        }

        TASK_OK
    }

    /// Raven `CTaskManager::Completed` — mark a task complete in the first group
    /// that owns it.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:912-925`
    pub fn completed(&mut self, id: i32) -> i32 {
        for group in self.m_task_groups.iter_mut() {
            if group.mark_task_complete(id) {
                break;
            }
        }
        TASK_OK
    }

    /// Raven `CTaskManager::SetCommand` — wrap a command block in a task, add it
    /// to the current group (if any), and push it.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:841-862`
    pub fn set_command(&mut self, command: Block, type_: i32) -> i32 {
        let guid = self.m_guid;
        self.m_guid += 1;
        let task = Task::create(guid, command);

        // If this is part of a task group, add it in.
        if let Some(group) = self.m_cur_group {
            self.m_task_groups[group.0 as usize].add(&task);
        }

        self.push_task(task, type_);
        TASK_OK
    }

    /// Raven `CTaskManager::PushTask`.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:975-996`
    pub fn push_task(&mut self, task: Task, flag: i32) -> i32 {
        match flag {
            PUSH_FRONT => {
                self.m_tasks.insert(0, task);
                TASK_OK
            }
            PUSH_BACK => {
                self.m_tasks.push(task);
                TASK_OK
            }
            // Invalid flag (Raven `return SEQ_FAILED`, == TASK_FAILED == 1).
            _ => TASK_FAILED,
        }
    }

    /// Raven `CTaskManager::PopTask`.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:1004-1032`
    pub fn pop_task(&mut self, flag: i32) -> Option<Task> {
        if self.m_tasks.is_empty() {
            return None;
        }
        match flag {
            POP_FRONT => Some(self.m_tasks.remove(0)),
            POP_BACK => self.m_tasks.pop(),
            _ => None,
        }
    }

    /// Raven `CTaskManager::GetCurrentTask` — pop the back task, free it, and
    /// return its owned block (used by the sequencer's `Interrupt`).
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:1040-1052`
    pub fn get_current_task(&mut self) -> Option<Block> {
        self.pop_task(POP_BACK).map(|task| task.m_block)
    }

    /// Raven `CTaskManager::RecallTask` — pop the back task, free it, and return
    /// its owned block (used by the sequencer's `Recall`).
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:950-967`
    pub fn recall_task(&mut self) -> Option<Block> {
        self.pop_task(POP_BACK).map(|task| task.m_block)
    }

    /// Raven `CTaskManager::Save` — inert in MP dedicated: the entire body is
    /// `#if 0`'d out, so it persists nothing in this build (Divergences).
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:1663-1782`
    pub fn save(&self) {}

    /// Raven `CTaskManager::Load` — inert in MP dedicated: the entire body is
    /// `#if 0`'d out, so it restores nothing in this build (Divergences).
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:1790-…`
    pub fn load(&mut self) {}
}

// ---------------------------------------------------------------------------
// Small transcription helpers (pure block-member reads, no host).
// ---------------------------------------------------------------------------

/// Read a member's raw data as a native-endian `f32` (`*(float *)GetMemberData`),
/// advancing `member_num`. Short/absent data reads `0.0` (§19 guard).
fn member_float(block: &Block, member_num: &mut i32) -> f32 {
    let data = block.get_member_data(*member_num).unwrap_or(&[]);
    *member_num += 1;
    let mut buf = [0u8; 4];
    let n = data.len().min(4);
    buf[..n].copy_from_slice(&data[..n]);
    f32::from_ne_bytes(buf)
}

/// Read a member's raw data as a native-endian `f32` **without** advancing
/// (`*(float *)GetMemberData(memberNum)`, no `++`). Short data reads `0.0`.
fn peek_member_float(block: &Block, member_num: i32) -> f32 {
    let data = block.get_member_data(member_num).unwrap_or(&[]);
    let mut buf = [0u8; 4];
    let n = data.len().min(4);
    buf[..n].copy_from_slice(&data[..n]);
    f32::from_ne_bytes(buf)
}

/// Read a member's raw data as a native-endian `i32` (`*(int *)GetMemberData`),
/// advancing `member_num`. Short/absent data reads `0` (§19 guard).
fn member_int(block: &Block, member_num: &mut i32) -> i32 {
    let data = block.get_member_data(*member_num).unwrap_or(&[]);
    *member_num += 1;
    let mut buf = [0u8; 4];
    let n = data.len().min(4);
    buf[..n].copy_from_slice(&data[..n]);
    i32::from_ne_bytes(buf)
}

/// Read a member's data as a C string (`(char *)GetMemberData`), advancing
/// `member_num`. Bytes are read up to the first NUL.
fn member_c_string(block: &Block, member_num: &mut i32) -> String {
    let data = block.get_member_data(*member_num).unwrap_or(&[]);
    *member_num += 1;
    bytes_to_c_string(data)
}

/// C string out of a raw byte field (up to the first NUL, lossy UTF-8).
fn bytes_to_c_string(data: &[u8]) -> String {
    let len = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..len]).into_owned()
}

/// Raven `CTaskManager::Check` — does the member at `memberNum` have id `targetID`?
/// Source: `oracle/codemp/icarus/TaskManager.cpp:356-362`
fn check(block: &Block, member_num: i32, target_id: i32) -> bool {
    block.get_member(member_num).is_some_and(|m| m.m_id == target_id)
}

/// The per-frame heartbeat gate re-index of the disjoint `Icarus` field borrows.
/// Raven `SV_GentityNum(m_ownerID)` then `owner->r.svFlags & SVF_ICARUS_FREEZE`
/// (`TaskManager.cpp:322-329`). A free fn per ICARUS-D3 (ruling 24): it takes the
/// sequencer/task-manager out of their owning slots into detached locals so the
/// task handlers can hold `&mut Icarus` for `I_*` dispatch while the scheduler is
/// a local, then restores them.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:322-338`
pub fn update(icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32) -> i32 {
    // Freeze gate: dereferencing the raw `*mut sharedEntity_t` is the confined
    // ABI-seam `unsafe` (porting-rules §D11).
    let owner = host.gentity(owner_id);
    if unsafe { (*owner).r.svFlags } & SVF_ICARUS_FREEZE != 0 {
        return TASK_FAILED;
    }

    let idx = owner_id as usize;
    if owner_id < 0 || idx >= MAX_GENTITIES {
        return TASK_FAILED;
    }

    // Detach the sequencer + task manager so `Go`'s handlers/callback can hold
    // `&mut Icarus` while these are local (ruling 24 disjoint-borrow discipline).
    let sequencer_id = match icarus.sequencers[idx] {
        Some(id) => id,
        None => return TASK_FAILED,
    };
    let mut seqr = match icarus
        .instance
        .as_mut()
        .and_then(|inst| inst.take_sequencer(sequencer_id))
    {
        Some(s) => s,
        None => return TASK_FAILED,
    };
    let mut tm = match icarus.task_managers[idx].take() {
        Some(tm) => tm,
        None => {
            // Restore the sequencer before bailing.
            if let Some(inst) = icarus.instance.as_mut() {
                inst.restore_sequencer(sequencer_id, seqr);
            }
            return TASK_FAILED;
        }
    };

    tm.m_count = 0; // Needed for runaway init.
    tm.m_resident = true;

    let return_val = go(&mut seqr, &mut tm, icarus, host, owner_id);

    tm.m_resident = false;

    // Restore the detached scheduler.
    icarus.task_managers[idx] = Some(tm);
    if let Some(inst) = icarus.instance.as_mut() {
        inst.restore_sequencer(sequencer_id, seqr);
    }

    return_val
}

/// Raven `CTaskManager::Go` — the heartbeat's task-dispatch loop.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:700-833`
pub fn go(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
) -> i32 {
    // Check for run away scripts.
    tm.m_count += 1;
    if tm.m_count > RUNAWAY_LIMIT {
        i_dprintf(icarus, host, WL_ERROR, "Runaway loop detected!\n");
        return TASK_FAILED;
    }

    // If there are tasks to complete, do so.
    if tm.m_tasks.is_empty() {
        return TASK_OK;
    }

    // Get the next task.
    let mut task = match tm.pop_task(POP_BACK) {
        Some(t) => t,
        None => {
            i_dprintf(icarus, host, WL_ERROR, "Invalid task found in Go()!\n");
            return TASK_FAILED;
        }
    };

    // If this hasn't been stamped, do so.
    if task.m_time_stamp == 0 {
        task.m_time_stamp = i_get_time(icarus, host);
    }

    // Switch and call the proper function.
    match task.get_id() {
        ID_WAIT => {
            let completed = wait(tm, icarus, host, owner_id, &mut task);
            if !completed {
                tm.push_task(task, PUSH_BACK);
                return TASK_OK;
            }
            tm.completed(task.m_id);
        }
        ID_WAITSIGNAL => {
            let completed = wait_signal(icarus, host, owner_id, &task);
            if !completed {
                tm.push_task(task, PUSH_BACK);
                return TASK_OK;
            }
            tm.completed(task.m_id);
        }
        ID_PRINT => {
            print(tm, icarus, host, owner_id, &task);
        }
        ID_SOUND => {
            sound(tm, icarus, host, owner_id, &task);
        }
        ID_MOVE => {
            move_(icarus, host, owner_id, &task);
        }
        ID_ROTATE => {
            rotate(icarus, host, owner_id, &task);
        }
        ID_KILL => {
            kill(tm, icarus, host, owner_id, &task);
        }
        ID_REMOVE => {
            remove(tm, icarus, host, owner_id, &task);
        }
        ID_CAMERA => {
            camera(tm, icarus, host, owner_id, &task);
        }
        ID_SET => {
            set(icarus, host, owner_id, &task);
        }
        ID_USE => {
            use_(tm, icarus, host, owner_id, &task);
        }
        ID_DECLARE => {
            declare_variable(tm, icarus, host, owner_id, &task);
        }
        ID_FREE => {
            free_variable(tm, icarus, host, owner_id, &task);
        }
        ID_SIGNAL => {
            signal(tm, icarus, host, owner_id, &task);
        }
        ID_PLAY => {
            play(icarus, host, owner_id, &task);
        }
        _ => {
            i_dprintf(icarus, host, WL_ERROR, "Found unknown task type!\n");
            return TASK_FAILED;
        }
    }

    // Pump the sequencer for another task; the owned block moves out of the task.
    callback_command(seqr, tm, icarus, host, owner_id, task.m_block, TASK_RETURN_COMPLETE)
}

/// Raven `CTaskManager::CallbackCommand` — hand the finished block back to the
/// sequencer, then pump `Go` again.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:933-942`
fn callback_command(
    seqr: &mut Sequencer,
    tm: &mut TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    block: Block,
    return_code: i32,
) -> i32 {
    if csequencer::callback(seqr, tm, icarus, host, owner_id, block, return_code)
        == csequencer::SEQ_OK
    {
        return go(seqr, tm, icarus, host, owner_id);
    }

    i_dprintf(icarus, host, WL_ERROR, "Command callback failure!\n");
    TASK_FAILED
}

// ---------------------------------------------------------------------------
// I_* dispatch shims — copy the fn pointer out of `icarus.interface_export`
// (fn pointers are `Copy`) then call it with `&mut Icarus` (Raven `m_ie->I_*`).
// ---------------------------------------------------------------------------

fn i_dprintf(icarus: &mut Icarus, host: &mut dyn EngineHost, level: i32, msg: &str) {
    let f = icarus.interface_export.i_dprintf;
    f(icarus, host, level, msg);
}

fn i_get_time(icarus: &mut Icarus, host: &mut dyn EngineHost) -> u32 {
    let f = icarus.interface_export.i_get_time;
    f(icarus, host)
}

// ---------------------------------------------------------------------------
// Value extraction — Get / GetFloat / GetVector (TaskManager.cpp:370-692).
// Out-params fold to `Option`: `None` == Raven's `false` (§C7).
// ---------------------------------------------------------------------------

/// Raven `CTaskManager::GetFloat`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:370-435`
fn get_float(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_id: i32,
    block: &Block,
    member_num: &mut i32,
) -> Option<f32> {
    // See if this is a get() command replacement.
    if check(block, *member_num, ID_GET) {
        *member_num += 1;
        let type_ = member_float(block, member_num) as i32;
        let name = member_c_string(block, member_num);

        if type_ != TK_FLOAT {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                "Get() call tried to return a non-FLOAT parameter!\n",
            );
            return None;
        }

        let mut value = 0.0f32;
        let f = icarus.interface_export.i_get_float;
        let r = f(icarus, host, ent_id, type_, &name, &mut value);
        return if r != 0 { Some(value) } else { None };
    }

    // Look for a random() inline call.
    if check(block, *member_num, ID_RANDOM) {
        *member_num += 1;
        let min = member_float(block, member_num);
        let max = member_float(block, member_num);
        let f = icarus.interface_export.i_random;
        return Some(f(icarus, host, min, max));
    }

    // Look for a tag() inline call — not a valid replacement for FLOAT.
    if check(block, *member_num, ID_TAG) {
        i_dprintf(
            icarus,
            host,
            WL_WARNING,
            "Invalid use of \"tag\" inline.  Not a valid replacement for type FLOAT\n",
        );
        return None;
    }

    let bm_id = block.get_member(*member_num).map(|m| m.m_id).unwrap_or(-1);
    if bm_id == TK_INT {
        Some(member_int(block, member_num) as f32)
    } else if bm_id == TK_FLOAT {
        Some(member_float(block, member_num))
    } else {
        i_dprintf(icarus, host, WL_WARNING, "Unexpected value; expected type FLOAT\n");
        None
    }
}

/// Raven `CTaskManager::GetVector`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:443-523`
fn get_vector(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_id: i32,
    block: &Block,
    member_num: &mut i32,
) -> Option<vec3_t> {
    // See if this is a get() command replacement.
    if check(block, *member_num, ID_GET) {
        *member_num += 1;
        let type_ = member_float(block, member_num) as i32;
        let name = member_c_string(block, member_num);

        if type_ != TK_VECTOR {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                "Get() call tried to return a non-VECTOR parameter!\n",
            );
        }

        let mut value = [0.0f32; 3];
        let f = icarus.interface_export.i_get_vector;
        let r = f(icarus, host, ent_id, type_, &name, &mut value);
        return if r != 0 { Some(value) } else { None };
    }

    // Look for a random() inline call.
    if check(block, *member_num, ID_RANDOM) {
        *member_num += 1;
        let min = member_float(block, member_num);
        let max = member_float(block, member_num);
        let f = icarus.interface_export.i_random;
        let mut value = [0.0f32; 3];
        for slot in value.iter_mut() {
            *slot = f(icarus, host, min, max);
        }
        return Some(value);
    }

    // Look for a tag() inline call.
    if check(block, *member_num, ID_TAG) {
        *member_num += 1;
        let tag_name = get(icarus, host, ent_id, block, member_num)?;
        let tag_lookup = get_float(icarus, host, ent_id, block, member_num)?;

        let mut value = [0.0f32; 3];
        let f = icarus.interface_export.i_get_tag;
        if f(icarus, host, ent_id, &tag_name, tag_lookup as i32, &mut value) == 0 {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                &format!("Unable to find tag \"{}\" for ent {}!\n", tag_name, ent_id),
            );
            return None;
        }
        return Some(value);
    }

    // Check for a real vector here (peek the type without advancing).
    let type_ = peek_member_float(block, *member_num) as i32;
    if type_ != TK_VECTOR {
        return None;
    }
    *member_num += 1;

    let mut value = [0.0f32; 3];
    for slot in value.iter_mut() {
        *slot = get_float(icarus, host, ent_id, block, member_num)?;
    }
    Some(value)
}

/// Raven `CTaskManager::Get` — `char **value` out-param folds to `Option<String>`
/// (a fresh owned string per read, replacing Raven's reused `static tempBuffer`;
/// behavior-identical except in the multi-numeric-Get aliasing corner Raven's
/// shared buffer would clobber — that clobber has no live caller here).
/// Source: `oracle/codemp/icarus/TaskManager.cpp:531-692`
fn get(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_id: i32,
    block: &Block,
    member_num: &mut i32,
) -> Option<String> {
    // Look for a get() inline call.
    if check(block, *member_num, ID_GET) {
        *member_num += 1;
        let type_ = member_float(block, member_num) as i32;
        let name = member_c_string(block, member_num);

        match type_ {
            TK_STRING => {
                let f = icarus.interface_export.i_get_string;
                match f(icarus, host, ent_id, type_, &name) {
                    Some(s) => Some(s),
                    None => {
                        i_dprintf(
                            icarus,
                            host,
                            WL_ERROR,
                            &format!("Get() parameter \"{}\" could not be found!\n", name),
                        );
                        None
                    }
                }
            }
            TK_FLOAT => {
                let mut temp = 0.0f32;
                let f = icarus.interface_export.i_get_float;
                if f(icarus, host, ent_id, type_, &name, &mut temp) == 0 {
                    i_dprintf(
                        icarus,
                        host,
                        WL_ERROR,
                        &format!("Get() parameter \"{}\" could not be found!\n", name),
                    );
                    return None;
                }
                Some(format!("{:.6}", temp))
            }
            TK_VECTOR => {
                let mut vval = [0.0f32; 3];
                let f = icarus.interface_export.i_get_vector;
                if f(icarus, host, ent_id, type_, &name, &mut vval) == 0 {
                    i_dprintf(
                        icarus,
                        host,
                        WL_ERROR,
                        &format!("Get() parameter \"{}\" could not be found!\n", name),
                    );
                    return None;
                }
                Some(format!("{:.6} {:.6} {:.6}", vval[0], vval[1], vval[2]))
            }
            _ => {
                i_dprintf(
                    icarus,
                    host,
                    WL_ERROR,
                    "Get() call tried to return an unknown type!\n",
                );
                None
            }
        }
    } else if check(block, *member_num, ID_RANDOM) {
        // Look for a random() inline call.
        *member_num += 1;
        let min = member_float(block, member_num);
        let max = member_float(block, member_num);
        let f = icarus.interface_export.i_random;
        let ret = f(icarus, host, min, max);
        Some(format!("{:.6}", ret))
    } else if check(block, *member_num, ID_TAG) {
        // Look for a tag() inline call.
        *member_num += 1;
        let tag_name = get(icarus, host, ent_id, block, member_num)?;
        let tag_lookup = get_float(icarus, host, ent_id, block, member_num)?;

        let mut vector = [0.0f32; 3];
        let f = icarus.interface_export.i_get_tag;
        if f(icarus, host, ent_id, &tag_name, tag_lookup as i32, &mut vector) == 0 {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                &format!("Unable to find tag \"{}\"!\n", tag_name),
            );
            return None;
        }
        Some(format!("{:.6} {:.6} {:.6}", vector[0], vector[1], vector[2]))
    } else {
        // Get an actual piece of data.
        let bm_id = block.get_member(*member_num).map(|m| m.m_id).unwrap_or(-1);
        if bm_id == TK_INT {
            let fval = member_int(block, member_num) as f32;
            Some(format!("{:.6}", fval))
        } else if bm_id == TK_FLOAT {
            let fval = member_float(block, member_num);
            Some(format!("{:.6}", fval))
        } else if bm_id == TK_VECTOR {
            // Raven's loop re-derives the whole vector string each iteration
            // (a quirk preserved); the final string holds all three axes.
            *member_num += 1;
            let mut vval = [0.0f32; 3];
            for i in 0..3 {
                vval[i] = get_float(icarus, host, ent_id, block, member_num)?;
            }
            Some(format!("{:.6} {:.6} {:.6}", vval[0], vval[1], vval[2]))
        } else if bm_id == TK_STRING || bm_id == TK_IDENTIFIER {
            Some(member_c_string(block, member_num))
        } else {
            i_dprintf(icarus, host, WL_WARNING, "Unexpected value; expected type STRING\n");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Task functions (TaskManager.cpp:1062-1610). Each reads its block, dispatches
// through `I_*`, and (for instant commands) self-completes its task group.
// ---------------------------------------------------------------------------

/// Raven `CTaskManager::Wait` — returns `completed`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1062-1138`
fn wait(
    tm: &TaskManager,
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    owner_id: i32,
    task: &mut Task,
) -> bool {
    let mut member_num = 0;

    let bm_id = task.m_block.get_member(0).map(|m| m.m_id).unwrap_or(-1);

    // Check if this is a task completion wait.
    if bm_id == TK_STRING {
        let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
            Some(s) => s,
            None => return false,
        };
        // `GetTaskGroup(sVal)` on the running manager; unknown name → not complete.
        return match tm.get_task_group(&s_val) {
            Some(gid) => tm.m_task_groups[gid.0 as usize].complete(),
            None => false,
        };
    }

    // Otherwise it's a time completion wait.
    let dwtime;
    if check(&task.m_block, member_num, ID_RANDOM) {
        // Get it random only the first time.
        let first = peek_member_float(&task.m_block, member_num);
        if first == Q3_INFINITE {
            // We have not evaluated this random yet.
            let mut mn = member_num + 1; // past the ID_RANDOM sentinel member
            let min = member_float(&task.m_block, &mut mn);
            let max = member_float(&task.m_block, &mut mn);
            let f = icarus.interface_export.i_random;
            dwtime = f(icarus, host, min, max);
            // Store the result in the first member.
            if let Some(m) = task.m_block.m_members.get_mut(member_num as usize) {
                m.set_data(&dwtime.to_ne_bytes());
            }
        } else {
            dwtime = first;
        }
    } else {
        dwtime = match get_float(icarus, host, owner_id, &task.m_block, &mut member_num) {
            Some(v) => v,
            None => return false,
        };
    }

    let time = i_get_time(icarus, host);
    if (task.m_time_stamp as f32 + dwtime) < (time as f32) {
        // Complete: re-randomize next time if this was a random wait.
        if check(&task.m_block, 0, ID_RANDOM) {
            let dwinf = Q3_INFINITE;
            if let Some(m) = task.m_block.m_members.get_mut(0) {
                m.set_data(&dwinf.to_ne_bytes());
            }
        }
        return true;
    }
    false
}

/// Raven `CTaskManager::WaitSignal` — returns `completed`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1146-1169`
fn wait_signal(icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) -> bool {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return false,
    };

    // `m_owner->GetOwner()->CheckSignal(sVal)` → the instance signal map.
    let signalled = icarus
        .instance
        .as_ref()
        .is_some_and(|inst| inst.check_signal(&s_val));

    if signalled {
        if let Some(inst) = icarus.instance.as_mut() {
            inst.clear_signal(&s_val);
        }
        return true;
    }
    false
}

/// Raven `CTaskManager::Print`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1177-1192`
fn print(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_center_print;
    f(icarus, host, &s_val);
    tm.completed(task.m_id);
}

/// Raven `CTaskManager::Sound`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1200-1216`
fn sound(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let s_val2 = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    // Only instantly complete if the user has requested it.
    let f = icarus.interface_export.i_play_sound;
    if f(icarus, host, task.m_id, owner_id, &s_val2, &s_val) != 0 {
        tm.completed(task.m_id);
    }
}

/// Raven `CTaskManager::Rotate`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1224-1261`
fn rotate(icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let block = &task.m_block;
    let mut member_num = 0;
    let vector;

    if check(block, member_num, ID_TAG) {
        member_num += 1;
        let tag_name = match get(icarus, host, owner_id, block, &mut member_num) {
            Some(s) => s,
            None => return,
        };
        let tag_lookup = match get_float(icarus, host, owner_id, block, &mut member_num) {
            Some(v) => v,
            None => return,
        };
        let mut v = [0.0f32; 3];
        let f = icarus.interface_export.i_get_tag;
        if f(icarus, host, owner_id, &tag_name, tag_lookup as i32, &mut v) == 0 {
            i_dprintf(
                icarus,
                host,
                WL_ERROR,
                &format!("Unable to find tag \"{}\"!\n", tag_name),
            );
            return;
        }
        vector = v;
    } else {
        vector = match get_vector(icarus, host, owner_id, block, &mut member_num) {
            Some(v) => v,
            None => return,
        };
    }

    let duration = match get_float(icarus, host, owner_id, block, &mut member_num) {
        Some(v) => v,
        None => return,
    };
    let f = icarus.interface_export.i_lerp2_angles;
    f(icarus, host, task.m_id, owner_id, vector, duration);
}

/// Raven `CTaskManager::Remove`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1269-1283`
fn remove(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_remove;
    f(icarus, host, owner_id, &s_val);
    tm.completed(task.m_id);
}

/// Raven `CTaskManager::Camera` — all camera targets are the "NOT SUPPORTED IN
/// MP" `CGCam_*` no-ops (`Q3_Interface.cpp`), but the arg parse + dispatch port.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1291-1417`
fn camera(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let block = &task.m_block;
    let mut member_num = 0;

    let type_ = match get_float(icarus, host, owner_id, block, &mut member_num) {
        Some(v) => v as i32,
        None => return,
    };

    macro_rules! gv {
        () => {
            match get_vector(icarus, host, owner_id, block, &mut member_num) {
                Some(v) => v,
                None => return,
            }
        };
    }
    macro_rules! gf {
        () => {
            match get_float(icarus, host, owner_id, block, &mut member_num) {
                Some(v) => v,
                None => return,
            }
        };
    }
    macro_rules! gs {
        () => {
            match get(icarus, host, owner_id, block, &mut member_num) {
                Some(s) => s,
                None => return,
            }
        };
    }

    match type_ {
        TYPE_PAN => {
            let v = gv!();
            let v2 = gv!();
            let f = gf!();
            let fp = icarus.interface_export.i_camera_pan;
            fp(icarus, host, v, v2, f);
        }
        TYPE_ZOOM => {
            let f = gf!();
            let f2 = gf!();
            let fp = icarus.interface_export.i_camera_zoom;
            fp(icarus, host, f, f2);
        }
        TYPE_MOVE => {
            let v = gv!();
            let f = gf!();
            let fp = icarus.interface_export.i_camera_move;
            fp(icarus, host, v, f);
        }
        TYPE_ROLL => {
            let f = gf!();
            let f2 = gf!();
            let fp = icarus.interface_export.i_camera_roll;
            fp(icarus, host, f, f2);
        }
        TYPE_FOLLOW => {
            let s = gs!();
            let f = gf!();
            let f2 = gf!();
            let fp = icarus.interface_export.i_camera_follow;
            fp(icarus, host, &s, f, f2);
        }
        TYPE_TRACK => {
            let s = gs!();
            let f = gf!();
            let f2 = gf!();
            let fp = icarus.interface_export.i_camera_track;
            fp(icarus, host, &s, f, f2);
        }
        TYPE_DISTANCE => {
            let f = gf!();
            let f2 = gf!();
            let fp = icarus.interface_export.i_camera_distance;
            fp(icarus, host, f, f2);
        }
        TYPE_FADE => {
            let v = gv!();
            let f = gf!();
            let v2 = gv!();
            let f2 = gf!();
            let f3 = gf!();
            let fp = icarus.interface_export.i_camera_fade;
            fp(icarus, host, v[0], v[1], v[2], f, v2[0], v2[1], v2[2], f2, f3);
        }
        TYPE_PATH => {
            let s = gs!();
            let fp = icarus.interface_export.i_camera_path;
            fp(icarus, host, &s);
        }
        TYPE_ENABLE => {
            let fp = icarus.interface_export.i_camera_enable;
            fp(icarus, host);
        }
        TYPE_DISABLE => {
            let fp = icarus.interface_export.i_camera_disable;
            fp(icarus, host);
        }
        TYPE_SHAKE => {
            let f = gf!();
            let f2 = gf!();
            let fp = icarus.interface_export.i_camera_shake;
            fp(icarus, host, f, f2 as i32);
        }
        _ => {}
    }

    tm.completed(task.m_id);
}

/// Raven `CTaskManager::Move`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1425-1454`
fn move_(icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let block = &task.m_block;
    let mut member_num = 0;

    // Get the goal position.
    let vector = match get_vector(icarus, host, owner_id, block, &mut member_num) {
        Some(v) => v,
        None => return,
    };

    // Check for a possible angles field.
    match get_vector(icarus, host, owner_id, block, &mut member_num) {
        None => {
            let duration = match get_float(icarus, host, owner_id, block, &mut member_num) {
                Some(v) => v,
                None => return,
            };
            let f = icarus.interface_export.i_lerp2_pos;
            // Raven passes `NULL` angles; the frozen `I_Lerp2Pos` is always
            // present, so the zero-vector stands (Q3_Lerp2Pos drops it).
            f(icarus, host, task.m_id, owner_id, vector, [0.0; 3], duration);
        }
        Some(vector2) => {
            let duration = match get_float(icarus, host, owner_id, block, &mut member_num) {
                Some(v) => v,
                None => return,
            };
            let f = icarus.interface_export.i_lerp2_pos;
            f(icarus, host, task.m_id, owner_id, vector, vector2, duration);
        }
    }
}

/// Raven `CTaskManager::Kill`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1462-1476`
fn kill(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_kill;
    f(icarus, host, owner_id, &s_val);
    tm.completed(task.m_id);
}

/// Raven `CTaskManager::Set`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1484-1497`
fn set(icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let s_val2 = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_set;
    f(icarus, host, task.m_id, owner_id, &s_val, &s_val2);
}

/// Raven `CTaskManager::Use`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1505-1519`
fn use_(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_use;
    f(icarus, host, owner_id, &s_val);
    tm.completed(task.m_id);
}

/// Raven `CTaskManager::DeclareVariable`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1527-1544`
fn declare_variable(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let block = &task.m_block;
    let mut member_num = 0;
    let f_val = match get_float(icarus, host, owner_id, block, &mut member_num) {
        Some(v) => v,
        None => return,
    };
    let s_val = match get(icarus, host, owner_id, block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_declare_variable;
    f(icarus, host, f_val as i32, &s_val);
    tm.completed(task.m_id);
}

/// Raven `CTaskManager::FreeVariable`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1552-1567`
fn free_variable(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_free_variable;
    f(icarus, host, &s_val);
    tm.completed(task.m_id);
}

/// Raven `CTaskManager::Signal` — raises a signal on the owning instance.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1575-1589`
fn signal(tm: &mut TaskManager, icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, &task.m_block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    // `m_owner->GetOwner()->Signal(sVal)` → the instance signal map.
    if let Some(inst) = icarus.instance.as_mut() {
        inst.signal(&s_val);
    }
    tm.completed(task.m_id);
}

/// Raven `CTaskManager::Play`.
/// Source: `oracle/codemp/icarus/TaskManager.cpp:1597-1610`
fn play(icarus: &mut Icarus, host: &mut dyn EngineHost, owner_id: i32, task: &Task) {
    let block = &task.m_block;
    let mut member_num = 0;
    let s_val = match get(icarus, host, owner_id, block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let s_val2 = match get(icarus, host, owner_id, block, &mut member_num) {
        Some(s) => s,
        None => return,
    };
    let f = icarus.interface_export.i_play;
    f(icarus, host, task.m_id, owner_id, &s_val, &s_val2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_seeds_a_default_scheduler() {
        let tm = TaskManager::create();
        assert!(tm.m_task_groups.is_empty());
        assert!(tm.m_tasks.is_empty());
        assert_eq!(tm.m_guid, 0);
        assert!(tm.m_cur_group.is_none());
        assert!(!tm.m_resident);
    }

    #[test]
    fn flush_returns_true_not_task_ok() {
        // Raven's stub returns `true` (== 1), which is distinct from TASK_OK (0).
        let mut tm = TaskManager::default();
        assert_eq!(tm.flush(), 1);
        assert_ne!(tm.flush(), TASK_OK);
    }

    #[test]
    fn is_running_tracks_pending_tasks() {
        let mut tm = TaskManager::default();
        assert!(!tm.is_running());
        // A bare `TaskManager` starts idle; adding a task group does not enqueue
        // tasks, so `is_running` stays false.
        tm.add_task_group("g");
        assert!(!tm.is_running());
    }

    #[test]
    fn add_task_group_allocates_stamps_and_indexes() {
        let mut tm = TaskManager::default();

        let a = tm.add_task_group("alpha");
        let b = tm.add_task_group("beta");

        // Handles are arena slots; GUIDs advance via `m_GUID++`.
        assert_eq!(a, TaskGroupId(0));
        assert_eq!(b, TaskGroupId(1));
        assert_eq!(tm.m_task_groups.len(), 2);
        assert_eq!(tm.m_guid, 2);
        assert_eq!(tm.m_task_groups[0].m_guid, 0);
        assert_eq!(tm.m_task_groups[1].m_guid, 1);

        // Both side-indexes resolve to the same handle.
        assert_eq!(tm.m_task_group_name_map["alpha"], a);
        assert_eq!(tm.m_task_group_id_map[&0], a);
        assert_eq!(tm.m_task_group_id_map[&1], b);
    }

    #[test]
    fn add_task_group_reuses_and_resets_existing_name() {
        let mut tm = TaskManager::default();
        let a = tm.add_task_group("alpha");

        // Dirty the existing group's completion state.
        tm.m_task_groups[a.0 as usize].m_completed_tasks.insert(7, true);
        tm.m_task_groups[a.0 as usize].m_num_completed = 3;

        // Re-adding the same name returns the same handle and reruns `Init`
        // (clears completion state) without allocating or advancing the GUID.
        let again = tm.add_task_group("alpha");
        assert_eq!(again, a);
        assert_eq!(tm.m_task_groups.len(), 1);
        assert_eq!(tm.m_guid, 1);
        assert!(tm.m_task_groups[a.0 as usize].m_completed_tasks.is_empty());
        assert_eq!(tm.m_task_groups[a.0 as usize].m_num_completed, 0);
    }

    #[test]
    fn push_pop_task_front_and_back() {
        let mut tm = TaskManager::default();
        let mk = |id: i32| {
            Task::create(
                id,
                Block {
                    m_members: Vec::new(),
                    m_id: 0,
                    m_flags: 0,
                },
            )
        };
        tm.push_task(mk(1), PUSH_BACK);
        tm.push_task(mk(2), PUSH_BACK);
        tm.push_task(mk(0), PUSH_FRONT);
        // Order is [0, 1, 2]; POP_BACK returns the last.
        assert_eq!(tm.pop_task(POP_BACK).unwrap().m_id, 2);
        assert_eq!(tm.pop_task(POP_FRONT).unwrap().m_id, 0);
        assert_eq!(tm.pop_task(POP_BACK).unwrap().m_id, 1);
        assert!(tm.pop_task(POP_BACK).is_none());
    }
}
