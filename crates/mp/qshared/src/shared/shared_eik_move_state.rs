#![allow(non_camel_case_types)]

/// Raven `sharedEIKMoveState` IK bone move states.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:960-964`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum sharedEIKMoveState {
    IKS_NONE = 0,
    IKS_DYNAMIC,
}
