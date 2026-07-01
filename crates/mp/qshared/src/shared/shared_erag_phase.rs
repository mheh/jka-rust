#![allow(non_camel_case_types)]

/// Raven `sharedERagPhase` ragdoll update callback phases.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:856-865`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum sharedERagPhase {
    RP_START_DEATH_ANIM,
    RP_END_DEATH_ANIM,
    RP_DEATH_COLLISION,
    RP_CORPSE_SHOT,
    /// Raven: this actually does nothing but set the pelvisAnglesOffset, and pelvisPositionOffset
    RP_GET_PELVIS_OFFSET,
    /// Raven: this actually does nothing but set the pelvisAnglesOffset, and pelvisPositionOffset
    RP_SET_PELVIS_OFFSET,
    /// Raven: this removes effectors given by the effectorsToTurnOff member
    RP_DISABLE_EFFECTORS,
}
