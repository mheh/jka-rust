//! `SysEventQueue` — the 256-entry `Sys_QueEvent` ring (LIFE frozen).

use crate::qcommon::sys_event_t::sysEvent_t;

/// `MAX_QUED_EVENTS` (`win_main.cpp:1162`).
pub const MAX_QUED_EVENTS: usize = 256;

/// `MASK_QUED_EVENTS` (`win_main.cpp:1163`) — `MAX_QUED_EVENTS - 1`, the ring
/// index mask.
pub const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;

/// Faithful queue semantics of `eventQue[256]` (`win_main.cpp:1162-1203`). NOT
/// the 1024-entry `com_pushedEvents` ring (`Common.event_queue`).
///
/// The `Sys_QueEvent`/`Sys_GetEvent` logic lives in `sys_engine.rs` (it threads
/// `Common` for `Z_Free`/`Sys_Milliseconds`), so the ring exposes its fields
/// `pub(crate)` rather than owning the behavior.
///
/// Source: `oracle/codemp/win32/win_main.cpp:1162-1166`
pub struct SysEventQueue {
    /// `eventQue[MAX_QUED_EVENTS]`.
    pub(crate) que: [sysEvent_t; MAX_QUED_EVENTS],
    /// `eventHead`/`eventTail`, monotonic; `& MASK_QUED_EVENTS` to index.
    pub(crate) head: i32,
    pub(crate) tail: i32,
}
