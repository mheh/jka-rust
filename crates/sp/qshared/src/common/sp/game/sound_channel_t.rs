//! SP `soundChannel_t` from `code/game/channels.h` (included by SP `q_shared.h`).

#![allow(non_camel_case_types)]

/// Raven `soundChannel_t` — which logical channel a sound plays on.
///
/// Raven: these entries are now also duplicated in ModView; note that the
/// order is ok to change, I only read/write text strings of them anyway. - Ste.
/// Type definition source: `oracle/code/game/channels.h:8-21`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum soundChannel_t {
    /// Auto-picks an empty channel to play sound on
    CHAN_AUTO = 0,
    /// menu sounds, etc
    CHAN_LOCAL,
    CHAN_WEAPON,
    /// Voice sounds cause mouth animation
    CHAN_VOICE,
    /// Causes mouth animation but still use normal sound falloff
    CHAN_VOICE_ATTEN,
    /// Causes mouth animation and is broadcast with no separation
    CHAN_VOICE_GLOBAL,
    CHAN_ITEM,
    CHAN_BODY,
    /// added for ambient sounds
    CHAN_AMBIENT,
    /// chat messages, etc
    CHAN_LOCAL_SOUND,
    /// announcer voices, etc
    CHAN_ANNOUNCER,
    /// attenuates similar to chan_voice, but uses empty channel auto-pick behaviour
    CHAN_LESS_ATTEN,
    /// played as a looping sound - added by BTO (VV)
    CHAN_MUSIC,
}

const _: () = assert!(core::mem::size_of::<soundChannel_t>() == 4);
