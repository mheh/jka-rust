//! Raven `CTaskGroup` — a completion-tracking task group.

use std::collections::BTreeMap;

use crate::taskmanager::ctask::Task;
use crate::taskmanager::ctask_manager::TaskGroupId;

/// Raven `CTaskGroup` → `TaskGroup` (§F idiomatic, ICARUS-D1 naming).
///
/// A completion-tracking group; owned in `TaskManager`'s `Vec<TaskGroup>` arena
/// keyed by `TaskGroupId` (ICARUS-D3 / ruling 27). `m_completedTasks` (Raven's
/// already-int-keyed `map<int,bool>`) → `BTreeMap<i32, bool>`; the raw
/// `CTaskGroup *m_parent` back-pointer → `Option<TaskGroupId>`.
/// Type definition source: `oracle/codemp/icarus/taskmanager.h:62-93`
#[derive(Default)]
pub struct TaskGroup {
    /// Raven `taskCallback_m m_completedTasks` (`map<int, bool>`).
    pub m_completed_tasks: BTreeMap<i32, bool>,
    /// Raven `CTaskGroup *m_parent`.
    pub m_parent: Option<TaskGroupId>,
    /// Raven `int m_numCompleted`.
    pub m_num_completed: i32,
    /// Raven `int m_GUID`.
    pub m_guid: i32,
}

/// Raven's anonymous `enum { TASK_OK, TASK_FAILED, TASK_START, TASK_END }` —
/// only `TASK_OK` (the `Add` return value) is used in this file.
/// Source: `oracle/codemp/icarus/taskmanager.h:22-28`
const TASK_OK: i32 = 0;

impl TaskGroup {
    /// Raven `CTaskGroup::Init` — also run by the (dropped, see §20) `CTaskGroup`
    /// constructor before it reseeds `m_GUID`/`m_parent`, both redundant with
    /// this method; the Rust `TaskGroup` is constructed via `#[derive(Default)]`
    /// instead (ICARUS-D3 / ruling 27), so this is the sole reset path.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:101-107`
    pub fn init(&mut self) {
        self.m_completed_tasks.clear();
        self.m_num_completed = 0;
        self.m_parent = None;
    }

    /// Raven `CTaskGroup::Add`.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:115-119`
    pub fn add(&mut self, task: &Task) -> i32 {
        // Raven `m_completedTasks[ task->GetGUID() ] = false` — `GetGUID` reads
        // `CTask::m_id`, ported as `Task::m_id` directly (not `get_id`, which
        // reads through the owned block).
        self.m_completed_tasks.insert(task.m_id, false);
        TASK_OK
    }

    /// Raven `CTaskGroup::SetGUID`.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:90-93`
    pub fn set_guid(&mut self, guid: i32) {
        self.m_guid = guid;
    }

    /// Raven `CTaskGroup::Complete` — all tracked tasks done.
    /// Source: `oracle/codemp/icarus/taskmanager.h:78`
    pub fn complete(&self) -> bool {
        self.m_num_completed == self.m_completed_tasks.len() as i32
    }

    /// Raven `CTaskGroup::MarkTaskComplete`.
    /// Source: `oracle/codemp/icarus/TaskManager.cpp:127-138`
    pub fn mark_task_complete(&mut self, id: i32) -> bool {
        if let Some(done) = self.m_completed_tasks.get_mut(&id) {
            *done = true;
            self.m_num_completed += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockstream::cblock::Block;

    fn task(guid: i32) -> Task {
        Task::create(
            guid,
            Block {
                m_members: Vec::new(),
                m_id: 0,
                m_flags: 0,
            },
        )
    }

    #[test]
    fn mark_task_complete_only_counts_tracked_ids() {
        let mut group = TaskGroup::default();
        group.add(&task(1));
        group.add(&task(2));

        // Unknown id: no-op, reports false, `Complete` still pending.
        assert!(!group.mark_task_complete(99));
        assert!(!group.complete());

        assert!(group.mark_task_complete(1));
        assert!(!group.complete());
        assert!(group.mark_task_complete(2));
        assert!(group.complete());
    }

    #[test]
    fn init_resets_completion_state() {
        let mut group = TaskGroup::default();
        group.add(&task(1));
        group.mark_task_complete(1);
        group.set_guid(5);

        group.init();

        assert!(group.m_completed_tasks.is_empty());
        assert_eq!(group.m_num_completed, 0);
        assert!(group.m_parent.is_none());
        // `Init` does not touch `m_GUID` (matches Raven).
        assert_eq!(group.m_guid, 5);
    }
}
