#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MusicState_e` — dynamic music state/track selection.
///
/// Raven: None.
/// Type definition source: `oracle/codemp/client/snd_music.h:11-36`
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MusicState_e {
    /// For normal walking around
    #[default]
    eBGRNDTRACK_EXPLORE = 0,
    /// For excitement
    eBGRNDTRACK_ACTION = 1,
    /// (optional) for final encounter
    eBGRNDTRACK_BOSS = 2,
    /// (optional) death "flourish"
    eBGRNDTRACK_DEATH = 3,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS0 = 4,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS1 = 5,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS2 = 6,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS3 = 7,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS0 = 8,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS1 = 9,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS2 = 10,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS3 = 11,
    /// Used for when music is just streaming, not part of dynamic stuff
    eBGRNDTRACK_NONDYNAMIC = 12,
    /// Silence (more of a logic thing than an actual track at the moment)
    eBGRNDTRACK_SILENCE = 13,
    /// The xfade channel
    eBGRNDTRACK_FADE = 14,
    /// Used only for array sizing
    eBGRNDTRACK_NUMBEROF = 15,
}
