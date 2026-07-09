#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use sp_qshared::shared::MAX_QPATH;

use crate::mp3::mp3_stream::MP3STREAM;
use crate::snd::sound_compression_method_t::SoundCompressionMethod_t;

/// Raven `sfx_s` — a loaded/loadable sound effect (typedef'd as `sfx_t`).
///
/// Type definition source: `oracle/code/client/snd_local.h:48-65`
#[repr(C)]
pub struct sfx_t {
    pub pSoundData: *mut i16,
    /// Couldn't be loaded, so use buzz
    pub bDefaultSound: bool,
    /// Not in Memory, set qtrue when loaded, and qfalse when its buffers are freed up because
    /// of being old, so can be reloaded
    pub bInMemory: bool,
    /// Used for cacheing purposes
    pub iLastLevelUsedOn: i16,
    pub eSoundCompressionMethod: SoundCompressionMethod_t,
    /// NULL ptr unless this sfx_t is an MP3. Use Z_Malloc and Z_Free
    pub pMP3StreamHeader: *mut MP3STREAM,
    /// Length in samples, always kept as 16bit now so this is #shorts (watch for stereo later
    /// for music?)
    pub iSoundLengthInSamples: i32,
    pub sSoundName: [c_char; MAX_QPATH],
    pub iLastTimeUsed: i32,
    /// Used to set the highest volume this sample has at load time - used for lipsynching
    pub fVolRange: f32,

    // Open AL
    pub Buffer: u32,
    pub lipSyncData: *mut c_char,

    /// Only used because of hash table when registering
    pub next: *mut sfx_t,
}

/// Raven typedef `sfx_s` (the tagged struct name) for `sfx_t`.
pub type sfx_s = sfx_t;

const _: () = assert!(core::mem::size_of::<sfx_t>() == 120);
const _: () = assert!(core::mem::offset_of!(sfx_t, pSoundData) == 0);
const _: () = assert!(core::mem::offset_of!(sfx_t, bDefaultSound) == 8);
const _: () = assert!(core::mem::offset_of!(sfx_t, bInMemory) == 9);
const _: () = assert!(core::mem::offset_of!(sfx_t, iLastLevelUsedOn) == 10);
const _: () = assert!(core::mem::offset_of!(sfx_t, eSoundCompressionMethod) == 12);
const _: () = assert!(core::mem::offset_of!(sfx_t, pMP3StreamHeader) == 16);
const _: () = assert!(core::mem::offset_of!(sfx_t, iSoundLengthInSamples) == 24);
const _: () = assert!(core::mem::offset_of!(sfx_t, sSoundName) == 28);
const _: () = assert!(core::mem::offset_of!(sfx_t, iLastTimeUsed) == 92);
const _: () = assert!(core::mem::offset_of!(sfx_t, fVolRange) == 96);
const _: () = assert!(core::mem::offset_of!(sfx_t, Buffer) == 100);
const _: () = assert!(core::mem::offset_of!(sfx_t, lipSyncData) == 104);
const _: () = assert!(core::mem::offset_of!(sfx_t, next) == 112);
