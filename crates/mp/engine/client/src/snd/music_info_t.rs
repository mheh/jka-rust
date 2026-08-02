//! Raven `MusicInfo_t` — one background-music track slot.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::fileHandle_t;

use crate::snd::music_state_e::MusicState_e;
use crate::snd::wavinfo_t::wavinfo_t;

/// Raven `MusicInfo_t` — the per-track state of the background-music player.
///
/// gh#24 ports the half the mixer paths touch: the open-file flag, the fade
/// counters, and the loaded-file block that `S_UnCacheDynamicMusic` frees.
/// The MP3 stream block and the disk-stream window arrive with gh#25 (DEC-57.3).
/// Type definition source: `oracle/codemp/client/snd_dma.cpp:38-98`
#[derive(Default)]
pub struct MusicInfo_t {
    pub bIsMP3: bool,
    /// The Z_Malloc'd file image, kept valid with `sLoadedDataName` and `iLoadedDataLen`.
    pub pLoadedData: Vec<u8>,
    pub sLoadedDataName: String,
    pub iLoadedDataLen: c_int,
    pub iXFadeVolumeSeekTime: c_int,
    /// Set this to 0 or 255 only, and stamp `iXFadeVolumeSeekTime` at the same time.
    pub iXFadeVolumeSeekTo: c_int,
    /// 0 = silent, 255 = max mixer vol, though still modulated via overall music_volume
    pub iXFadeVolume: c_int,
    pub fSmoothedOutVolume: f32,
    /// whether playing or not
    pub bActive: bool,
    /// whether was even loaded for this level (ie don't try and start playing it)
    pub bExists: bool,
    pub bTrackSwitchPending: bool,
    pub eTS_NewState: MusicState_e,
    pub fTS_NewTime: f32,
    /// valid handle, else -1 if an MP3 (so that NZ compares still work)
    pub s_backgroundFile: fileHandle_t,
    pub s_backgroundInfo: wavinfo_t,
    pub s_backgroundSamples: c_int,
    //TODO: Port MusicInfo_t MP3 block
    // Source: oracle/codemp/client/snd_dma.cpp:44-52. `sfxMP3_Bgrnd`,
    // `streamMP3_Bgrnd`, `chMP3_Bgrnd`, and the disk-stream window land with gh#25.
}
