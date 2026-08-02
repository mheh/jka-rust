//! `SoundSystem` — the owned home of every `snd_dma`/`snd_mem`/`snd_mix` global.
//!
//! Raven keeps the mixer in C file-scope globals. This struct is the one owned
//! instance `Engine.snd` holds (porting-rules §B3, §B6), and every `S_*`
//! function takes it as a receiver instead of reaching for ambient state.
//!
//! Source: `oracle/codemp/client/snd_dma.cpp:127-194`,
//! `oracle/codemp/client/snd_mix.cpp:10-12`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::cvar::CvarHandle;
use mp_qshared::shared::limits::MAX_GENTITIES;
use mp_qshared::shared::vec3_t;

use crate::snd::channel_t::channel_t;
use crate::snd::dma_t::dma_t;
use crate::snd::loop_sound_t::{loopSound_t, MAX_LOOP_SOUNDS};
use crate::snd::music_info_t::MusicInfo_t;
use crate::snd::music_state_e::MusicState_e;
use crate::snd::portable_samplepair_t::portable_samplepair_t;
use crate::snd::sfx_s::sfx_t;
use crate::snd_device::SoundDevice;

/// Raven `MAX_CHANNELS` — mixer channels.
///
/// Source: `oracle/codemp/client/snd_local.h:171`
pub const MAX_CHANNELS: usize = 32;

/// Raven `PAINTBUFFER_SIZE` — sample pairs the paint chain mixes per pass.
///
/// Source: `oracle/codemp/client/snd_local.h:26`
pub const PAINTBUFFER_SIZE: usize = 1024;

/// Raven `MAX_RAW_SAMPLES` — the streamed-audio ring length.
///
/// Source: `oracle/codemp/client/snd_local.h:182`
pub const MAX_RAW_SAMPLES: usize = 16384;

/// Raven `MAX_SFX` — sound-effect slots.
///
/// Raven: MAX_SFX may be larger than MAX_SOUNDS because of custom player sounds.
/// Source: `oracle/codemp/client/snd_dma.cpp:143`
pub const MAX_SFX: usize = 10000;

/// Raven `LOOP_HASH` — buckets in the sound-name hash table.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:147`
pub const LOOP_HASH: usize = 128;

/// Raven `eBGRNDTRACK_NUMBEROF` — background-music track slots.
///
/// Source: `oracle/codemp/client/snd_music.h:33`
pub const BGRNDTRACK_NUMBEROF: usize = 15;

/// The whole software mixer: channels, the sfx cache, the listener, the raw
/// stream, and the ring the paint chain writes.
/// `Engine.snd` owns one of these on a client build, and `None` on dedicated.
/// The OpenAL and EAX arm is dropped (DEC-57.4), so no field carries it.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:127-194`
pub struct SoundSystem {
    // ---- device and mix state ----
    /// Raven `dma` — the output format and the ring.
    /// Source: `oracle/codemp/client/snd_dma.cpp:132`
    pub dma: dma_t,
    /// The device read cursor, in the units Raven's `SNDDMA_GetDMAPos` returns
    /// (`dma.channels` per stereo frame). DEC-57.1 dissolves the five `SNDDMA_*`
    /// functions into the device end, so the device end writes this field.
    pub dma_pos: c_int,
    /// The open cpal output stream, `None` until `SNDDMA_Init` opens one.
    /// Raven's link-time device arm, made explicit.
    pub device: Option<SoundDevice>,
    /// Whether `SNDDMA_Init` may open a device at all. The platform shell sets
    /// this at client boot (DEC-56); `jampded` and every headless rig leave it
    /// false and keep the silent ring.
    pub device_enabled: bool,
    /// Raven `s_soundStarted` — non-zero between `S_Init` and `S_Shutdown`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:129`
    pub s_soundStarted: c_int,
    /// Raven `s_soundMuted` — true until `S_BeginRegistration` lets sound play again.
    /// Source: `oracle/codemp/client/snd_dma.cpp:130`
    pub s_soundMuted: bool,
    /// Raven `s_shutUp` — silences the missing-sound warning.
    /// Source: `oracle/codemp/client/snd_dma.cpp:16`
    pub s_shutUp: bool,
    /// Raven `s_soundtime` / `s_paintedtime` — both counted in sample pairs.
    /// Source: `oracle/codemp/client/snd_dma.cpp:138-139`
    pub s_soundtime: c_int,
    pub s_paintedtime: c_int,

