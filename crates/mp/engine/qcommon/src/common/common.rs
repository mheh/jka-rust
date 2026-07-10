//! `Common` — the qcommon-owned engine state + `com_printf` (STATE-D11 / LIFE-D2).

use std::time::Instant;

use mp_qshared::shared::qboolean;

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
/// Source: `oracle/codemp/qcommon/common.cpp:22-94`
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
    /// `TheStringPackage` (`stringed_ingame.cpp:71`) — the StringEd store, a
    /// `Common` sub-struct field per ruling 50; written through its `Default`
    /// (= Raven's `Clear(SE_FALSE)`) in `Engine::new()`'s write-list (ruling 55).
    pub stringed: crate::stringed::package::StringEdPackage,
    //TODO: Port Common cvars/cmd/cbuf/fs/net sub-structs + com_printf print state
    // Source: oracle/codemp/qcommon/common.cpp:32-72,128,137-171
    /// Raven `msg.cpp` file-scope statics/globals (ruling 2/3): `msgInit`,
    /// `msgHuff`, `oldsize`, `cl_shownet` (collapsed cvar-int read —
    /// PORT-NOTE below), the three `MSG_Read*String*` rotating scratch
    /// buffers, and the `entityStateFields`/`playerStateFields` net-field
    /// tables.
    ///
    /// Source: `oracle/codemp/qcommon/msg.cpp:12,19,21,37`
    pub msg_init: qboolean,
    pub msg_huff: crate::qcommon::huffman_t::huffman_t,
    /// Raven `oldsize` (`msg.cpp:37`).
    pub oldsize: i32,
    //TODO: Port cl_shownet
    // Source: oracle/codemp/qcommon/msg.cpp:12
    // PORT-NOTE(cvar): `cl_shownet` is a `cvar_t*` in Raven; `Common`'s
    // cvar sub-struct isn't landed yet (see the TODO above), so its
    // `->integer` reads collapse to a plain `i32` field here pending the
    // cvar-registry wave.
    pub cl_shownet: i32,
    /// Raven `MSG_ReadString`'s `static char string[MAX_STRING_CHARS]`.
    ///
    /// Source: `oracle/codemp/qcommon/msg.cpp:460`
    pub msg_read_string_buf: [u8; mp_qshared::shared::limits::MAX_STRING_CHARS],
    /// Raven `MSG_ReadBigString`'s `static char string[BIG_INFO_STRING]`.
    ///
    /// Source: `oracle/codemp/qcommon/msg.cpp:498`
    pub msg_read_big_string_buf: [u8; mp_qshared::shared::limits::BIG_INFO_STRING],
    /// Raven `MSG_ReadStringLine`'s `static char string[MAX_STRING_CHARS]`.
    ///
    /// Source: `oracle/codemp/qcommon/msg.cpp:522`
    pub msg_read_string_line_buf: [u8; mp_qshared::shared::limits::MAX_STRING_CHARS],
    //TODO: Port netField_t
    // Source: oracle/codemp/qcommon/qcommon.h (netField_t has no rosetta row
    // at time of transcription — escalated as a missing symbol).
    /// Raven `entityStateFields[]`.
    ///
    /// Source: `oracle/codemp/qcommon/msg.cpp:859-1051`
    pub entity_state_fields: Vec<crate::qcommon::net_field_t::netField_t>,
    /// Raven `playerStateFields[]`.
    ///
    /// Source: `oracle/codemp/qcommon/msg.cpp:1410-1568`
    pub player_state_fields: Vec<crate::qcommon::net_field_t::netField_t>,
}

/// Raven `Com_Printf` (`common.cpp:128`). Threads `&mut Common` and lives in
/// `mp_engine_qcommon` (com_printf resolution, LIFE-D2 amendment) — mutates the
/// redirect buffer (`rd_buffer`), console, and the lazily-opened `logfile`, all
/// `Common` state. Reachable from every engine crate; `core` callers pass
/// `&mut engine.common`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:128`
pub fn com_printf(common: &mut Common, msg: &str) {
    let _ = common;
    //TODO: Port Com_Printf rd_buffer redirect + logfile + console routing
    // Source: oracle/codemp/qcommon/common.cpp:137-181
    // Slice-0 minimal sink: the local-console write only (Sys_Print tail,
    // common.cpp:168); redirect/logfile land with their Common fields.
    print!("{msg}");
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
}
