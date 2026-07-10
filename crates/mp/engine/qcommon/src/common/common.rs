//! `Common` — the qcommon-owned engine state + `com_printf` (STATE-D11 / LIFE-D2).

use core::ffi::{c_char, c_int};
use std::time::Instant;

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::cvar::cvar_t;
use mp_qshared::shared::fileHandle_t;
use mp_qshared::shared::limits::{
    BIG_INFO_STRING, MAX_STRING_CHARS, MAX_STRING_TOKENS, MAX_TOKEN_CHARS,
};
use mp_qshared::shared::qboolean;

use crate::qcommon::net_chan_cpp_consts::MAX_LOOPBACK;
use crate::z_memman::zone_header_s::zoneHeader_t;

use super::error::ErrorState;
use super::journal::Journal;
use super::qrand::QRand;
use super::sys_event_queue::SysEventQueue;
use crate::cmd::cmd_consts::MAX_CMD_BUFFER;
use crate::common::common_consts::{MAX_CONSOLE_LINES, MAX_PUSHED_EVENTS};
use crate::files::file_handle_data_t::fileHandleData_t;
use crate::files::files_consts::MAX_SEARCH_PATHS;
use crate::files::searchpath_s::searchpath_t;
use crate::qcommon::filesystem_limits::MAX_FILE_HANDLES;
use crate::qcommon::sys_event_t::sysEvent_t;
use crate::vm::elastcommand::ELastCommand;
use crate::vm::module_registry::ModuleRegistry;
use crate::vm::module_registry::MAX_VM;
use crate::vm::vm_s::vm_t;
use crate::vm::vm_symbol_s::vmSymbol_t;

/// Raven `cmd_t` — the command-buffer descriptor (`data`/`maxsize`/`cursize`).
/// No rosetta row; resolved verbatim here as the shape of `Common::cmd_text`
/// (`cmd_common.cpp` PORT-NOTE), one type per its owning `Common` file.
///
/// Type definition source: `oracle/codemp/qcommon/cmd_common.cpp:10-14`
#[repr(C)]
pub struct cmd_t {
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
}

/// Raven `zoneStats_t` (`z_memman_pc.cpp:56-65`) — the zone allocator's
/// running byte/count totals, per-tag broken down over `memtag_t::TAG_COUNT`
/// slots. No rosetta row; resolved verbatim here as the shape of
/// `Common::TheZone.Stats` (`cmd_t` precedent above).
///
/// Type definition source: `oracle/codemp/qcommon/z_memman_pc.cpp:56-65`
#[repr(C)]
pub struct zoneStats_t {
    pub iCount: c_int,
    pub iCurrent: c_int,
    pub iPeak: c_int,
    pub iSizesPerTag: [c_int; memtag_t::TAG_COUNT as usize],
    pub iCountsPerTag: [c_int; memtag_t::TAG_COUNT as usize],
}

/// Raven `zone_t` (`z_memman_pc.cpp:68-72`) — `TheZone`'s aggregate shape
/// (stats + the allocation-list header). No rosetta row; resolved verbatim
/// here as the shape of `Common::TheZone` (`cmd_t` precedent above).
///
/// Type definition source: `oracle/codemp/qcommon/z_memman_pc.cpp:68-72`
#[repr(C)]
pub struct zone_t {
    pub Stats: zoneStats_t,
    pub Header: zoneHeader_t,
}

/// Raven `loopmsg_t` (`net_chan.cpp:477-480`) — one buffered loopback packet.
/// No rosetta row; resolved verbatim here as the shape of
/// `Common::loopbacks` (`cmd_t` precedent above).
///
/// Type definition source: `oracle/codemp/qcommon/net_chan.cpp:477-480`
#[repr(C)]
pub struct loopmsg_t {
    pub data: [u8; crate::qcommon::net_chan_cpp_consts::MAX_PACKETLEN as usize],
    pub datalen: c_int,
}

