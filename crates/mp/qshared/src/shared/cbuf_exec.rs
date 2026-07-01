#![allow(non_camel_case_types)]

/// Raven `cbufExec_t` command-buffer stuffing modes.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:405-410`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cbufExec_t {
    /// Raven: don't return until completed, a VM should NEVER use this,
    /// because some commands might cause the VM to be unloaded...
    EXEC_NOW,
    /// Raven: insert at current position, but don't run yet
    EXEC_INSERT,
    /// Raven: add to end of the command buffer (normal case)
    EXEC_APPEND,
}
