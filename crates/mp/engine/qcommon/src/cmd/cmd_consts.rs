#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MAX_CMD_BUFFER` — size of the deferred command-text ring buffer.
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:7`
pub const MAX_CMD_BUFFER: usize = 16384;

/// Raven `MAX_CMD_LINE` — max length of a single buffered command line.
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:8`
pub const MAX_CMD_LINE: usize = 1024;