    // ---- listener ----
    /// Raven `listener_number`, `listener_origin`, `listener_axis`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:134-136`
    pub listener_number: c_int,
    pub listener_origin: vec3_t,
    pub listener_axis: [vec3_t; 3],

    // ---- channels and sounds ----
    /// Raven `s_channels`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:127`
    pub s_channels: Box<[channel_t; MAX_CHANNELS]>,
    /// Raven `s_knownSfx` plus `s_numSfx`: the vector length is the count, and a
    /// slot index is Raven's `sfxHandle_t`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:144-145`
    pub s_knownSfx: Vec<sfx_t>,
    /// Raven `sfxHash` — the name-lookup chain heads, as `s_knownSfx` slots.
    /// Source: `oracle/codemp/client/snd_dma.cpp:148`
    pub sfxHash: Box<[Option<usize>; LOOP_HASH]>,

    // ---- looping sounds ----
    /// Raven `numLoopSounds` / `loopSounds`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:186-187`
    pub numLoopSounds: c_int,
    pub loopSounds: Box<[loopSound_t; MAX_LOOP_SOUNDS]>,
    /// Raven's `S_AddLoopSounds` function static `loopFrame`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:2006`
    pub loopFrame: c_int,

    // ---- raw stream ----
    /// Raven `s_rawend` / `s_rawsamples` — the streamed-audio write cursor and ring.
    /// Source: `oracle/codemp/client/snd_dma.cpp:190-191`
    pub s_rawend: c_int,
    pub s_rawsamples: Box<[portable_samplepair_t; MAX_RAW_SAMPLES]>,

    // ---- per-entity tables ----
    /// Raven `s_entityPosition`, `s_entityWavVol`, `s_entityWavVol_back`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:192-194`
    pub s_entityPosition: Box<[vec3_t; MAX_GENTITIES]>,
    pub s_entityWavVol: Box<[c_int; MAX_GENTITIES]>,
    pub s_entityWavVol_back: Box<[c_int; MAX_GENTITIES]>,
    /// Raven's `S_CheckAmplitude` file static `next_amplitude`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:2337`
    pub next_amplitude: c_int,

    // ---- paint chain ----
    /// Raven `paintbuffer` and `snd_vol` (`snd_mix.cpp`).
    /// Source: `oracle/codemp/client/snd_mix.cpp:10-11`
    pub paintbuffer: Box<[portable_samplepair_t; PAINTBUFFER_SIZE]>,
    pub snd_vol: c_int,

    // ---- `S_GetSoundtime` statics ----
    /// Raven's `S_GetSoundtime` function statics `buffers` and `oldsamplepos`.
    /// Source: `oracle/codemp/client/snd_dma.cpp:2746-2747`
    pub buffers: c_int,
    pub oldsamplepos: c_int,

    // ---- sound-memory accounting ----
    /// Raven's `Z_MemSize(TAG_SND_RAWDATA)`: bytes of sample data alive right
    /// now. The pool moved to owned `Vec`s, so the tag total is counted here.
    /// Source: `oracle/codemp/client/snd_dma.cpp:5025`
    pub sndRawDataBytes: c_int,
    /// Raven `gbInsideLoadSound` — set while `S_LoadSound_Actual` runs.
    /// Source: `oracle/codemp/client/snd_mem.cpp:720`
    pub gbInsideLoadSound: bool,

    // ---- background music (gh#25 fills the loading half) ----
    /// Raven `tMusic_Info` and the four music state globals beside it.
    /// Source: `oracle/codemp/client/snd_dma.cpp:104-109`
    pub tMusic_Info: Vec<MusicInfo_t>,
    pub bMusic_IsDynamic: bool,
    pub eMusic_StateActual: MusicState_e,
    pub eMusic_StateRequest: MusicState_e,
    /// only valid for non-dynamic music
    pub sMusic_BackgroundLoop: String,
    pub sInfoOnly_CurrentDynamicMusicSet: String,

