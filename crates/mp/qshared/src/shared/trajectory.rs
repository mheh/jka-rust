use core::ffi::c_int;

use crate::shared::vec3_t;

/// Raven `trType_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2426-2435`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2648-2657`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum trType_t {
    TR_STATIONARY = 0,
    /// Raven comment: non-parametric, but interpolate between snapshots
    TR_INTERPOLATE = 1,
    TR_LINEAR = 2,
    TR_LINEAR_STOP = 3,
    TR_NONLINEAR_STOP = 4,
    /// Raven comment: value = base + sin( time / duration ) * delta
    TR_SINE = 5,
    TR_GRAVITY = 6,
}

/// Raven `trajectory_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2437-2443`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2659-2665`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct trajectory_t {
    pub trType: trType_t,
    pub trTime: c_int,
    /// if non 0, trTime + trDuration = stop time
    pub trDuration: c_int,
    pub trBase: vec3_t,
    /// velocity, etc
    pub trDelta: vec3_t,
}

const _: () = assert!(core::mem::size_of::<trajectory_t>() == 36);
const _: () = assert!(core::mem::offset_of!(trajectory_t, trType) == 0);
const _: () = assert!(core::mem::offset_of!(trajectory_t, trTime) == 4);
const _: () = assert!(core::mem::offset_of!(trajectory_t, trDuration) == 8);
const _: () = assert!(core::mem::offset_of!(trajectory_t, trBase) == 12);
const _: () = assert!(core::mem::offset_of!(trajectory_t, trDelta) == 24);
