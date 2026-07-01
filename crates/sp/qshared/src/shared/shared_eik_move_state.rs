#![allow(non_camel_case_types)]

/// Raven `sharedEIKMoveState` — IK bone move states.
///
/// Raven declares this as a bare (non-typedef) C++ `enum`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2604-2608`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum sharedEIKMoveState {
    IKS_NONE = 0,
    IKS_DYNAMIC,
}
