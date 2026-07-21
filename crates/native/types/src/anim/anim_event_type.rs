//! `bg_public.h` animation event type enumeration.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:304-316`
//! Type definition source: `oracle/code/game/bg_public.h:520-532`

#![allow(non_camel_case_types)]

/// Raven `animEventType_t`.
///
/// Raven: Be sure to update animEventTypeTable and ParseAnimationEvtBlock(...) if you change this enum list!
/// Type definition source: `oracle/codemp/game/bg_public.h:304-316`
/// Type definition source: `oracle/code/game/bg_public.h:520-532`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum animEventType_t {
    AEV_NONE = 0,
    AEV_SOUND = 1, //# animID AEV_SOUND framenum soundpath randomlow randomhi chancetoplay
    AEV_FOOTSTEP = 2, //# animID AEV_FOOTSTEP framenum footstepType chancetoplay
    AEV_EFFECT = 3, //# animID AEV_EFFECT framenum effectpath boltName chancetoplay
    AEV_FIRE = 4,  //# animID AEV_FIRE framenum altfire chancetofire
    AEV_MOVE = 5,  //# animID AEV_MOVE framenum forwardpush rightpush uppush
    AEV_SOUNDCHAN = 6, //# animID AEV_SOUNDCHAN framenum CHANNEL soundpath randomlow randomhi chancetoplay
    AEV_SABER_SWING = 7, //# animID AEV_SABER_SWING framenum CHANNEL randomlow randomhi chancetoplay
    AEV_SABER_SPIN = 8, //# animID AEV_SABER_SPIN framenum CHANNEL chancetoplay
    AEV_NUM_AEV = 9,
}
