//! Slice-0 boot-success no-op stubs (LIFE-Q8, user-settled round-7 item 26).
//!
//! `com_init` steps 3/5/7/12 are deliberately-callable no-ops so the headless
//! dedicated spine boots before the B1 (cvar/cmd) and B2 (filesystem)
//! subsystem ports land; DEC-09.2 boot-transcript diffing activates then.
//! Each carries the mandated marker + one-line justification
//! (porting-rules: deliberately-callable no-op).

/// Raven `Cvar_Init` — com_init step 3.
//TODO: Port Cvar_Init
// Source: oracle/codemp/qcommon/cvar.cpp (Cvar_Init); com_init step 3, common.cpp:1226
// Deliberate no-op: boot-success stub until the B1 cvar port (LIFE-Q8).
pub fn cvar_init() {}

/// Raven `Cbuf_Init` — com_init step 5.
//TODO: Port Cbuf_Init
// Source: oracle/codemp/qcommon/cmd_common.cpp (Cbuf_Init); com_init step 5, common.cpp:1233
// Deliberate no-op: boot-success stub until the B1 cmd/cbuf port (LIFE-Q8).
pub fn cbuf_init() {}

/// Raven `Cmd_Init` — com_init step 7.
//TODO: Port Cmd_Init
// Source: oracle/codemp/qcommon/cmd_common.cpp (Cmd_Init); com_init step 7, common.cpp:1242
// Deliberate no-op: boot-success stub until the B1 cmd port (LIFE-Q8).
pub fn cmd_init() {}

/// Raven `FS_InitFilesystem` — com_init step 12.
//TODO: Port FS_InitFilesystem
// Source: oracle/codemp/qcommon/files_common.cpp (FS_InitFilesystem); com_init step 12, common.cpp:1266
// Deliberate no-op: boot-success stub until the B2 filesystem port (LIFE-Q8);
// the real step ERR_FATALs on an unreadable mpdefault.cfg.
pub fn fs_init_filesystem() {}
