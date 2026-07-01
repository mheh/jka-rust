#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `soundChannel_t` sound channel selector.
///
/// Raven declares this as `typedef int` alongside a separate anonymous enum of
/// channel ids, so the alias stays an int and the enumerators are `const`s.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:1945-1961`
pub type soundChannel_t = c_int;

/// Raven: Auto-picks an empty channel to play sound on
pub const CHAN_AUTO: soundChannel_t = 0;
/// Raven: menu sounds, etc
pub const CHAN_LOCAL: soundChannel_t = 1;
pub const CHAN_WEAPON: soundChannel_t = 2;
/// Raven: Voice sounds cause mouth animation
pub const CHAN_VOICE: soundChannel_t = 3;
/// Raven: Causes mouth animation but still use normal sound falloff
pub const CHAN_VOICE_ATTEN: soundChannel_t = 4;
pub const CHAN_ITEM: soundChannel_t = 5;
pub const CHAN_BODY: soundChannel_t = 6;
/// Raven: added for ambient sounds
pub const CHAN_AMBIENT: soundChannel_t = 7;
/// Raven: chat messages, etc
pub const CHAN_LOCAL_SOUND: soundChannel_t = 8;
/// Raven: announcer voices, etc
pub const CHAN_ANNOUNCER: soundChannel_t = 9;
/// Raven: attenuates similar to chan_voice, but uses empty channel auto-pick behaviour
pub const CHAN_LESS_ATTEN: soundChannel_t = 10;
/// Raven: menu stuff, etc
pub const CHAN_MENU1: soundChannel_t = 11;
/// Raven: Causes mouth animation and is broadcast, like announcer
pub const CHAN_VOICE_GLOBAL: soundChannel_t = 12;
/// Raven: music played as a looping sound - added by BTO (VV)
pub const CHAN_MUSIC: soundChannel_t = 13;
