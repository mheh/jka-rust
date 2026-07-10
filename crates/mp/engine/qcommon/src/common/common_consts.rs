#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MAXPRINTMSG` — max size of a single `Com_Printf`/`Com_Error` message
/// buffer.
/// Source: `oracle/codemp/qcommon/common.cpp:18`
pub const MAXPRINTMSG: usize = 4096;

/// Raven `MAX_NUM_ARGVS` — max `com_argv` command-line argument slots.
/// Source: `oracle/codemp/qcommon/common.cpp:20`
pub const MAX_NUM_ARGVS: usize = 50;

/// Raven `MAX_CONSOLE_LINES` — max deferred `+`-separated console command
/// lines parsed from the command line.
/// Source: `oracle/codemp/qcommon/common.cpp:386`
pub const MAX_CONSOLE_LINES: usize = 32;

/// Raven `MAX_PUSHED_EVENTS` — size of the `com_pushedEvents` ring buffer.
/// Source: `oracle/codemp/qcommon/common.cpp:747`
pub const MAX_PUSHED_EVENTS: usize = 1024;
