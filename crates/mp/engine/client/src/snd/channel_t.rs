//! Raven `channel_t` — one mixer channel.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

/// Raven `START_SAMPLE_IMMEDIATE` — the channel mixes from the next paint.
///
/// Source: `oracle/codemp/client/snd_local.h:77`
pub const START_SAMPLE_IMMEDIATE: u32 = 0x7fff_ffff;

/// Raven `channel_t` — a live playback channel in the software mixer.
///
/// The sound stack is engine-internal, so `sfx_t *thesfx` becomes an index into
/// `SoundSystem::s_knownSfx` and `None` reads as Raven's NULL.
/// The OpenAL block and the MP3 sliding-decode block are dropped here: DEC-57.4
/// drops the OpenAL arm, and gh#25 owns the MP3 decoder.
/// Type definition source: `oracle/codemp/client/snd_local.h:94-129`
#[derive(Default, Clone, Copy)]
pub struct channel_t {
    /// START_SAMPLE_IMMEDIATE = set immediately on next mix
    pub startSample: u32,
    /// to allow overriding a specific sound
    pub entnum: c_int,
    /// to allow overriding a specific sound
    pub entchannel: c_int,
    /// 0-255 volume after spatialization
    pub leftvol: c_int,
    /// 0-255 volume after spatialization
    pub rightvol: c_int,
    /// 0-255 volume before spatialization
    pub master_vol: c_int,
    /// only use if fixed_origin is set
    pub origin: vec3_t,
    /// use origin instead of fetching entnum's origin
    pub fixed_origin: bool,
    /// The `s_knownSfx` slot this channel plays. `None` is Raven's NULL `thesfx`.
    pub thesfx: Option<usize>,
    /// from an S_AddLoopSound call, cleared each frame
    pub loopSound: bool,
}
