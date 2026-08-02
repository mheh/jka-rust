//! Raven `sfx_s` — one loaded or loadable sound effect.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::mp3::mp3_stream_state::MP3StreamState;
use crate::snd::sfx_sample_data::SfxSampleData;
use crate::snd::sound_compression_method_t::SoundCompressionMethod_t;

/// Raven `sfx_s` — a sound effect slot in `s_knownSfx` (typedef `sfx_t`).
///
/// The sound stack is engine-internal, so the sample block becomes an owned
/// `SfxSampleData` and the `sfx_s *next` hash chain becomes a slot index.
/// The OpenAL `Buffer` and `lipSyncData` fields are dropped with the OpenAL arm
/// (DEC-57.4).
/// Type definition source: `oracle/codemp/client/snd_local.h:48-65`
#[derive(Clone)]
pub struct sfx_t {
    /// The sample block. `None` is Raven's NULL `pSoundData`, which it tells
    /// apart from a zero-length block that `Z_Malloc(0)` still returns.
    pub pSoundData: Option<SfxSampleData>,
    /// couldn't be loaded, so use buzz
    pub bDefaultSound: bool,
    /// not in Memory, set qtrue when loaded, and qfalse when its buffers are freed up because of
    /// being old, so can be reloaded
    pub bInMemory: bool,
    pub eSoundCompressionMethod: SoundCompressionMethod_t,
    /// Raven `pMP3StreamHeader` — the pristine stream state a playing channel
    /// copies. `None` unless this sfx is an MP3.
    pub pMP3StreamHeader: Option<Box<MP3StreamState>>,
    /// length in samples, always kept as 16bit now so this is #shorts
    pub iSoundLengthInSamples: c_int,
    pub sSoundName: String,
    pub iLastTimeUsed: c_int,
    /// used to set the highest volume this sample has at load time - used for lipsynching
    pub fVolRange: f32,
    /// used for cacheing purposes
    pub iLastLevelUsedOn: c_int,
    /// only used because of hash table when registering
    pub next: Option<usize>,
}

impl Default for sfx_t {
    /// Raven zero-fills a fresh `sfx_t` in `S_FindName`, so every field starts empty.
    /// Source: `oracle/codemp/client/snd_dma.cpp:851`
    fn default() -> sfx_t {
        sfx_t {
            pSoundData: None,
            bDefaultSound: false,
            bInMemory: false,
            eSoundCompressionMethod: SoundCompressionMethod_t::ct_16,
            pMP3StreamHeader: None,
            iSoundLengthInSamples: 0,
            sSoundName: String::new(),
            iLastTimeUsed: 0,
            fVolRange: 0.0,
            iLastLevelUsedOn: 0,
            next: None,
        }
    }
}

/// Raven typedef `sfx_s` (the tagged struct name) for `sfx_t`.
pub type sfx_s = sfx_t;
