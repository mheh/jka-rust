//! MP `bg_public.h` animation event type definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:304-316`

#![allow(non_camel_case_types)]

/// Raven `animEventType_t`.
///
/// Raven: Be sure to update animEventTypeTable and ParseAnimationEvtBlock(...) if you change this enum list!
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:304-316`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum animEventType_t {
    AEV_NONE = 0,
    AEV_SOUND = 1,
    AEV_FOOTSTEP = 2,
    AEV_EFFECT = 3,
    AEV_FIRE = 4,
    AEV_MOVE = 5,
    AEV_SOUNDCHAN = 6,
    AEV_SABER_SWING = 7,
    AEV_SABER_SPIN = 8,
    AEV_NUM_AEV = 9,
}
