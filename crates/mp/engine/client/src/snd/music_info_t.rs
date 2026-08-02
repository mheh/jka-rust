//! Raven `MusicInfo_t` — one background-music track slot.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::fileHandle_t;

use crate::mp3::mp3_stream_state::MP3StreamState;
use crate::snd::channel_mp3_state::ChannelMp3State;
use crate::snd::music_state_e::MusicState_e;
use crate::snd::sfx_s::sfx_t;
use crate::snd::wavinfo_t::wavinfo_t;

/// Raven `iMP3MusicStream_DiskBytesToRead` / `iMP3MusicStream_DiskBufferSize` —
/// the disk-streamer read size and its window.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:35-36`
pub const iMP3MusicStream_DiskBytesToRead: c_int = 10000;
pub const iMP3MusicStream_DiskBufferSize: c_int = iMP3MusicStream_DiskBytesToRead * 2;

/// Raven `MusicInfo_t` — the per-track state of the background-music player.
///
/// The dynamic player copies whole tracks between slots (the cross-fade slot is
/// a struct copy), so the type is `Clone`. A copied stream restarts its decode,
/// which is what Raven's `Rewind` does at every place a copy is followed by
/// playback.
/// Type definition source: `oracle/codemp/client/snd_dma.cpp:38-98`
#[derive(Clone)]
pub struct MusicInfo_t {
    pub bIsMP3: bool,
    /// Raven `sfxMP3_Bgrnd` — the fake `sfx_t` the background channel plays.
    pub sfxMP3_Bgrnd: sfx_t,
    /// Raven `streamMP3_Bgrnd` — the pristine stream the channel copies from.
    pub streamMP3_Bgrnd: MP3StreamState,
    /// Raven `chMP3_Bgrnd` — the working channel, sliding window included.
    pub chMP3_Bgrnd: ChannelMp3State,
    /// Raven `byMP3MusicStream_DiskBuffer` plus its two cursors — the window a
    /// non-dynamic MP3 track streams through.
    pub byMP3MusicStream_DiskBuffer: Vec<u8>,
    pub iMP3MusicStream_DiskReadPos: c_int,
    pub iMP3MusicStream_DiskWindowPos: c_int,
    /// The loaded file image, kept valid with `sLoadedDataName` and `iLoadedDataLen`.
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
}

impl Default for MusicInfo_t {
    /// Raven's `tMusic_Info[...] = {0}` file-scope zero fill.
    /// Source: `oracle/codemp/client/snd_dma.cpp:104`
    fn default() -> MusicInfo_t {
        MusicInfo_t {
            bIsMP3: false,
            sfxMP3_Bgrnd: sfx_t::default(),
            streamMP3_Bgrnd: MP3StreamState::default(),
            chMP3_Bgrnd: ChannelMp3State::default(),
            byMP3MusicStream_DiskBuffer: vec![0u8; iMP3MusicStream_DiskBufferSize as usize],
            iMP3MusicStream_DiskReadPos: 0,
            iMP3MusicStream_DiskWindowPos: 0,
            pLoadedData: Vec::new(),
            sLoadedDataName: String::new(),
            iLoadedDataLen: 0,
            iXFadeVolumeSeekTime: 0,
            iXFadeVolumeSeekTo: 0,
            iXFadeVolume: 0,
            fSmoothedOutVolume: 0.0,
            bActive: false,
            bExists: false,
            bTrackSwitchPending: false,
            eTS_NewState: MusicState_e::eBGRNDTRACK_EXPLORE,
            fTS_NewTime: 0.0,
            s_backgroundFile: 0,
            s_backgroundInfo: wavinfo_t::default(),
            s_backgroundSamples: 0,
        }
    }
}

impl MusicInfo_t {
    /// Raven `MusicInfo_t::Rewind` — restart the track and reset the sample count.
    ///
    /// Source: `oracle/codemp/client/snd_dma.cpp:84-88`
    pub fn Rewind(&mut self) {
        let pristine = self.streamMP3_Bgrnd.clone();
        self.chMP3_Bgrnd.iMP3SlidingDecodeWritePos = 0;
        self.chMP3_Bgrnd.iMP3SlidingDecodeWindowPos = 0;
        self.chMP3_Bgrnd.MP3StreamHeader = pristine;
        self.s_backgroundSamples = self.sfxMP3_Bgrnd.iSoundLengthInSamples;
    }
}
