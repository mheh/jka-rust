#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MusicState_e` — background music state enumeration.
///
/// Type definition source: `oracle/code/client/snd_music.h:11-36`
#[repr(i32)]
pub enum MusicState_e {
    /// For normal walking around
    eBGRNDTRACK_EXPLORE = 0,
    /// For excitement
    eBGRNDTRACK_ACTION,
    /// (optional) for final encounter
    eBGRNDTRACK_BOSS,
    /// (optional) death "flourish"
    eBGRNDTRACK_DEATH,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS0,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS1,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS2,
    /// Transition from action to explore
    eBGRNDTRACK_ACTIONTRANS3,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS0,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS1,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS2,
    /// Transition from explore to silence
    eBGRNDTRACK_EXPLORETRANS3,
    /// Used for when music is just streaming, not part of dynamic stuff
    /// (used to be defined as same as explore entry, but this allows playing
    /// music in between 2 invocations of the same dynamic music without
    /// mid-level reload, and also faster level transitioning if two consecutive
    /// dynamic sections use same DMS.DAT entries.)
    eBGRNDTRACK_NONDYNAMIC,
    /// Silence (more of a logic thing than an actual track at the moment)
    eBGRNDTRACK_SILENCE,
    /// The xfade channel
    eBGRNDTRACK_FADE,
    /// Number of music states (for array sizing)
    eBGRNDTRACK_NUMBEROF,
}