/// Raven `loopback_t` (`net_chan.cpp:482-484`) — the localhost transport's
/// per-direction message ring. No rosetta row; resolved verbatim here as the
/// shape of `Common::loopbacks` (`cmd_t` precedent above).
///
/// Type definition source: `oracle/codemp/qcommon/net_chan.cpp:482-484`
#[repr(C)]
pub struct loopback_t {
    pub msgs: [loopmsg_t; MAX_LOOPBACK as usize],
    pub get: c_int,
    pub send: c_int,
}

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
    /// Raven `q_math.c`'s file-static `holdrand` LCG — the engine island's OWN
    /// generator instance (ruling 21), distinct from the game-tier
    /// `BgState.rng`. Written through its `Default` (Raven's `0x89abcdef`
    /// static initializer) in `Engine::new()`'s write-list, then reseeded by
    /// `Com_Init`'s `Rand_Init(Sys_Milliseconds())`.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    pub qrand: QRand,
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

    // ---- `common.cpp` file-scope globals (verbatim Raven names) ----
    /// Raven `com_frameTime`/`com_frameMsec`/`com_frameNumber` (int).
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:79-81`
    pub com_frameTime: c_int,
    pub com_frameMsec: c_int,
    pub com_frameNumber: c_int,
    /// Raven `com_errorEntered` / `com_fullyInitialized`.
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:83-84`
    pub com_errorEntered: qboolean,
    pub com_fullyInitialized: bool,
    /// Raven `com_journalFile` / `com_journalDataFile` (config/event journal).
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:34-35`
    pub com_journalFile: fileHandle_t,
    pub com_journalDataFile: fileHandle_t,
    /// Raven `com_*` cvar pointers.
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:37-75`
    pub com_speeds: *mut cvar_t,
    pub com_viewlog: *mut cvar_t,
    pub com_developer: *mut cvar_t,
    pub com_vmdebug: *mut cvar_t,
    pub com_dedicated: *mut cvar_t,
    pub com_timescale: *mut cvar_t,
    pub com_fixedtime: *mut cvar_t,
    pub com_dropsim: *mut cvar_t,
    pub com_journal: *mut cvar_t,
    pub com_maxfps: *mut cvar_t,
    pub com_timedemo: *mut cvar_t,
    pub com_sv_running: *mut cvar_t,
    pub com_cl_running: *mut cvar_t,
    pub com_logfile: *mut cvar_t,
    pub com_showtrace: *mut cvar_t,
    pub com_optvehtrace: *mut cvar_t,
    pub com_G2Report: *mut cvar_t,
    pub com_terrainPhysics: *mut cvar_t,
    pub com_version: *mut cvar_t,
    pub com_blood: *mut cvar_t,
    pub com_buildScript: *mut cvar_t,
    pub com_introPlayed: *mut cvar_t,
    pub cl_paused: *mut cvar_t,
    pub sv_paused: *mut cvar_t,
    pub com_cameraMode: *mut cvar_t,
    pub com_RMG: *mut cvar_t,
    pub com_validateZone: *mut cvar_t,
    /// Raven `Com_EventLoop`'s `random()` drop-sim seed (fn-static hoist).
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:900-915`
    pub com_eventloop_seed: c_int,
    /// Raven `com_pushedEvents`/`com_pushedEventsHead`/`com_pushedEventsTail`
    /// (the `Com_PushEvent` ring; distinct from the `Sys_QueEvent` queue).
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:749-752`
    pub com_pushedEvents: [sysEvent_t; MAX_PUSHED_EVENTS],
    pub com_pushedEventsHead: c_int,
    pub com_pushedEventsTail: c_int,
    /// Raven `com_numConsoleLines` / `com_consoleLines[MAX_CONSOLE_LINES]`.
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:387-388`
    pub com_numConsoleLines: c_int,
    pub com_consoleLines: [*mut c_char; MAX_CONSOLE_LINES],
    /// Raven `Com_BeginRedirect` state: `rd_buffer`/`rd_buffersize`/`rd_flush`
    /// (`rd_flush` is Raven's `void (*)(char *)` redirect callback pointer).
    ///
    /// Source: `oracle/codemp/qcommon/common.cpp:90-93`
    pub rd_buffer: *mut c_char,
    pub rd_buffersize: c_int,
    pub rd_flush: *mut extern "C" fn(*mut c_char),

    // ---- `cmd_common.cpp` / `cmd_pc.cpp` command system ----
    /// Raven `cmd_wait` / `cmd_argc`.
    ///
    /// Source: `oracle/codemp/qcommon/cmd_common.cpp:16,290`
    pub cmd_wait: c_int,
    pub cmd_argc: c_int,
    /// Raven `cmd_argv[MAX_STRING_TOKENS]` (points into `cmd_tokenized`) and the
    /// `cmd_tokenized` scratch.
    ///
    /// Source: `oracle/codemp/qcommon/cmd_common.cpp:291-292`
    pub cmd_argv: [*mut c_char; MAX_STRING_TOKENS],
    pub cmd_tokenized: [c_char; BIG_INFO_STRING + MAX_STRING_TOKENS],
    /// Raven `cmd_text` (`cmd_t`) + its backing `cmd_text_buf[MAX_CMD_BUFFER]`.
    ///
    /// Source: `oracle/codemp/qcommon/cmd_common.cpp:17-18`
    pub cmd_text: cmd_t,
    pub cmd_text_buf: [u8; MAX_CMD_BUFFER],
    /// Raven `Cmd_Args`'s `static char cmd_args[MAX_STRING_CHARS]` and
    /// `Cmd_ArgsFrom`'s `static char cmd_args[BIG_INFO_STRING]` (fn-static hoists,
    /// three-kind rule).
    ///
    /// Source: `oracle/codemp/qcommon/cmd_common.cpp:337,359`
    pub cmd_args_buf: [c_char; MAX_STRING_CHARS],
    pub cmd_args_from_buf: [c_char; BIG_INFO_STRING],
    /// Raven `static cmd_function_t *cmd_functions` — head of the
    /// registered-command linked list.
    ///
    /// Source: `oracle/codemp/qcommon/cmd_pc.cpp:11`
    pub cmd_functions: *mut crate::cmd_pc::cmd_function_t,

    // ---- `cvar.cpp` ----
    /// Raven `cvar_modifiedFlags`.
    ///
    /// Source: `oracle/codemp/qcommon/cvar.cpp:8`
    pub cvar_modifiedFlags: c_int,

    // ---- collision (`cm_load.cpp`) trace counters ----
    /// Raven `c_pointcontents` / `c_traces` / `c_brush_traces` / `c_patch_traces`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_load.cpp:38-39`
    pub c_pointcontents: c_int,
    pub c_traces: c_int,
    pub c_brush_traces: c_int,
    pub c_patch_traces: c_int,

    // ---- filesystem (`files_common.cpp` / `files_pc.cpp`) ----
    /// Raven `fs_searchpaths` / `fsh[MAX_FILE_HANDLES]`.
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:193,279`
    pub fs_searchpaths: *mut searchpath_t,
    pub fsh: [fileHandleData_t; MAX_FILE_HANDLES],
    /// Raven `fs_*` cvar pointers.
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:184-241`
    pub fs_debug: *mut cvar_t,
    pub fs_basepath: *mut cvar_t,
    pub fs_cdpath: *mut cvar_t,
    pub fs_homepath: *mut cvar_t,
    pub fs_gamedirvar: *mut cvar_t,
    /// Raven `fs_gamedir[MAX_OSPATH]` (single game-dir name).
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:183`
    pub fs_gamedir: [c_char; MAX_OSPATH],
    /// Raven `fs_checksumFeed` / `fs_fakeChkSum` / `fs_reordered`.
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:199-200`
    pub fs_checksumFeed: c_int,
    pub fs_fakeChkSum: c_int,
    pub fs_reordered: qboolean,
    /// Raven pure/referenced server-pak tables.
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:207-290`
    pub fs_numServerPaks: c_int,
    pub fs_serverPaks: [c_int; MAX_SEARCH_PATHS],
    pub fs_numServerReferencedPaks: c_int,
    pub fs_serverReferencedPaks: [c_int; MAX_SEARCH_PATHS],
    pub fs_serverReferencedPakNames: [*mut c_char; MAX_SEARCH_PATHS],
    /// Raven `FS_*Checksums`/`FS_*Names` rotating `static char` return buffers
    /// (fn-static hoists, three-kind rule): `FS_GamePureChecksum` uses
    /// `MAX_STRING_TOKENS`; the loaded/referenced variants use `BIG_INFO_STRING`.
    ///
    /// Source: `oracle/codemp/qcommon/files_pc.cpp:2647,2673,2699,2729,2755,2784,2833`
    pub fs_game_pure_checksum_info: [c_char; MAX_STRING_TOKENS],
    pub fs_loaded_pak_checksums_info: [c_char; BIG_INFO_STRING],
    pub fs_loaded_pak_names_info: [c_char; BIG_INFO_STRING],
    pub fs_loaded_pak_pure_checksums_info: [c_char; BIG_INFO_STRING],
    pub fs_referenced_pak_checksums_info: [c_char; BIG_INFO_STRING],
    pub fs_referenced_pak_names_info: [c_char; BIG_INFO_STRING],
    pub fs_referenced_pak_pure_checksums_info: [c_char; BIG_INFO_STRING],

    // ---- networking (`net_chan.cpp`) ----
    /// Raven `showpackets`/`showdrop` (`cvar_t*` in Raven; collapsed to the
    /// cached `->integer` per the module's net-cvar PORT-NOTE) and
    /// `net_qport`/`net_killdroppedfragments` (same collapse).
    ///
    /// Source: `oracle/codemp/qcommon/net_chan.cpp:40-43`
    pub showpackets: c_int,
    pub showdrop: c_int,
    pub net_qport: c_int,
    pub net_killdroppedfragments: c_int,
    /// Raven `NET_AdrToString`'s `static char s[64]` rotating return buffer
    /// (fn-static hoist, three-kind rule).
    ///
    /// Source: `oracle/codemp/qcommon/net_chan.cpp:408`
    pub net_adr_to_string_buf: [c_char; 64],

    // ---- zone allocator (`z_memman_pc.cpp`) ----
    /// Raven `hunk_tag` (`Hunk_ClearToMark`/`Hunk_SetMark` alternation flag).
    ///
    /// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:60`
    pub hunk_tag: memtag_t,

    // ---- VM core (`vm.cpp`) ----
    /// Raven `currentVM` / `lastVM` / `gvm` (game VM) VM pointers and the
    /// `vmTable[MAX_VM]` array + `vm_debugLevel`.
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:24-29`, `oracle/codemp/server/server.h:234`
    pub currentVM: *mut vm_t,
    pub lastVM: *mut vm_t,
    pub gvm: *mut vm_t,
    pub vmTable: [vm_t; MAX_VM],
    pub vm_debugLevel: c_int,
    /// Raven `VM_ValueToSymbol`'s `static char text[MAX_TOKEN_CHARS]` and
    /// `VM_ValueToFunctionSymbol`'s `static vmSymbol_t nullSym` (fn-static hoists).
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:72,115`
    pub vm_value_to_symbol_buf: [c_char; MAX_TOKEN_CHARS],
    pub vm_value_to_function_symbol_null_sym: vmSymbol_t,
    /// Raven `VM_LogSyscalls`'s `static int callnum` / `static FILE *f`
    /// (fn-static hoists).
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:935-936`
    pub vm_log_syscalls_callnum: c_int,
    pub vm_log_syscalls_f: *mut libc::FILE,

    // ---- x86 VM JIT (`vm_x86.cpp`) ----
    /// Raven `vm_x86.cpp` compiler file-scope statics: `buf`/`jused`/`code`
    /// (emit buffers), `compiledOfs`/`pc`, `instructionPointers`, the peephole
    /// registers `instruction`/`pass`/`lastConst`/`oc0`/`oc1`/`pop0`/`pop1`,
    /// `LastCommand`, and `asmCallPtr`.
    ///
    /// Source: `oracle/codemp/qcommon/vm_x86.cpp:27-83`
    pub buf: *mut u8,
    pub jused: *mut u8,
    pub code: *mut u8,
    pub compiled_ofs: c_int,
    pub pc: c_int,
    pub instruction_pointers: *mut c_int,
    pub instruction: c_int,
    pub pass: c_int,
    pub last_const: c_int,
    pub oc0: c_int,
    pub oc1: c_int,
    pub pop0: c_int,
    pub pop1: c_int,
    pub last_command: ELastCommand,
    pub asm_call_ptr: c_int,
    /// Raven `vm_x86.cpp` `DoSyscall` bridge statics: `programStack`/`opStack`/
    /// `syscallNum`/`savedVM` (renamed to avoid the `vm_t` field clashes).
    ///
    /// Source: `oracle/codemp/qcommon/vm_x86.cpp:92-95`
    pub call_program_stack: c_int,
    pub call_op_stack: *mut c_int,
    pub call_syscall_num: c_int,
    pub current_vm: *mut vm_t,

    // ---- zone allocator (`z_memman_pc.cpp`) ----
    /// Raven `TheZone` (the zone allocator's stats + allocation-list header).
    ///
    /// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:77`
    pub TheZone: zone_t,

    // ---- filesystem (`files_common.cpp`) fn-static / init-state hoists ----
    /// Raven `FS_BuildOSPath`'s `static char ospath[2][MAX_OSPATH]` /
    /// `static int toggle` (fn-static hoists, three-kind rule).
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:296-297`
    pub fs_build_os_path_buf: [[c_char; MAX_OSPATH]; 2],
    pub fs_build_os_path_toggle: c_int,
    /// Raven `FS_BuildOSPath` (`base`/`game`/`qpath` overload)'s
    /// `static char ospath[4][MAX_OSPATH]` / `static int toggle` — a
    /// SEPARATE fn-scope-static pair from the single-`qpath` overload above
    /// (Raven gives each overload its own statics).
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:315-317`
    pub fs_build_os_path4_buf: [[c_char; MAX_OSPATH]; 4],
    pub fs_build_os_path4_toggle: c_int,
    /// Raven `fs_loadStack` — total files currently loaded into memory.
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:196`
    pub fs_loadStack: c_int,
    /// Raven `initialized` — whether `FS_InitFilesystem` has completed.
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:224`
    pub initialized: qboolean,
    /// Raven `lastValidBase` / `lastValidGame` — the last known-good
    /// basepath/gamedir, restored on a failed `FS_SetBaseDir`/pure check.
    ///
    /// Source: `oracle/codemp/qcommon/files_common.cpp:217-218`
    pub lastValidBase: [c_char; MAX_OSPATH],
    pub lastValidGame: [c_char; MAX_OSPATH],

    // ---- networking (`net_chan.cpp`) ----
    /// Raven `loopbacks[2]` — the localhost transport's per-direction
    /// message rings.
    ///
    /// Source: `oracle/codemp/qcommon/net_chan.cpp:486`
    pub loopbacks: [loopback_t; 2],

    // ---- server bot glue (`sv_bot.cpp`) ----
    /// Raven `bot_enable` — cached `bot_enable` cvar integer (mirrors the
    /// `cl_shownet` PORT-NOTE collapse above pending the cvar-registry wave).
    ///
    /// Source: `oracle/codemp/server/sv_bot.cpp:20`
    pub bot_enable: c_int,
}

/// Raven `#define MAX_OSPATH PATH_MAX` (1024 here, matching the FS field sizes).
///
/// Source: `oracle/codemp/qcommon/q_shared.h` (`MAX_OSPATH`)
const MAX_OSPATH: usize = 1024;

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
