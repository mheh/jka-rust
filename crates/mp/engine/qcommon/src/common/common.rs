//! `Common` — the qcommon-owned engine state + `com_printf` (STATE-D11 / LIFE-D2).

use std::time::Instant;

use super::error::ErrorState;
use super::journal::Journal;
use super::sys_event_queue::SysEventQueue;
use crate::vm::module_registry::ModuleRegistry;

/// The `common.cpp` global state, a field of the aggregate `Engine`
/// (`mp_engine_core`). Owns the `com_frameTime`/error/journal/event/module-table
/// state plus the `cvars`/`cmd`/`cbuf`/`fs`/`net` sub-structs (subsystem detail,
/// not frozen here). Hosts `com_printf` (STATE-D11) and reaches the receiverless
/// `com_error` (STATE-D7).
///
/// Field types below the lifecycle-named set are `common`-module subsystem
/// detail (state-ownership treats each owned struct's field list as a non-goal).
///
/// Source: `oracle/oracle/codemp/qcommon/common.cpp:22-94`
pub struct Common {
    /// `com_frameTime`/`com_frameMsec`/`com_frameNumber` (`common.cpp:79-81`).
    pub frame_time: i32,
    pub frame_msec: i32,
    pub frame_number: i32,
    /// `Com_Frame` `static int lastTime` (`common.cpp:1601`; §B3 fn-static hoist).
    pub frame_last_time: i32,
    /// `com_fullyInitialized` (`common.cpp:84`).
    pub fully_initialized: bool,
    /// `com_errorEntered`/`com_errorMessage` + MP rapid-error statics.
    pub error: ErrorState,
    /// `com_journalFile`/`com_journalDataFile` + `journal` mode (MP only).
    pub journal: Journal,
    /// `eventQue[256]` (`Sys_QueEvent` ring; distinct from `com_pushedEvents`).
    pub sys_events: SysEventQueue,
    /// `vmTable[MAX_VM]` replacement (LIFE-Q5 / STATE-D10) — the module registry
    /// nests here, `engine.common.modules`.
    pub modules: ModuleRegistry,
    /// `sys_timeBase` (`win_shared.cpp:22-34`): the `std::time::Instant` base,
    /// captured in `Engine::new()` (LIFE-D4b), read-only afterward.
    pub time_base: Instant,
    //TODO: Port Common cvars/cmd/cbuf/fs/net sub-structs + com_printf print state
    // Source: oracle/oracle/codemp/qcommon/common.cpp:32-72,128,137-171
}

/// Raven `Com_Printf` (`common.cpp:128`). Threads `&mut Common` and lives in
/// `mp_engine_qcommon` (com_printf resolution, LIFE-D2 amendment) — mutates the
/// redirect buffer (`rd_buffer`), console, and the lazily-opened `logfile`, all
/// `Common` state. Reachable from every engine crate; `core` callers pass
/// `&mut engine.common`.
///
/// Source: `oracle/oracle/codemp/qcommon/common.cpp:128`
pub fn com_printf(common: &mut Common, msg: &str) {
    let _ = (common, msg);
    todo!("Port Com_Printf — oracle/oracle/codemp/qcommon/common.cpp:128")
}
