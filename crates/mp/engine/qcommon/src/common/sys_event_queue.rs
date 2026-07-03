//! `SysEventQueue` — the 256-entry `Sys_QueEvent` ring (LIFE frozen).

use crate::qcommon::sys_event_t::sysEvent_t;

/// `MAX_QUED_EVENTS` (`win_main.cpp:1162`).
pub const MAX_QUED_EVENTS: usize = 256;

/// Faithful queue semantics of `eventQue[256]` (`win_main.cpp:1162-1203`). NOT
/// the 1024-entry `com_pushedEvents` ring (`Common.event_queue`).
///
/// Source: `oracle/oracle/codemp/win32/win_main.cpp:1162-1166`
pub struct SysEventQueue {
    /// `eventQue[MAX_QUED_EVENTS]`.
    que: [sysEvent_t; MAX_QUED_EVENTS],
    /// Monotonic; `& (MAX_QUED_EVENTS-1)` to index.
    head: usize,
    tail: usize,
}