    // ---- cvar handles ----
    /// Raven's cached `cvar_t*` sound cvars (§B5 index-not-pointer).
    /// Source: `oracle/codemp/client/snd_dma.cpp:151-168,5012`
    pub s_volume: Option<CvarHandle>,
    pub s_volumeVoice: Option<CvarHandle>,
    pub s_testsound: Option<CvarHandle>,
    pub s_khz: Option<CvarHandle>,
    pub s_allowDynamicMusic: Option<CvarHandle>,
    pub s_show: Option<CvarHandle>,
    pub s_mixahead: Option<CvarHandle>,
    pub s_mixPreStep: Option<CvarHandle>,
    pub s_musicVolume: Option<CvarHandle>,
    pub s_separation: Option<CvarHandle>,
    pub s_lip_threshold_1: Option<CvarHandle>,
    pub s_lip_threshold_2: Option<CvarHandle>,
    pub s_lip_threshold_3: Option<CvarHandle>,
    pub s_lip_threshold_4: Option<CvarHandle>,
    pub s_language: Option<CvarHandle>,
    pub s_debugdynamic: Option<CvarHandle>,
    pub s_soundpoolmegs: Option<CvarHandle>,
}

impl Default for SoundSystem {
    /// The C loader's zero fill of every sound global, before `S_Init` runs.
    fn default() -> SoundSystem {
        SoundSystem {
            dma: dma_t::default(),
            dma_pos: 0,
            device: None,
            device_enabled: false,
            s_soundStarted: 0,
            s_soundMuted: false,
            s_shutUp: false,
            s_soundtime: 0,
            s_paintedtime: 0,
            listener_number: 0,
            listener_origin: [0.0; 3],
            listener_axis: [[0.0; 3]; 3],
            s_channels: Box::new([channel_t::default(); MAX_CHANNELS]),
            s_knownSfx: Vec::new(),
            sfxHash: Box::new([None; LOOP_HASH]),
            numLoopSounds: 0,
            loopSounds: Box::new([loopSound_t::default(); MAX_LOOP_SOUNDS]),
            loopFrame: 0,
            s_rawend: 0,
            s_rawsamples: vec![portable_samplepair_t::default(); MAX_RAW_SAMPLES]
                .into_boxed_slice()
                .try_into()
                .ok()
                .expect("raw-sample ring length"),
            s_entityPosition: vec![[0.0f32; 3]; MAX_GENTITIES]
                .into_boxed_slice()
                .try_into()
                .ok()
                .expect("entity-position table length"),
            s_entityWavVol: vec![0; MAX_GENTITIES]
                .into_boxed_slice()
                .try_into()
                .ok()
                .expect("lipsync table length"),
            s_entityWavVol_back: vec![0; MAX_GENTITIES]
                .into_boxed_slice()
                .try_into()
                .ok()
                .expect("lipsync backup table length"),
            next_amplitude: 0,
            paintbuffer: vec![portable_samplepair_t::default(); PAINTBUFFER_SIZE]
                .into_boxed_slice()
                .try_into()
                .ok()
                .expect("paint buffer length"),
            snd_vol: 0,
            buffers: 0,
            oldsamplepos: 0,
            sndRawDataBytes: 0,
            gbInsideLoadSound: false,
            tMusic_Info: (0..BGRNDTRACK_NUMBEROF).map(|_| MusicInfo_t::default()).collect(),
            bMusic_IsDynamic: false,
            eMusic_StateActual: MusicState_e::eBGRNDTRACK_EXPLORE,
            eMusic_StateRequest: MusicState_e::eBGRNDTRACK_EXPLORE,
            sMusic_BackgroundLoop: String::new(),
            sInfoOnly_CurrentDynamicMusicSet: String::new(),
            s_volume: None,
            s_volumeVoice: None,
            s_testsound: None,
            s_khz: None,
            s_allowDynamicMusic: None,
            s_show: None,
            s_mixahead: None,
            s_mixPreStep: None,
            s_musicVolume: None,
            s_separation: None,
            s_lip_threshold_1: None,
            s_lip_threshold_2: None,
            s_lip_threshold_3: None,
            s_lip_threshold_4: None,
            s_language: None,
            s_debugdynamic: None,
            s_soundpoolmegs: None,
        }
    }
}

impl SoundSystem {
    /// Raven's `s_numSfx` — the number of `sfx_t` slots in use.
    /// Source: `oracle/codemp/client/snd_dma.cpp:145`
    pub fn s_numSfx(&self) -> c_int {
        self.s_knownSfx.len() as c_int
    }
}
