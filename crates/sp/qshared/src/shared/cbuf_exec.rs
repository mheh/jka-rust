#![allow(non_camel_case_types)]

/// Raven `cbufExec_t` — command-buffer stuffing parameters.
///
/// Type definition source: `oracle/code/game/q_shared.h:221-226`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cbufExec_t {
    /// Raven: don't return until completed, a VM should NEVER use this, because
    /// some commands might cause the VM to be unloaded...
    EXEC_NOW,
    /// Raven: insert at current position, but don't run yet
    EXEC_INSERT,
    /// Raven: add to end of the command buffer (normal case)
    EXEC_APPEND,
}
