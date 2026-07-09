//! MP `bg_public.h` team objective task enumeration.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1034-1043`

#![allow(non_camel_case_types)]

/// Raven `teamtask_t` — team objective task assignment.
///
/// Raven: Enumeration defining team-oriented objectives and tasks that can be
/// assigned to players in team-based game modes.
/// Type definition source: `oracle/codemp/game/bg_public.h:1034-1043`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum teamtask_t {
    TEAMTASK_NONE = 0,
    TEAMTASK_OFFENSE = 1,
    TEAMTASK_DEFENSE = 2,
    TEAMTASK_PATROL = 3,
    TEAMTASK_FOLLOW = 4,
    TEAMTASK_RETRIEVE = 5,
    TEAMTASK_ESCORT = 6,
    TEAMTASK_CAMP = 7,
}
