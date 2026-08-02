//! `snd_dma.cpp` — channels, spatialization, the sfx cache, and the frame driver.
//!
//! DEC-57.4 drops the OpenAL and EAX arm, so every `s_UseOpenAL` branch is gone
//! and only the software mixer is ported. DEC-57.1 dissolves the five `SNDDMA_*`
//! functions into the device end: `SoundSystem` owns the ring and the read
//! cursor, and the device end writes the cursor.
//!
//! Source: `oracle/codemp/client/snd_dma.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::cmd_common::{Cmd_Argc, Cmd_Argv};
use mp_engine_qcommon::cmd_pc::{Cmd_AddCommand, Cmd_RemoveCommand};
use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::common_fns::{Com_DPrintf, Com_Milliseconds};
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_Set};
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Read};
use mp_engine_qcommon::sys_engine::Sys_StreamedRead;
use mp_qshared::shared::cvar::{
    CVAR_ARCHIVE, CVAR_CHEAT, CVAR_LATCH, CVAR_NORESTART, CVAR_ROM,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::fileHandle_t;
use mp_qshared::shared::limits::MAX_GENTITIES;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::sound_channel::{
    CHAN_AMBIENT, CHAN_AUTO, CHAN_LESS_ATTEN, CHAN_LOCAL_SOUND, CHAN_VOICE, CHAN_VOICE_ATTEN,
    CHAN_VOICE_GLOBAL,
};
use mp_qshared::shared::vec3_t;
use mp_qshared::shared::MAX_QPATH;
use mp_renderer::hook_install::rm_from_view;
use native_math::qmath::{_DotProduct, _VectorCopy, _VectorSubtract, VectorNormalize};
use native_platform::sys_main::{Sys_BeginStreamedFile, Sys_EndStreamedFile, Sys_LowPhysicalMemory};
use native_string::q_string::Q_stricmp;

use crate::client_host::snd_from_view;
use crate::mp3::mp3_stream::MP3STREAM;
use crate::snd::channel_t::{channel_t, START_SAMPLE_IMMEDIATE};
use crate::snd::loop_sound_t::MAX_LOOP_SOUNDS;
use crate::snd::music_info_t::{
    iMP3MusicStream_DiskBufferSize, iMP3MusicStream_DiskBytesToRead, MusicInfo_t,
};
use crate::snd::music_state_e::MusicState_e;
use crate::snd::sfx_sample_data::SfxSampleData;
use crate::snd::sound_compression_method_t::SoundCompressionMethod_t;
use crate::snd::sound_system::{
    SoundSystem, BGRNDTRACK_NUMBEROF, LOOP_HASH, MAX_CHANNELS, MAX_RAW_SAMPLES, MAX_SFX,
};
use crate::snd_ambient::{AS_Free, AS_Init};
use crate::mp3::mp3_stream_state::MP3StreamState;
use crate::snd::channel_mp3_state::ChannelMp3State;
use crate::snd::sfx_s::sfx_t;
use crate::snd_mem::{COM_DefaultExtension_str, S_LoadSound, S_MP3_CalcVols_f_body};
use crate::snd_mix::S_PaintChannels;
use crate::snd_mp3::{
    MP3Stream_GetPlayingTimeInSeconds, MP3Stream_GetRemainingTimeInSeconds, MP3Stream_GetSamples,
    MP3Stream_InitPlayingTimeFields, MP3Stream_SeekTo, MP3_IsValid,
};
use crate::snd_music::{
    Music_AllowedToTransition, Music_BaseStateToString, Music_DynamicDataAvailable,
    Music_GetFileNameForState, Music_GetLevelSetName, Music_GetRandomEntryTime,
    Music_StateCanBeInterrupted,
};

/// Raven `sfxHandle_t` — the `s_knownSfx` slot a registered sound answers with.
///
/// Source: `oracle/codemp/game/q_shared.h:1822`
pub type sfxHandle_t = c_int;

/// Raven `SOUND_FULLVOLUME` — only begin attenuating sound volumes when outside
/// this range.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:119`
const SOUND_FULLVOLUME: f32 = 256.0;

/// Raven `SOUND_ATTENUATE` / `VOICE_ATTENUATE` — the two distance falloff rates.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:121-122`
const SOUND_ATTENUATE: f32 = 0.0008;
const VOICE_ATTENUATE: f32 = 0.004;

/// Raven `SOUND_FMAXVOL` / `SOUND_MAXVOL` — the mono ceiling and the channel ceiling.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:124-125`
const SOUND_FMAXVOL: f32 = 0.75;
const SOUND_MAXVOL: c_int = 255;

/// The retail DirectSound secondary buffer is 65536 bytes at every `s_khz` rate.
///
/// Source: `oracle/codemp/win32/win_snd.cpp:12,246`
const DMA_BUFFER_BYTES: usize = 0x10000;

/// Raven `FUZZY_AMOUNT` — the slack the `s_mp3overhead` default adds on top of
/// one `MP3STREAM`.
///
/// Raven: so it has to be significantly over, not just break even.
/// Source: `oracle/codemp/client/snd_mp3.cpp:222`
const FUZZY_AMOUNT: usize = 5 * 1024;

/// Raven `WAV_FORMAT_PCM` / `WAV_FORMAT_MP3`.
///
/// `WAV_FORMAT_MP3` is never a real wav format; it only keeps an MP3 track from
/// matching one of the legitimate values.
/// Source: `oracle/codemp/client/snd_local.h:132-134`
const WAV_FORMAT_PCM: c_int = 1;
const WAV_FORMAT_MP3: c_int = 3;

/// Raven `fDYNAMIC_XFADE_SECONDS` — the dynamic-music cross-fade length.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:102`
const fDYNAMIC_XFADE_SECONDS: f32 = 1.0;

// ===========================================================================
// The device end (DEC-57.1)
// ===========================================================================

/// Raven `SNDDMA_Init` — pick the output format and allocate the ring.
///
/// The port keeps the retail secondary-buffer shape and owns the ring outright.
//TODO: Port SNDDMA_Init device open
// Source: oracle/codemp/win32/win_snd.cpp:183. DEC-57.1 puts the real device
// behind the cpal callback, which the platform shell (gh#22) seats.
/// Source: `oracle/codemp/win32/win_snd.cpp:183-250`
fn SNDDMA_Init(common: &Common, snd: &mut SoundSystem) -> bool {
    snd.dma.channels = 2;
    snd.dma.samplebits = 16;

    snd.dma.speed = match common.cvar(snd.s_khz).integer {
        44 => 44100,
        22 => 22050,
        _ => 11025,
    };

    snd.dma.samples = (DMA_BUFFER_BYTES / (snd.dma.samplebits as usize / 8)) as c_int;
    snd.dma.submission_chunk = 1;
    snd.dma.buffer = vec![0u8; DMA_BUFFER_BYTES];
    snd.dma_pos = 0;
    true
}

/// Raven `SNDDMA_Shutdown` — release the ring.
///
/// Source: `oracle/codemp/win32/win_snd.cpp:257-265`
fn SNDDMA_Shutdown(snd: &mut SoundSystem) {
    snd.dma.buffer = Vec::new();
}

/// Raven `SNDDMA_GetDMAPos` — the device read cursor, masked to the ring.
///
/// Source: `oracle/codemp/win32/win_snd.cpp:274-289`
fn SNDDMA_GetDMAPos(snd: &SoundSystem) -> c_int {
    snd.dma_pos & (snd.dma.samples - 1)
}

// Raven's `SNDDMA_BeginPainting` and `SNDDMA_Submit` lock and unlock the
// DirectSound secondary buffer. The port owns the ring outright, so both calls
// have no body and their call sites drop them (DEC-57.1).

// ===========================================================================
// Channels and the sfx cache
// ===========================================================================

/// Raven `Channel_Clear` — reset one channel.
///
/// Raven zeroes everything except the MP3 sliding-decode buffer in the middle of
/// the struct, so the window allocation survives the clear.
/// Source: `oracle/codemp/client/snd_dma.cpp:321-330`
fn Channel_Clear(snd: &mut SoundSystem, channel: usize) {
    snd.s_channels[channel] = channel_t::default();
    snd.s_channelsMp3[channel].clear();
}

/// Raven `S_HashSFXName` — the sound-name hash, extension excluded.
///
/// Raven holds each letter in a signed `char`, so a byte of 0x80 or above adds a
/// negative term. The cast below keeps that sign.
/// Source: `oracle/codemp/client/snd_dma.cpp:756-772`
fn S_HashSFXName(name: &str) -> usize {
    let mut hash: i64 = 0;
    for (i, byte) in name.bytes().enumerate() {
        let mut letter = byte.to_ascii_lowercase();
        if letter == b'.' {
            break; // don't include extension
        }
        if letter == b'\\' {
            letter = b'/'; // damn path names
        }
        hash += i64::from(letter as i8) * (i as i64 + 119);
    }
    (hash as usize) & (LOOP_HASH - 1)
}

/// Raven `S_FindName` — find the sound by name, or take a fresh slot for it.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:781-859`
pub fn S_FindName(snd: &mut SoundSystem, name: &str) -> usize {
    if name.is_empty() {
        com_error(errorParm_t::ERR_FATAL, "S_FindName: empty name\n".to_string());
    }

    if name.len() >= MAX_QPATH {
        com_error(
            errorParm_t::ERR_FATAL,
            format!("Sound name too long: {name}"),
        );
    }

    let sSoundNameNoExt = COM_StripExtension(name);

    let hash = S_HashSFXName(&sSoundNameNoExt);

    // see if already loaded
    let mut walk = snd.sfxHash[hash];
    while let Some(slot) = walk {
        if Q_stricmp(&snd.s_knownSfx[slot].sSoundName, &sSoundNameNoExt) == 0 {
            return slot;
        }
        walk = snd.s_knownSfx[slot].next;
    }

    // we don't clear the soundName after failed loads any more, so it'll always be the last entry
    let mut i = snd.s_knownSfx.len();

    if snd.s_knownSfx.len() == MAX_SFX {
        // ok, no sfx's free, but are there any with defaultSound set? (which the registering ent
        // will never see because he gets zero returned if it's default...)
        i = snd
            .s_knownSfx
            .iter()
            .position(|sfx| sfx.bDefaultSound)
            .unwrap_or(snd.s_knownSfx.len());

        if i == snd.s_knownSfx.len() {
            // genuinely out of handles...
            com_error(
                errorParm_t::ERR_FATAL,
                "S_FindName: out of sfx_t".to_string(),
            );
        }
        snd.s_knownSfx[i] = Default::default();
    } else {
        snd.s_knownSfx.push(Default::default());
    }

    snd.s_knownSfx[i].sSoundName = sSoundNameNoExt.to_ascii_lowercase(); // force it down low

    snd.s_knownSfx[i].next = snd.sfxHash[hash];
    snd.sfxHash[hash] = Some(i);

    i
}

/// Raven `S_DefaultSound` — build the 512-sample buzz a failed load falls back to.
///
/// Only the `_DEBUG` build calls it: the retail build registers `sound/null.wav`
/// instead (DEC-62.6). It stays ported because `S_BeginRegistration` names it.
/// Source: `oracle/codemp/client/snd_dma.cpp:866-878`
pub fn S_DefaultSound(view: &mut EngineHostView, snd: &mut SoundSystem, sfx: usize) {
    snd.s_knownSfx[sfx].iSoundLengthInSamples = 512;
    SND_malloc(view, snd, 512 * 2, sfx);
    snd.s_knownSfx[sfx].bInMemory = true;

    let data = snd.s_knownSfx[sfx]
        .pSoundData
        .as_mut()
        .and_then(SfxSampleData::pcm_mut)
        .expect("SND_malloc seated the default sound's block");
    for i in 0..data.len() {
        data[i] = i as i16;
    }
}

/// Raven `S_DisableSounds` — stop everything until the next `S_BeginRegistration`.
///
/// Raven: this is called when the hunk is cleared and the sounds are no longer valid.
/// Source: `oracle/codemp/client/snd_dma.cpp:890-893`
pub fn S_DisableSounds(common: &mut Common, snd: &mut SoundSystem) {
    S_StopAllSounds(common, snd);
    snd.s_soundMuted = true;
}

/// Raven `S_BeginRegistration` — let sound play again, and seat the sfx table on
/// the first call.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:901-933`
pub fn S_BeginRegistration(view: &mut EngineHostView, snd: &mut SoundSystem) {
    snd.s_soundMuted = false; // we can play again

    if snd.s_knownSfx.is_empty() {
        SND_setup(view, snd);

        snd.s_knownSfx.clear();
        snd.sfxHash.fill(None);

        // The `_DEBUG` build calls `S_DefaultSound` on a `***DEFAULT***` slot here.
        S_RegisterSound(view, snd, "sound/null.wav");
    }
}

/// Raven `S_RegisterSound` — return the handle of a loaded sound, and 0 where the
/// file is missing.
///
/// Raven: creates a default buzz sound if the file can't be loaded.
/// Source: `oracle/codemp/client/snd_dma.cpp:992-1042`
pub fn S_RegisterSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    name: &str,
) -> sfxHandle_t {
    if snd.s_soundStarted == 0 {
        return 0;
    }

    if name.len() >= MAX_QPATH {
        com_printf(
            view.common,
            &format!("^1Sound name exceeds MAX_QPATH - {name}\n"),
        );
        return 0;
    }

    let sfx = S_FindName(snd, name);

    SND_TouchSFX(view, snd, sfx);

    if snd.s_knownSfx[sfx].bDefaultSound {
        return 0;
    }

    if snd.s_knownSfx[sfx].pSoundData.is_some() {
        return sfx as sfxHandle_t;
    }

    snd.s_knownSfx[sfx].bInMemory = false;

    S_memoryLoad(view, snd, sfx);

    if snd.s_knownSfx[sfx].bDefaultSound {
        // The `S_COLOR_YELLOW` "using default" warning is a `!FINAL_BUILD` print,
        // and the retail build drops it (DEC-62.6).
        return 0;
    }

    sfx as sfxHandle_t
}

/// Raven `S_memoryLoad` — load one sound, and flag it default where the load fails.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1044-1054`
pub fn S_memoryLoad(view: &mut EngineHostView, snd: &mut SoundSystem, sfx: usize) {
    // load the sound file...
    if !S_LoadSound(view, snd, sfx) {
        snd.s_knownSfx[sfx].bDefaultSound = true;
    }
    snd.s_knownSfx[sfx].bInMemory = true;
}

/// Raven `S_CheckChannelStomp` — decide whether a new sound replaces a playing one.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1059-1075`
fn S_CheckChannelStomp(chan1: c_int, chan2: c_int) -> bool {
    if chan1 == chan2 {
        return true;
    }

    let voice1 = chan1 == CHAN_VOICE || chan1 == CHAN_VOICE_ATTEN || chan1 == CHAN_VOICE_GLOBAL;
    let voice2 = chan2 == CHAN_VOICE || chan2 == CHAN_VOICE_ATTEN || chan2 == CHAN_VOICE_GLOBAL;

    voice1 && voice2
}

/// Raven `S_PickChannel` — choose the channel a new sound takes, and clear it.
///
/// The first pass looks for the same entity and channel to stomp. A second pass
/// runs for every channel except `CHAN_AUTO` and `CHAN_LESS_ATTEN`, and it takes
/// the first free slot. Otherwise the oldest non-loop channel dies.
/// Source: `oracle/codemp/client/snd_dma.cpp:1089-1154`
pub fn S_PickChannel(common: &mut Common, snd: &mut SoundSystem, entnum: c_int, entchannel: c_int) -> usize {
    if entchannel < 0 {
        com_error(
            errorParm_t::ERR_DROP,
            "S_PickChannel: entchannel<0".to_string(),
        );
    }

    // Check for replacement sound, or find the best one to replace
    let mut firstToDie: usize = 0;
    let mut foundChan = false;

    let passes = if entchannel == CHAN_AUTO || entchannel == CHAN_LESS_ATTEN {
        1
    } else {
        2
    };

    let mut pass = 0;
    while pass < passes && !foundChan {
        for ch_idx in 0..MAX_CHANNELS {
            if entchannel == CHAN_AUTO || entchannel == CHAN_LESS_ATTEN || pass > 0 {
                // if we're on the second pass, just find the first open chan
                if snd.s_channels[ch_idx].thesfx.is_none() {
                    // grab the first open channel
                    firstToDie = ch_idx;
                    break;
                }
            } else if snd.s_channels[ch_idx].entnum == entnum
                && S_CheckChannelStomp(snd.s_channels[ch_idx].entchannel, entchannel)
            {
                // always override sound from same entity
                if common.cvar(snd.s_show).integer == 1 {
                    if let Some(sfx) = snd.s_channels[ch_idx].thesfx {
                        let name = snd.s_knownSfx[sfx].sSoundName.clone();
                        com_printf(common, &format!("^3...overrides {name}\n"));
                        snd.s_channels[ch_idx].thesfx = None; // just to clear the next error msg
                    }
                }
                firstToDie = ch_idx;
                foundChan = true;
                break;
            }

            // don't let anything else override local player sounds
            if snd.s_channels[ch_idx].entnum == snd.listener_number
                && entnum != snd.listener_number
                && snd.s_channels[ch_idx].thesfx.is_some()
            {
                continue;
            }

            // don't override loop sounds
            if snd.s_channels[ch_idx].loopSound {
                continue;
            }

            if snd.s_channels[ch_idx].startSample < snd.s_channels[firstToDie].startSample {
                firstToDie = ch_idx;
            }
        }
        pass += 1;
    }

    if common.cvar(snd.s_show).integer == 1 {
        if let Some(sfx) = snd.s_channels[firstToDie].thesfx {
            let name = snd.s_knownSfx[sfx].sSoundName.clone();
            com_printf(common, &format!("^1***kicking {name}\n"));
        }
    }

    Channel_Clear(snd, firstToDie);

    firstToDie
}

/// Raven `S_SpatializeOrigin` — the per-channel stereo split and distance falloff.
///
/// The returned pair is Raven's `left_vol` and `right_vol` out-params.
/// Source: `oracle/codemp/client/snd_dma.cpp:1345-1423`
pub fn S_SpatializeOrigin(
    common: &Common,
    snd: &SoundSystem,
    origin: vec3_t,
    master_vol: f32,
    channel: c_int,
) -> (c_int, c_int) {
    let mut dist_mult = SOUND_ATTENUATE;

    // calculate stereo seperation and distance attenuation
    let mut source_vec: vec3_t = [0.0; 3];
    _VectorSubtract(origin, snd.listener_origin, &mut source_vec);

    let mut dist = VectorNormalize(&mut source_vec);
    if channel == CHAN_VOICE {
        dist -= SOUND_FULLVOLUME * 3.0;
    } else if channel == CHAN_LESS_ATTEN {
        dist -= SOUND_FULLVOLUME * 8.0; // maybe is too large
    } else if channel == CHAN_VOICE_ATTEN {
        dist -= SOUND_FULLVOLUME * 1.35; // used to be 0.15f, dropped off too sharply - dmv
        dist_mult = VOICE_ATTENUATE;
    } else if channel == CHAN_VOICE_GLOBAL {
        dist = -1.0;
    } else {
        // use normal attenuation.
        dist -= SOUND_FULLVOLUME;
    }

    if dist < 0.0 {
        dist = 0.0; // close enough to be at full volume
    }
    dist *= dist_mult; // different attenuation levels

    let dot = -_DotProduct(snd.listener_axis[1], source_vec);

    let (lscale, rscale) = if snd.dma.channels == 1 {
        // no attenuation = no spatialization
        (SOUND_FMAXVOL, SOUND_FMAXVOL)
    } else {
        let separation = common.cvar(snd.s_separation).value;
        let mut rscale = separation + (1.0 - separation) * dot;
        let mut lscale = separation - (1.0 - separation) * dot;
        if rscale < 0.0 {
            rscale = 0.0;
        }
        if lscale < 0.0 {
            lscale = 0.0;
        }
        (lscale, rscale)
    };

    // add in distance effect
    let mut scale = (1.0 - dist) * rscale;
    let mut right_vol = (master_vol * scale) as c_int;
    if right_vol < 0 {
        right_vol = 0;
    }

    scale = (1.0 - dist) * lscale;
    let mut left_vol = (master_vol * scale) as c_int;
    if left_vol < 0 {
        left_vol = 0;
    }

    (left_vol, right_vol)
}

/// Raven `S_StartAmbientSound` — start a one-shot ambient sound on `CHAN_AMBIENT`.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1438-1503`
pub fn S_StartAmbientSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    origin: Option<vec3_t>,
    entityNum: c_int,
    volume: u8,
    sfxHandle: sfxHandle_t,
) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }
    if origin.is_none() && (entityNum < 0 || entityNum > MAX_GENTITIES as c_int) {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_StartAmbientSound: bad entitynum {entityNum}"),
        );
    }

    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_StartAmbientSound: handle {sfxHandle} out of range"),
        );
    }

    let sfx = sfxHandle as usize;
    if !snd.s_knownSfx[sfx].bInMemory {
        S_memoryLoad(view, snd, sfx);
    }
    SND_TouchSFX(view, snd, sfx);

    if view.common.cvar(snd.s_show).integer == 1 {
        let painted = snd.s_paintedtime;
        let name = snd.s_knownSfx[sfx].sSoundName.clone();
        com_printf(
            view.common,
            &format!("{painted} : {name} on ({entityNum}) Ambient\n"),
        );
    }

    // pick a channel to play on
    let ch = S_PickChannel(view.common, snd, entityNum, CHAN_AMBIENT);

    match origin {
        Some(org) => {
            _VectorCopy(org, &mut snd.s_channels[ch].origin);
            snd.s_channels[ch].fixed_origin = true;
        }
        None => {
            snd.s_channels[ch].fixed_origin = false;
        }
    }

    snd.s_channels[ch].master_vol = c_int::from(volume);
    snd.s_channels[ch].entnum = entityNum;
    snd.s_channels[ch].entchannel = CHAN_AMBIENT;
    snd.s_channels[ch].thesfx = Some(sfx);
    snd.s_channels[ch].startSample = START_SAMPLE_IMMEDIATE;

    // these will get calced at next spatialize, unless the game isn't running
    snd.s_channels[ch].leftvol = snd.s_channels[ch].master_vol;
    snd.s_channels[ch].rightvol = snd.s_channels[ch].master_vol;
}

/// Raven `S_MuteSound` — silence one entity channel and free its slot.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1512-1530`
pub fn S_MuteSound(
    common: &mut Common,
    snd: &mut SoundSystem,
    entityNum: c_int,
    entchannel: c_int,
) {
    // I guess this works.
    let ch = S_PickChannel(common, snd, entityNum, entchannel);

    snd.s_channels[ch].master_vol = 0;
    snd.s_channels[ch].entnum = 0;
    snd.s_channels[ch].entchannel = 0;
    snd.s_channels[ch].thesfx = None;
    snd.s_channels[ch].startSample = 0;

    snd.s_channels[ch].leftvol = 0;
    snd.s_channels[ch].rightvol = 0;
}

/// Raven `S_StartSound` — validate the parameters and queue the sound up.
///
/// Raven: if pos is NULL, the sound will be dynamically sourced from the entity.
/// Entchannel 0 will never override a playing sound.
///
/// Raven's bound is `entityNum > MAX_GENTITIES`, and it skips the check outright
/// for a fixed origin, so an out-of-range number writes past the per-entity
/// tables. The port panics on that index instead (porting-rules §19).
/// Source: `oracle/codemp/client/snd_dma.cpp:1541-1648`
pub fn S_StartSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    origin: Option<vec3_t>,
    entityNum: c_int,
    entchannel: c_int,
    sfxHandle: sfxHandle_t,
) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

    if origin.is_none() && (entityNum < 0 || entityNum > MAX_GENTITIES as c_int) {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_StartSound: bad entitynum {entityNum}"),
        );
    }

    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_StartSound: handle {sfxHandle} out of range"),
        );
    }

    let sfx = sfxHandle as usize;
    if !snd.s_knownSfx[sfx].bInMemory {
        S_memoryLoad(view, snd, sfx);
    }
    SND_TouchSFX(view, snd, sfx);

    if view.common.cvar(snd.s_show).integer == 1 {
        let painted = snd.s_paintedtime;
        let name = snd.s_knownSfx[sfx].sSoundName.clone();
        com_printf(view.common, &format!("{painted} : {name} on ({entityNum})\n"));
    }

    // pick a channel to play on
    let ch = S_PickChannel(view.common, snd, entityNum, entchannel);

    match origin {
        Some(org) => {
            _VectorCopy(org, &mut snd.s_channels[ch].origin);
            snd.s_channels[ch].fixed_origin = true;
        }
        None => {
            snd.s_channels[ch].fixed_origin = false;
        }
    }

    snd.s_channels[ch].master_vol = SOUND_MAXVOL; // FIXME: Um.. control?
    snd.s_channels[ch].entnum = entityNum;
    snd.s_channels[ch].entchannel = entchannel;
    snd.s_channels[ch].thesfx = Some(sfx);
    snd.s_channels[ch].startSample = START_SAMPLE_IMMEDIATE;

    // these will get calced at next spatialize, unless the game isn't running
    snd.s_channels[ch].leftvol = snd.s_channels[ch].master_vol;
    snd.s_channels[ch].rightvol = snd.s_channels[ch].master_vol;

    if entchannel < CHAN_AMBIENT && entityNum == snd.listener_number {
        // only do it for body sounds not local sounds; this won't be attenuated so let it scale down
        snd.s_channels[ch].master_vol = (SOUND_MAXVOL as f32 * SOUND_FMAXVOL) as c_int;
    }
    if entchannel == CHAN_VOICE || entchannel == CHAN_VOICE_ATTEN || entchannel == CHAN_VOICE_GLOBAL
    {
        // we've started the sound but it's silent for now
        snd.s_entityWavVol[snd.s_channels[ch].entnum as usize] = -1;
    }
}

/// Raven `S_StartLocalSound` — start a sound on the listener entity.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1655-1665`
pub fn S_StartLocalSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    sfxHandle: sfxHandle_t,
    channelNum: c_int,
) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_StartLocalSound: handle {sfxHandle} out of range"),
        );
    }

    let listener = snd.listener_number;
    S_StartSound(view, snd, None, listener, channelNum, sfxHandle);
}

/// Raven `S_StartLocalLoopingSound` — add a head-relative looping sound.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1673-1686`
pub fn S_StartLocalLoopingSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    sfxHandle: sfxHandle_t,
) {
    let nullVec: vec3_t = [0.0, 0.0, 0.0];

    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_StartLocalLoopingSound: handle {sfxHandle} out of range"),
        );
    }

    let listener = snd.listener_number;
    S_AddLoopingSound(view, snd, listener, nullVec, nullVec, sfxHandle);
}

/// Raven `S_GetSampleLengthInMilliSeconds` — the play time of one sound.
///
/// A bad handle answers 0, and a dead sound system answers Raven's 512-second guess.
/// Source: `oracle/codemp/client/snd_dma.cpp:1690-1707`
pub fn S_GetSampleLengthInMilliSeconds(snd: &SoundSystem, sfxHandle: sfxHandle_t) -> f32 {
    if snd.s_soundStarted == 0 {
        // we have no sound, so let's just make a reasonable guess
        return 512.0 * 1000.0;
    }

    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        return 0.0;
    }

    let f = snd.s_knownSfx[sfxHandle as usize].iSoundLengthInSamples as f32 / snd.dma.speed as f32;

    f * 1000.0
}

/// Raven `S_ClearSoundBuffer` — silence the ring before a file-access stall.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1718-1750`
pub fn S_ClearSoundBuffer(snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }
    // Raven's `#if 0` channel wipe is compiled out in this build.
    snd.s_rawend = 0;

    let clear = if snd.dma.samplebits == 8 { 0x80u8 } else { 0 };

    if !snd.dma.buffer.is_empty() {
        let bytes = (snd.dma.samples * snd.dma.samplebits / 8) as usize;
        snd.dma.buffer[..bytes].fill(clear);
    }
}

/// Raven `S_CIN_StopSound` — drop the cinematic sound off whichever channel holds it.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1755-1786`
pub fn S_CIN_StopSound(snd: &mut SoundSystem, sfxHandle: sfxHandle_t) {
    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_CIN_StopSound: handle {sfxHandle} out of range"),
        );
    }

    for i in 0..MAX_CHANNELS {
        let ch = snd.s_channels[i];
        let Some(sfx) = ch.thesfx else {
            continue;
        };
        if f64::from(ch.leftvol) < 0.25 && f64::from(ch.rightvol) < 0.25 {
            continue;
        }
        if sfx == sfxHandle as usize {
            SND_FreeSFXMem(snd, sfx); // heh, may as well...
            snd.s_channels[i].thesfx = None;
            break;
        }
    }
}

/// Raven `S_StopSounds` — stop every effect and clear the ring, music excluded.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1794-1831`
pub fn S_StopSounds(snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 {
        return;
    }

    // stop looping sounds
    S_ClearLoopingSounds(snd);

    // clear all the s_channels
    snd.s_channels.fill(channel_t::default());

    // clear out the lip synching override array
    snd.s_entityWavVol.fill(0);

    S_ClearSoundBuffer(snd);
}

/// Raven `S_StopAllSounds` — `S_StopSounds` plus the background track.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1839-1847`
pub fn S_StopAllSounds(common: &mut Common, snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 {
        return;
    }
    // stop the background music
    S_StopBackgroundTrack(common, snd);

    S_StopSounds(snd);
}

/// Raven `S_ClearLoopingSounds` — empty the per-frame loop list.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1863-1874`
pub fn S_ClearLoopingSounds(snd: &mut SoundSystem) {
    snd.numLoopSounds = 0;
}

/// Raven `S_StopLoopingSound` — drop every loop entry an entity owns.
///
/// Raven: sort of a slow method though, isn't there some better way?
/// Source: `oracle/codemp/client/snd_dma.cpp:1884-1902`
pub fn S_StopLoopingSound(snd: &mut SoundSystem, entityNum: c_int) {
    let mut i = 0;

    while i < snd.numLoopSounds {
        if snd.loopSounds[i as usize].entnum == entityNum {
            let mut x = i + 1;
            while x < snd.numLoopSounds {
                snd.loopSounds[(x - 1) as usize] = snd.loopSounds[x as usize];
                x += 1;
            }
            snd.numLoopSounds -= 1;
        }
        i += 1;
    }
}

/// Raven `S_AddLoopingSound` — add one looping sound for this frame.
///
/// Raven: called during entity generation for a frame. Include velocity in case
/// I get around to doing doppler...
/// Source: `oracle/codemp/client/snd_dma.cpp:1912-1942`
pub fn S_AddLoopingSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    entityNum: c_int,
    origin: vec3_t,
    velocity: vec3_t,
    sfxHandle: sfxHandle_t,
) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }
    if snd.numLoopSounds >= MAX_LOOP_SOUNDS as c_int {
        return;
    }

    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_AddLoopingSound: handle {sfxHandle} out of range"),
        );
    }

    let sfx = sfxHandle as usize;
    if !snd.s_knownSfx[sfx].bInMemory {
        S_memoryLoad(view, snd, sfx);
    }
    SND_TouchSFX(view, snd, sfx);

    if snd.s_knownSfx[sfx].iSoundLengthInSamples == 0 {
        let name = snd.s_knownSfx[sfx].sSoundName.clone();
        com_error(errorParm_t::ERR_DROP, format!("{name} has length 0"));
    }

    let slot = snd.numLoopSounds as usize;
    _VectorCopy(origin, &mut snd.loopSounds[slot].origin);
    _VectorCopy(velocity, &mut snd.loopSounds[slot].velocity);
    snd.loopSounds[slot].sfx = Some(sfx);
    snd.loopSounds[slot].volume = SOUND_MAXVOL as u8;
    snd.loopSounds[slot].entnum = entityNum;
    snd.numLoopSounds += 1;
}

/// Raven `S_AddAmbientLoopingSound` — a looping sound with its own volume and no entity.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:1950-1987`
pub fn S_AddAmbientLoopingSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    origin: vec3_t,
    volume: u8,
    sfxHandle: sfxHandle_t,
) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }
    if snd.numLoopSounds >= MAX_LOOP_SOUNDS as c_int {
        return;
    }

    if sfxHandle < 0 || sfxHandle >= snd.s_numSfx() {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_StartSound: handle {sfxHandle} out of range"),
        );
    }

    let sfx = sfxHandle as usize;
    if !snd.s_knownSfx[sfx].bInMemory {
        S_memoryLoad(view, snd, sfx);
    }
    SND_TouchSFX(view, snd, sfx);

    if snd.s_knownSfx[sfx].iSoundLengthInSamples == 0 {
        let name = snd.s_knownSfx[sfx].sSoundName.clone();
        com_error(errorParm_t::ERR_DROP, format!("{name} has length 0"));
    }

    let slot = snd.numLoopSounds as usize;
    _VectorCopy(origin, &mut snd.loopSounds[slot].origin);
    snd.loopSounds[slot].sfx = Some(sfx);

    // Raven's TODO: Calculate the distance falloff
    snd.loopSounds[slot].volume = volume;
    snd.numLoopSounds += 1;
}

/// Raven `S_AddLoopSounds` — spatialize the loop list and merge duplicates onto
/// one channel each.
///
/// Raven: all sounds are on the same cycle, so any duplicates can just sum up the
/// channel multipliers.
/// Source: `oracle/codemp/client/snd_dma.cpp:2000-2059`
fn S_AddLoopSounds(common: &mut Common, snd: &mut SoundSystem) {
    snd.loopFrame += 1;
    for i in 0..snd.numLoopSounds as usize {
        if snd.loopSounds[i].mergeFrame == snd.loopFrame {
            continue; // already merged into an earlier sound
        }

        // find the total contribution of all sounds of this type
        let mut left_total = 0;
        let mut right_total = 0;

        for j in i..snd.numLoopSounds as usize {
            if snd.loopSounds[j].sfx != snd.loopSounds[i].sfx {
                continue;
            }
            snd.loopSounds[j].mergeFrame = snd.loopFrame; // don't check this again later

            // FIXME: Allow for volume change!!
            let (left, right) = S_SpatializeOrigin(
                common,
                snd,
                snd.loopSounds[j].origin,
                f32::from(snd.loopSounds[j].volume),
                CHAN_AUTO,
            );

            left_total += left;
            right_total += right;
        }

        if left_total == 0 && right_total == 0 {
            continue; // not audible
        }

        // allocate a channel
        let ch = S_PickChannel(common, snd, 0, 0);

        if left_total > SOUND_MAXVOL {
            left_total = SOUND_MAXVOL;
        }
        if right_total > SOUND_MAXVOL {
            right_total = SOUND_MAXVOL;
        }
        snd.s_channels[ch].leftvol = left_total;
        snd.s_channels[ch].rightvol = right_total;
        snd.s_channels[ch].loopSound = true; // remove next frame
        snd.s_channels[ch].thesfx = snd.loopSounds[i].sfx;
    }
}

// Raven's `S_ByteSwapRawSamples` returns at once where `LittleShort(256) == 256`,
// which is every target this tree builds for, so the streamed-WAV call site
// drops it.
// Source: `oracle/codemp/client/snd_dma.cpp:2071-2087`

// Raven's `S_GetRawSamplePointer` hands `s_rawsamples` back to a caller that no
// tree has (porting-rules §20 drops a zero-caller API). `SoundSystem` owns the
// ring, so a reader takes it by field.
// Source: `oracle/codemp/client/snd_dma.cpp:2090-2092`

/// Raven `S_RawSamples` — write a streamed block into the raw ring the paint
/// chain reads ahead of the channels.
///
/// Raven: cinematics and voice-over-network send raw samples, and 1.0 volume is
/// the direct output of the source samples.
/// Source: `oracle/codemp/client/snd_dma.cpp:2102-2273`
#[allow(clippy::too_many_arguments)]
pub fn S_RawSamples(
    common: &mut Common,
    snd: &mut SoundSystem,
    samples: c_int,
    rate: c_int,
    width: c_int,
    s_channels: c_int,
    data: &[u8],
    volume: f32,
    bFirstOrOnlyUpdateThisFrame: bool,
) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

    let mut intVolume = (256.0 * volume) as c_int;

    if snd.s_rawend < snd.s_soundtime {
        let (rawend, soundtime) = (snd.s_rawend, snd.s_soundtime);
        Com_DPrintf(
            common,
            &format!("S_RawSamples: resetting minimum: {rawend} < {soundtime}\n"),
        );
        snd.s_rawend = snd.s_soundtime;
    }

    let scale = rate as f32 / snd.dma.speed as f32;

    // Reads one 16-bit source sample at the given short index.
    let short_at = |index: usize| -> c_int {
        c_int::from(i16::from_le_bytes([data[index * 2], data[index * 2 + 1]]))
    };

    if s_channels == 2 && width == 2 {
        if scale == 1.0 {
            // optimized case
            for i in 0..samples as usize {
                let dst = (snd.s_rawend & (MAX_RAW_SAMPLES as c_int - 1)) as usize;
                snd.s_rawend += 1;
                let (left, right) = (short_at(i * 2) * intVolume, short_at(i * 2 + 1) * intVolume);
                if bFirstOrOnlyUpdateThisFrame {
                    snd.s_rawsamples[dst].left = left;
                    snd.s_rawsamples[dst].right = right;
                } else {
                    snd.s_rawsamples[dst].left += left;
                    snd.s_rawsamples[dst].right += right;
                }
            }
        } else {
            for i in 0.. {
                let src = (i as f32 * scale) as c_int;
                if src >= samples {
                    break;
                }
                let src = src as usize;
                let dst = (snd.s_rawend & (MAX_RAW_SAMPLES as c_int - 1)) as usize;
                snd.s_rawend += 1;
                let (left, right) = (
                    short_at(src * 2) * intVolume,
                    short_at(src * 2 + 1) * intVolume,
                );
                if bFirstOrOnlyUpdateThisFrame {
                    snd.s_rawsamples[dst].left = left;
                    snd.s_rawsamples[dst].right = right;
                } else {
                    snd.s_rawsamples[dst].left += left;
                    snd.s_rawsamples[dst].right += right;
                }
            }
        }
    } else if s_channels == 1 && width == 2 {
        for i in 0.. {
            let src = (i as f32 * scale) as c_int;
            if src >= samples {
                break;
            }
            let src = src as usize;
            let dst = (snd.s_rawend & (MAX_RAW_SAMPLES as c_int - 1)) as usize;
            snd.s_rawend += 1;
            let value = short_at(src) * intVolume;
            if bFirstOrOnlyUpdateThisFrame {
                snd.s_rawsamples[dst].left = value;
                snd.s_rawsamples[dst].right = value;
            } else {
                snd.s_rawsamples[dst].left += value;
                snd.s_rawsamples[dst].right += value;
            }
        }
    } else if s_channels == 2 && width == 1 {
        intVolume *= 256;

        for i in 0.. {
            let src = (i as f32 * scale) as c_int;
            if src >= samples {
                break;
            }
            let src = src as usize;
            let dst = (snd.s_rawend & (MAX_RAW_SAMPLES as c_int - 1)) as usize;
            snd.s_rawend += 1;
            let left = c_int::from(data[src * 2] as i8) * intVolume;
            let right = c_int::from(data[src * 2 + 1] as i8) * intVolume;
            if bFirstOrOnlyUpdateThisFrame {
                snd.s_rawsamples[dst].left = left;
                snd.s_rawsamples[dst].right = right;
            } else {
                snd.s_rawsamples[dst].left += left;
                snd.s_rawsamples[dst].right += right;
            }
        }
    } else if s_channels == 1 && width == 1 {
        intVolume *= 256;

        for i in 0.. {
            let src = (i as f32 * scale) as c_int;
            if src >= samples {
                break;
            }
            let src = src as usize;
            let dst = (snd.s_rawend & (MAX_RAW_SAMPLES as c_int - 1)) as usize;
            snd.s_rawend += 1;
            let value = (c_int::from(data[src]) - 128) * intVolume;
            if bFirstOrOnlyUpdateThisFrame {
                snd.s_rawsamples[dst].left = value;
                snd.s_rawsamples[dst].right = value;
            } else {
                snd.s_rawsamples[dst].left += value;
                snd.s_rawsamples[dst].right += value;
            }
        }
    }

    if snd.s_rawend > snd.s_soundtime + MAX_RAW_SAMPLES as c_int {
        let (rawend, soundtime) = (snd.s_rawend, snd.s_soundtime);
        Com_DPrintf(
            common,
            &format!("S_RawSamples: overflowed {rawend} > {soundtime}\n"),
        );
    }
}

/// Raven `S_UpdateEntityPosition` — record where an entity is this frame.
///
/// Raven's bound is `entityNum > MAX_GENTITIES`, so `MAX_GENTITIES` itself writes
/// one entry past the table. The port panics on that index instead (§19).
/// Source: `oracle/codemp/client/snd_dma.cpp:2284-2330`
pub fn S_UpdateEntityPosition(snd: &mut SoundSystem, entityNum: c_int, origin: vec3_t) {
    if entityNum < 0 || entityNum > MAX_GENTITIES as c_int {
        com_error(
            errorParm_t::ERR_DROP,
            format!("S_UpdateEntityPosition: bad entitynum {entityNum}"),
        );
    }

    _VectorCopy(origin, &mut snd.s_entityPosition[entityNum as usize]);
}

/// Raven `S_CheckAmplitude` — the lip-sync amplitude of the sound one channel
/// plays right now.
///
/// The scan reads ten samples 100 apart, squares them, and buckets the mean
/// against the four `s_threshold` cvars. Raven reads outside the sample block in
/// two cases: its guard lets the last read run one sample past the end, and a
/// negative `offset` reads below the start. The port answers 0 for both (§19).
/// Source: `oracle/codemp/client/snd_dma.cpp:2338-2456`
fn S_CheckAmplitude(common: &Common, snd: &mut SoundSystem, channel: usize, s_oldpaintedtime: u32) -> c_int {
    let ch = snd.s_channels[channel];
    let sfx = ch.thesfx.expect("S_CheckAmplitude on a free channel");

    // now, is this a cycle - or have we just started a new sample - where we should update the
    // backup table, and write this value into the new table? or should we just take the value FROM
    // the back up table and feed it out.
    if ch.startSample == s_oldpaintedtime || snd.next_amplitude < snd.s_soundtime {
        let mut sample;
        let mut sample_total = 0;
        let mut count = 0;

        // if we haven't started the sample yet, we must be at the beginning; figure out where we
        // are in the sample right now.
        let offset = s_oldpaintedtime.wrapping_sub(ch.startSample) as c_int;

        // scan through 10 samples 100( at 11hz or 200 at 22hz) samples apart.
        for i in 0..10 {
            // have we run off the end?
            if (offset + (i * 100)) > snd.s_knownSfx[sfx].iSoundLengthInSamples {
                break;
            }

            let index = offset + i * 100;
            sample = c_int::from(
                usize::try_from(index)
                    .ok()
                    .and_then(|index| {
                        snd.s_knownSfx[sfx]
                            .pSoundData
                            .as_ref()?
                            .pcm()?
                            .get(index)
                            .copied()
                    })
                    .unwrap_or(0),
            );
            sample >>= 8;

            // square it for better accuracy
            sample_total += sample * sample;
            count += 1;
        }

        // if we are already done with this sample, then its silence
        if count == 0 {
            return 0;
        }
        sample_total /= count;

        let volRange = snd.s_knownSfx[sfx].fVolRange;
        let total = sample_total as f32;

        // I hate doing this, but its the simplest way
        sample = if total < volRange * common.cvar(snd.s_lip_threshold_1).value {
            // tell the scripts that are relying on this that we are still going, but actually
            // silent right now.
            -1
        } else if total < volRange * common.cvar(snd.s_lip_threshold_2).value {
            1
        } else if total < volRange * common.cvar(snd.s_lip_threshold_3).value {
            2
        } else if total < volRange * common.cvar(snd.s_lip_threshold_4).value {
            3
        } else {
            4
        };

        // store away the value we got into the back up table
        snd.s_entityWavVol_back[ch.entnum as usize] = sample;
        return sample;
    }

    // no, just get last value calculated from backup table
    snd.s_entityWavVol_back[ch.entnum as usize]
}

/// Raven `S_Respatialize` — recompute every channel volume for the new listener,
/// then merge the loop list into channels.
///
/// A channel that falls silent is freed here, and a voice channel keeps playing
/// out of range so a script waiting on it still finishes.
/// Source: `oracle/codemp/client/snd_dma.cpp:2464-2610`
pub fn S_Respatialize(
    common: &mut Common,
    snd: &mut SoundSystem,
    entityNum: c_int,
    head: vec3_t,
    axis: [vec3_t; 3],
    _inwater: c_int,
) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

    // `inwater` only drives the dropped EAX arm (DEC-57.4), so the software
    // mixer ignores it.
    snd.listener_number = entityNum;
    _VectorCopy(head, &mut snd.listener_origin);
    _VectorCopy(axis[0], &mut snd.listener_axis[0]);
    _VectorCopy(axis[1], &mut snd.listener_axis[1]);
    _VectorCopy(axis[2], &mut snd.listener_axis[2]);

    // update spatialization for dynamic sounds
    for i in 0..MAX_CHANNELS {
        let ch = snd.s_channels[i];
        if ch.thesfx.is_none() {
            continue;
        }
        if ch.loopSound {
            // loopSounds are regenerated fresh each frame
            Channel_Clear(snd, i);
            continue;
        }

        // anything coming from the view entity will always be full volume
        if ch.entnum == snd.listener_number {
            snd.s_channels[i].leftvol = ch.master_vol;
            snd.s_channels[i].rightvol = ch.master_vol;
        } else {
            let origin = if ch.fixed_origin {
                ch.origin
            } else {
                snd.s_entityPosition[ch.entnum as usize]
            };

            let (left, right) =
                S_SpatializeOrigin(common, snd, origin, ch.master_vol as f32, ch.entchannel);
            snd.s_channels[i].leftvol = left;
            snd.s_channels[i].rightvol = right;
        }

        // NOTE: Made it so that voice sounds keep playing, even out of range
        //		so that tasks waiting for sound completion keep proper timing
        let voice = ch.entchannel == CHAN_VOICE
            || ch.entchannel == CHAN_VOICE_ATTEN
            || ch.entchannel == CHAN_VOICE_GLOBAL;
        if !voice && snd.s_channels[i].leftvol == 0 && snd.s_channels[i].rightvol == 0 {
            Channel_Clear(snd, i);
            continue;
        }
    }

    // add loopsounds
    S_AddLoopSounds(common, snd);
}

/// Raven `S_ScanChannelStarts` — stamp the start time on new channels and free
/// the finished ones.
///
/// Returns true where at least one sound started since the last mix.
/// Source: `oracle/codemp/client/snd_dma.cpp:2620-2652`
fn S_ScanChannelStarts(snd: &mut SoundSystem) -> bool {
    let mut newSamples = false;

    for i in 0..MAX_CHANNELS {
        let ch = snd.s_channels[i];
        let Some(sfx) = ch.thesfx else {
            continue;
        };
        if ch.loopSound {
            continue;
        }

        // if this channel was just started this frame,
        // set the sample count to it begins mixing
        // into the very first sample
        if ch.startSample == START_SAMPLE_IMMEDIATE {
            snd.s_channels[i].startSample = snd.s_paintedtime as u32;
            newSamples = true;
            continue;
        }

        // if it is completely finished by now, clear it
        if ch.startSample as c_int + snd.s_knownSfx[sfx].iSoundLengthInSamples <= snd.s_paintedtime
        {
            snd.s_channels[i].thesfx = None;
            continue;
        }
    }

    newSamples
}

/// Raven `S_DoLipSynchs` — refresh the per-entity mouth amplitudes after the paint.
///
/// Raven runs it after the paint, because only the painter unpacks the MP3 data
/// the amplitude scan reads.
/// Source: `oracle/codemp/client/snd_dma.cpp:2657-2691`
fn S_DoLipSynchs(common: &mut Common, snd: &mut SoundSystem, s_oldpaintedtime: u32) {
    // clear out the lip synching override array for this frame
    snd.s_entityWavVol.fill(0);

    for i in 0..MAX_CHANNELS {
        let ch = snd.s_channels[i];
        let Some(sfx) = ch.thesfx else {
            continue;
        };
        if ch.loopSound {
            continue;
        }

        // if we are playing a sample that should override the lip texture on its owning model,
        // lets figure out what the amplitude is, stick it in a table, then return it
        if ch.entchannel == CHAN_VOICE
            || ch.entchannel == CHAN_VOICE_ATTEN
            || ch.entchannel == CHAN_VOICE_GLOBAL
        {
            // go away and work out amplitude for this sound we are playing right now.
            let vol = S_CheckAmplitude(common, snd, i, s_oldpaintedtime);
            snd.s_entityWavVol[ch.entnum as usize] = vol;
            if common.cvar(snd.s_show).integer == 3 {
                let entnum = ch.entnum;
                let name = snd.s_knownSfx[sfx].sSoundName.clone();
                com_printf(common, &format!("({entnum}){i} {name} vol = {vol}\n"));
            }
        }
    }

    if snd.next_amplitude < snd.s_soundtime {
        snd.next_amplitude = snd.s_soundtime + 800;
    }
}

/// Raven `S_Update` — the once-per-frame driver.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:2700-2741`
pub fn S_Update(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

    let common = &mut *view.common;

    // debugging output
    if common.cvar(snd.s_show).integer == 2 {
        let mut total = 0;
        let mut totalMeg = 0;
        for i in 0..MAX_CHANNELS {
            let ch = snd.s_channels[i];
            let Some(sfx) = ch.thesfx else {
                continue;
            };
            if ch.leftvol != 0 || ch.rightvol != 0 {
                let (entnum, leftvol, rightvol) = (ch.entnum, ch.leftvol, ch.rightvol);
                let name = snd.s_knownSfx[sfx].sSoundName.clone();
                com_printf(
                    common,
                    &format!("({entnum}) {leftvol:3} {rightvol:3} {name}\n"),
                );
                total += 1;
                totalMeg += SND_MemUsed(snd, sfx);
            }
        }

        if total != 0 {
            let painted = snd.s_paintedtime;
            let megs = totalMeg as f32 / 1024.0 / 1024.0;
            com_printf(
                common,
                &format!("----({total})---- painted: {painted}, SND {megs:.2}MB\n"),
            );
        }
    }

    // add raw data from streamed samples
    S_UpdateBackgroundTrack(view, snd);

    // mix some sound
    S_Update_(view, snd);
}

/// Raven `S_GetSoundtime` — read the device cursor and set the mix window.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:2743-2784`
fn S_GetSoundtime(common: &mut Common, snd: &mut SoundSystem) {
    let fullsamples = snd.dma.samples / snd.dma.channels;

    // it is possible to miscount buffers if it has wrapped twice between
    // calls to S_Update.  Oh well.
    let samplepos = SNDDMA_GetDMAPos(snd);
    if samplepos < snd.oldsamplepos {
        snd.buffers += 1; // buffer wrapped

        if snd.s_paintedtime > 0x4000_0000 {
            // time to chop things off to avoid 32 bit limits
            snd.buffers = 0;
            snd.s_paintedtime = fullsamples;
            S_StopAllSounds(common, snd);
        }
    }
    snd.oldsamplepos = samplepos;

    snd.s_soundtime = snd.buffers * fullsamples + samplepos / snd.dma.channels;

    if snd.dma.submission_chunk < 256 {
        snd.s_paintedtime = (snd.s_soundtime as f32
            + common.cvar(snd.s_mixPreStep).value * snd.dma.speed as f32)
            as c_int;
    } else {
        snd.s_paintedtime = snd.s_soundtime + snd.dma.submission_chunk;
    }
}

/// Raven `S_Update_` — paint the mix window and refresh the lip-sync table.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:2787-3072`
pub fn S_Update_(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

    let common = &mut *view.common;

    // Updates s_soundtime
    S_GetSoundtime(common, snd);

    let s_oldpaintedtime = snd.s_paintedtime as u32;

    // clear any sound effects that end before the current time,
    // and start any new sounds
    S_ScanChannelStarts(snd);

    // mix ahead of current position
    // Raven declares `endtime` unsigned, so the block rounding and the
    // buffer-length clamp below both run in unsigned arithmetic.
    let mut endtime =
        (snd.s_soundtime as f32 + common.cvar(snd.s_mixahead).value * snd.dma.speed as f32) as c_int
            as u32;

    // mix to an even submission block size
    let chunk = snd.dma.submission_chunk as u32;
    endtime = endtime.wrapping_add(chunk).wrapping_sub(1) & !chunk.wrapping_sub(1);

    // never mix more than the complete buffer
    let samps = snd.dma.samples >> (snd.dma.channels - 1);
    if endtime.wrapping_sub(snd.s_soundtime as u32) > samps as u32 {
        endtime = (snd.s_soundtime + samps) as u32;
    }

    S_PaintChannels(common, snd, endtime as c_int);

    S_DoLipSynchs(common, snd, s_oldpaintedtime);
}

// ===========================================================================
// Sound memory manager
// ===========================================================================

/// Raven `SND_malloc` — size the sfx sample block, and page old sounds out where
/// `s_soundpoolmegs` is negative.
///
/// Raven returns the block and the caller stores it, so the port sizes the sfx's
/// own `pSoundData` instead (porting-rules §C9). `iSize` stays Raven's byte count.
/// Source: `oracle/codemp/client/snd_dma.cpp:5017-5034`
pub fn SND_malloc(view: &mut EngineHostView, snd: &mut SoundSystem, iSize: c_int, sfx: usize) {
    // don't bother asking for zeroed mem
    let bytes = iSize.max(0) as usize;
    snd.s_knownSfx[sfx].pSoundData = Some(
        match snd.s_knownSfx[sfx].eSoundCompressionMethod {
            // The MP3 arm keeps the raw file image, which is byte sized.
            SoundCompressionMethod_t::ct_MP3 => SfxSampleData::Mp3(vec![0u8; bytes]),
            _ => SfxSampleData::Pcm(vec![0i16; bytes / 2]),
        },
    );
    snd.sndRawDataBytes += iSize;

    // if "s_soundpoolmegs" is < 0, then the -ve of the value is the maximum
    // amount of sounds we're allowed to have loaded...
    if snd.s_soundpoolmegs.is_some() && view.common.cvar(snd.s_soundpoolmegs).integer < 0 {
        let cap = -view.common.cvar(snd.s_soundpoolmegs).integer * 1024 * 1024;
        while snd.sndRawDataBytes > cap {
            let iBytesFreed = SND_FreeOldestSound(view, snd, Some(sfx));
            if iBytesFreed == 0 {
                break; // sanity
            }
        }
    }
}

/// Raven `SND_setup` — register the pool-size cvar once per process.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5039-5048`
fn SND_setup(view: &mut EngineHostView, snd: &mut SoundSystem) {
    snd.s_soundpoolmegs = Some(Cvar_Get(view, "s_soundpoolmegs", "25", CVAR_ARCHIVE));
    if Sys_LowPhysicalMemory() != 0 {
        Cvar_Set(view, "s_soundpoolmegs", "0");
    }

    com_printf(view.common, "Sound memory manager started\n");
}

/// Raven `SND_MemUsed` — the bytes one sfx holds.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5053-5065`
pub fn SND_MemUsed(snd: &SoundSystem, sfx: usize) -> c_int {
    match &snd.s_knownSfx[sfx].pSoundData {
        Some(data) => data.byte_len() as c_int,
        None => 0,
    }
}

/// Raven `SND_FreeSFXMem` — release one sfx's samples and report the bytes freed.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5071-5115`
fn SND_FreeSFXMem(snd: &mut SoundSystem, sfx: usize) -> c_int {
    let iBytesFreed = SND_MemUsed(snd, sfx);
    snd.s_knownSfx[sfx].pSoundData = None;
    snd.sndRawDataBytes -= iBytesFreed;

    // Raven frees the MP3 stream header with the samples.
    snd.s_knownSfx[sfx].pMP3StreamHeader = None;

    snd.s_knownSfx[sfx].bInMemory = false;

    iBytesFreed
}

/// Raven `S_DisplayFreeMemory` — print the audio pool totals, and print nothing
/// where the pool is empty.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5117-5144`
fn S_DisplayFreeMemory(view: &mut EngineHostView, snd: &SoundSystem) {
    let iSoundDataSize = snd.sndRawDataBytes;
    // Raven's `Z_MemSize(TAG_SND_DYNAMICMUSIC)`: the loaded dynamic-music blocks.
    let iMusicDataSize: c_int = snd
        .tMusic_Info
        .iter()
        .map(|track| track.pLoadedData.len() as c_int)
        .sum();

    if iSoundDataSize != 0 || iMusicDataSize != 0 {
        let total = (iSoundDataSize + iMusicDataSize) as f32 / 1024.0 / 1024.0;
        let wav = iSoundDataSize as f32 / 1024.0 / 1024.0;
        let music = iMusicDataSize as f32 / 1024.0 / 1024.0;
        com_printf(
            view.common,
            &format!("\n{total:.2}MB audio data:  ( {wav:.2}MB WAV/MP3 ) + ( {music:.2}MB Music )\n"),
        );

        // now count up amount used on this level...
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let level = unsafe { rm_from_view(view) }.media_get_level();
        let mut levelBytes = 0;
        for i in 1..snd.s_knownSfx.len() {
            if snd.s_knownSfx[i].iLastLevelUsedOn == level {
                levelBytes += SND_MemUsed(snd, i);
            }
        }

        let megs = levelBytes as f32 / 1024.0 / 1024.0;
        com_printf(
            view.common,
            &format!("{megs:.2}MB in sfx_t alloc data (WAV/MP3) loaded this level\n"),
        );
    }
}

/// Raven `SND_TouchSFX` — stamp the sound with the time and the level it was used on.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5146-5150`
pub fn SND_TouchSFX(view: &mut EngineHostView, snd: &mut SoundSystem, sfx: usize) {
    snd.s_knownSfx[sfx].iLastTimeUsed = Com_Milliseconds(view) + 1;
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    snd.s_knownSfx[sfx].iLastLevelUsedOn = unsafe { rm_from_view(view) }.media_get_level();
}

/// Raven `S_FreeAllSFXMem` — release every sound except the default one.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5155-5161`
pub fn S_FreeAllSFXMem(snd: &mut SoundSystem) {
    for i in 1..snd.s_knownSfx.len() {
        SND_FreeSFXMem(snd, i);
    }
}

/// Raven `SND_FreeOldestSound` — drop the least recently used sound no channel holds.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5167-5215`
pub fn SND_FreeOldestSound(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    pButNotThisOne: Option<usize>,
) -> c_int {
    let mut iOldest = Com_Milliseconds(view);
    let mut iUsed = 0;

    // start on 1 so we never dump the default sound...
    for i in 1..snd.s_knownSfx.len() {
        if Some(i) == pButNotThisOne {
            continue;
        }
        if snd.s_knownSfx[i].bDefaultSound
            || !snd.s_knownSfx[i].bInMemory
            || snd.s_knownSfx[i].iLastTimeUsed >= iOldest
        {
            continue;
        }

        // new bit, we can't throw away any sfx_t struct in use by a channel, else the paint code
        // will crash...
        let held = snd
            .s_channels
            .iter()
            .any(|ch| ch.thesfx == Some(i));
        if !held {
            // this sfx_t struct wasn't used by any channels, so we can lose it...
            iUsed = i;
            iOldest = snd.s_knownSfx[i].iLastTimeUsed;
        }
    }

    let mut iBytesFreed = 0;
    if iUsed != 0 {
        let name = snd.s_knownSfx[iUsed].sSoundName.clone();
        Com_DPrintf(
            view.common,
            &format!("SND_FreeOldestSound: freeing sound {name}\n"),
        );

        iBytesFreed = SND_FreeSFXMem(snd, iUsed);
    }

    iBytesFreed
}

/// Raven `SND_RegisterAudio_LevelLoadEnd` — bring the audio pool back under
/// `s_soundpoolmegs` before a level starts.
///
/// Returns true where at least one sound was dropped.
/// Source: `oracle/codemp/client/snd_dma.cpp:5228-5278`
pub fn SND_RegisterAudio_LevelLoadEnd(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    bDeleteEverythingNotUsedThisLevel: bool,
) -> bool {
    let mut bAtLeastOneSoundDropped = false;

    Com_DPrintf(view.common, "SND_RegisterAudio_LevelLoadEnd():\n");

    if snd.gbInsideLoadSound {
        Com_DPrintf(
            view.common,
            "(Inside S_LoadSound (z_malloc recovery?), exiting...\n",
        );
    } else {
        let mut iLoadedAudioBytes = snd.sndRawDataBytes;
        let iMaxAudioBytes = view.common.cvar(snd.s_soundpoolmegs).integer * 1024 * 1024;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let level = unsafe { rm_from_view(view) }.media_get_level();

        // i=1 so we never page out default sound
        let mut i = 1;
        while i < snd.s_knownSfx.len()
            && (iLoadedAudioBytes > iMaxAudioBytes || bDeleteEverythingNotUsedThisLevel)
        {
            if snd.s_knownSfx[i].bInMemory {
                let bDeleteThis = if bDeleteEverythingNotUsedThisLevel {
                    snd.s_knownSfx[i].iLastLevelUsedOn != level
                } else {
                    snd.s_knownSfx[i].iLastLevelUsedOn < level
                };

                if bDeleteThis {
                    let name = snd.s_knownSfx[i].sSoundName.clone();
                    Com_DPrintf(view.common, &format!("Dumping sfx_t \"{name}\"\n"));

                    if SND_FreeSFXMem(snd, i) != 0 {
                        bAtLeastOneSoundDropped = true;
                    }

                    iLoadedAudioBytes = snd.sndRawDataBytes;
                }
            }
            i += 1;
        }
    }

    Com_DPrintf(view.common, "SND_RegisterAudio_LevelLoadEnd(): Ok\n");

    bAtLeastOneSoundDropped
}

/// Raven `S_ReloadAllUsedSounds` — reload every paged-out sound this level needs.
///
/// Raven: only called from snd_restart. QA request...
/// Source: `oracle/codemp/client/snd_dma.cpp:629-644`
pub fn S_ReloadAllUsedSounds(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if snd.s_soundStarted != 0 && !snd.s_soundMuted {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let level = unsafe { rm_from_view(view) }.media_get_level();
        // start @ 1 to skip freeing default sound
        for i in 1..snd.s_knownSfx.len() {
            if !snd.s_knownSfx[i].bInMemory
                && !snd.s_knownSfx[i].bDefaultSound
                && snd.s_knownSfx[i].iLastLevelUsedOn == level
            {
                S_memoryLoad(view, snd, i);
            }
        }
    }
}

// ===========================================================================
// Background music
// ===========================================================================

/// Raven `S_FileExists` — does this pak path resolve to a readable file?
///
/// Raven: do NOT replace this with a call to `FS_FileExists`, that is for
/// checking about writing out, and does not work for this.
/// Source: `oracle/codemp/client/snd_dma.cpp:3963-3973`
pub fn S_FileExists(view: &mut EngineHostView, psFilename: &str) -> bool {
    let mut fhTemp: fileHandle_t = 0;
    // `true` so the handle can be closed without closing a PAK.
    FS_FOpenFileRead(view, psFilename, &mut fhTemp, true);
    if fhTemp == 0 {
        return false;
    }

    FS_FCloseFile(view.common, fhTemp);
    true
}

/// Raven `FGetLittleLong` / `FGetLittleShort` — read one little-endian value off
/// an open file.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:3916-3930`
fn FGetLittleLong(common: &mut Common, f: fileHandle_t) -> c_int {
    let mut v = [0u8; 4];
    FS_Read(common, v.as_mut_ptr() as *mut (), 4, f);
    c_int::from_le_bytes(v)
}

fn FGetLittleShort(common: &mut Common, f: fileHandle_t) -> c_int {
    let mut v = [0u8; 2];
    FS_Read(common, v.as_mut_ptr() as *mut (), 2, f);
    c_int::from(i16::from_le_bytes(v))
}

/// Raven `S_FindWavChunk` — the length of the named chunk, or 0 where the next
/// chunk is a different one.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:3933-3956`
fn S_FindWavChunk(common: &mut Common, f: fileHandle_t, chunk: &str) -> c_int {
    let mut name = [0u8; 4];
    let r = FS_Read(common, name.as_mut_ptr() as *mut (), 4, f);
    if r != 4 {
        return 0;
    }
    let mut len = FGetLittleLong(common, f);
    if !(0..=0xfff_ffff).contains(&len) {
        return 0;
    }
    len = (len + 1) & !1; // pad to word boundary

    if name != chunk.as_bytes()[..4] {
        return 0;
    }
    len
}

/// Raven `MP3MusicStream_Reset` — put the disk window back on the file head.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:3977-3981`
fn MP3MusicStream_Reset(pMusicInfo: &mut MusicInfo_t) {
    pMusicInfo.iMP3MusicStream_DiskReadPos = 0;
    pMusicInfo.iMP3MusicStream_DiskWindowPos = 0;
}

/// Raven `MP3MusicStream_ReadFromDisk` — pull enough of the file into the window
/// that the decoder can read `iReadBytesNeeded` bytes at `iReadOffset`.
///
/// Raven answers the pointer the decoder should read from; the port leaves the
/// window on `pMusicInfo` and the caller reads it at
/// `iReadOffset - iMP3MusicStream_DiskWindowPos`.
/// Source: `oracle/codemp/client/snd_dma.cpp:3986-4018`
fn MP3MusicStream_ReadFromDisk(
    common: &mut Common,
    pMusicInfo: &mut MusicInfo_t,
    iReadOffset: c_int,
    iReadBytesNeeded: c_int,
) {
    if iReadOffset < pMusicInfo.iMP3MusicStream_DiskWindowPos {
        // Raven asserts here and returns the window base anyway.
        return;
    }

    while iReadOffset + iReadBytesNeeded > pMusicInfo.iMP3MusicStream_DiskReadPos {
        let at =
            (pMusicInfo.iMP3MusicStream_DiskReadPos - pMusicInfo.iMP3MusicStream_DiskWindowPos)
                .max(0) as usize;
        let room = pMusicInfo.byMP3MusicStream_DiskBuffer.len().saturating_sub(at);
        let want = (iMP3MusicStream_DiskBytesToRead as usize).min(room);
        if want == 0 {
            break;
        }
        let handle = pMusicInfo.s_backgroundFile;
        let iBytesRead = FS_Read(
            common,
            pMusicInfo.byMP3MusicStream_DiskBuffer[at..].as_mut_ptr() as *mut (),
            want as c_int,
            handle,
        );

        pMusicInfo.iMP3MusicStream_DiskReadPos += iBytesRead;

        // Quietly ignore any request to read past the file end: the disk code
        // cannot know how much source a given output size needs, so it always
        // asks for too much.
        if iBytesRead != iMP3MusicStream_DiskBytesToRead {
            break;
        }
    }

    // Once past the halfway point of the window, backscroll it.
    if pMusicInfo.iMP3MusicStream_DiskReadPos - pMusicInfo.iMP3MusicStream_DiskWindowPos
        > iMP3MusicStream_DiskBufferSize / 2
    {
        let iMoveSrcOffset = (iReadOffset - pMusicInfo.iMP3MusicStream_DiskWindowPos).max(0) as usize;
        let iMoveCount = ((pMusicInfo.iMP3MusicStream_DiskReadPos
            - pMusicInfo.iMP3MusicStream_DiskWindowPos)
            .max(0) as usize)
            .saturating_sub(iMoveSrcOffset);
        pMusicInfo
            .byMP3MusicStream_DiskBuffer
            .copy_within(iMoveSrcOffset..iMoveSrcOffset + iMoveCount, 0);
        pMusicInfo.iMP3MusicStream_DiskWindowPos += iMoveSrcOffset as c_int;
    }
}

/// Raven `S_StopBackgroundTrack_Actual` — close one music track's file.
///
/// Raven notes this does NOT reset `s_rawend`.
/// Source: `oracle/codemp/client/snd_dma.cpp:4023-4034`
fn S_StopBackgroundTrack_Actual(common: &mut Common, snd: &mut SoundSystem, track: usize) {
    if snd.tMusic_Info[track].s_backgroundFile != 0 {
        if snd.tMusic_Info[track].s_backgroundFile != -1 {
            let handle = snd.tMusic_Info[track].s_backgroundFile;
            Sys_EndStreamedFile(handle);
            FS_FCloseFile(common, handle);
        }
        snd.tMusic_Info[track].s_backgroundFile = 0;
    }
}

/// Raven `S_UnCacheDynamicMusic` — free the in-memory dynamic-music tracks.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4049-4055`
pub fn S_UnCacheDynamicMusic(snd: &mut SoundSystem) {
    // `eBGRNDTRACK_DATABEGIN` to `eBGRNDTRACK_DATAEND`, the twelve data tracks.
    for track in 0..MusicState_e::eBGRNDTRACK_NONDYNAMIC as usize {
        // Raven `FreeMusic`: the loaded block and its name stay valid together.
        snd.tMusic_Info[track].pLoadedData = Vec::new();
        snd.tMusic_Info[track].sLoadedDataName = String::new();
        snd.tMusic_Info[track].iLoadedDataLen = 0;
    }
}

/// Raven `S_StopBackgroundTrack` — close every track and reset the raw cursor.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4624-4632`
pub fn S_StopBackgroundTrack(common: &mut Common, snd: &mut SoundSystem) {
    for track in 0..BGRNDTRACK_NUMBEROF {
        S_StopBackgroundTrack_Actual(common, snd, track);
    }

    snd.s_rawend = 0;
}

/// Raven `FreeMusic` — drop one track's loaded file image.
///
/// The block and its name stay valid or invalid together.
/// Source: `oracle/codemp/client/snd_dma.cpp:4036-4045`
fn FreeMusic(pMusicInfo: &mut MusicInfo_t) {
    if !pMusicInfo.pLoadedData.is_empty() {
        pMusicInfo.pLoadedData = Vec::new();
        pMusicInfo.sLoadedDataName = String::new();
        pMusicInfo.iLoadedDataLen = 0;
    }
}

/// Raven `MusicInfo_t::SeekTo` — restart the track at one play time.
///
/// The port takes the source and the mixer rate the seek needs, which Raven
/// reads out of file scope.
/// Source: `oracle/codemp/client/snd_dma.cpp:90-96`
fn MusicInfo_SeekTo(pMusicInfo: &mut MusicInfo_t, dmaSpeed: c_int, fTime: f32) {
    pMusicInfo.chMP3_Bgrnd.iMP3SlidingDecodeWindowPos = 0;
    pMusicInfo.chMP3_Bgrnd.iMP3SlidingDecodeWritePos = 0;
    let pristine = pMusicInfo.streamMP3_Bgrnd.clone();
    let source = core::mem::take(&mut pMusicInfo.pLoadedData);
    MP3Stream_SeekTo(
        &mut pMusicInfo.chMP3_Bgrnd,
        &pristine,
        &source,
        0,
        dmaSpeed,
        fTime,
    );
    pMusicInfo.pLoadedData = source;
    pMusicInfo.s_backgroundSamples = pMusicInfo.sfxMP3_Bgrnd.iSoundLengthInSamples;
}

/// Raven `S_StartBackgroundTrack_Actual` — open one music track, MP3 or WAV.
///
/// A dynamic track is read whole into memory and its file handle drops to -1,
/// the special "valid, but not a real file" value. A non-dynamic MP3 streams off
/// disk through the window, and a WAV streams through `Sys_StreamedRead`.
/// Source: `oracle/codemp/client/snd_dma.cpp:4057-4265`
fn S_StartBackgroundTrack_Actual(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    track: usize,
    qbDynamic: bool,
    intro: &str,
    loop_name: &str,
) -> bool {
    snd.sMusic_BackgroundLoop = loop_name.to_string();
    snd.sMusic_BackgroundLoop.truncate(MAX_QPATH - 1);

    // Raven trims to `sizeof(name) - 4` so a name with no room for an extension
    // takes the soft fopen error rather than `COM_DefaultExtension`'s ERR_DROP.
    let mut name = intro.to_string();
    name.truncate(MAX_QPATH - 4);
    COM_DefaultExtension_str(&mut name, ".mp3");

    // Close the background track, but do NOT reset `s_rawend`, or the music
    // still in the ring gets cut off.
    S_StopBackgroundTrack_Actual(view.common, snd, track);

    let dmaSpeed = snd.dma.speed;
    let mut pMusicInfo = core::mem::take(&mut snd.tMusic_Info[track]);
    pMusicInfo.bIsMP3 = false;

    if intro.is_empty() {
        snd.tMusic_Info[track] = pMusicInfo;
        return false;
    }

    // If the file requested is not the one already loaded, ditch that one.
    if Q_stricmp(&name, &pMusicInfo.sLoadedDataName) != 0 {
        FreeMusic(&mut pMusicInfo);
    }

    if name.len() >= 4 && name[name.len() - 4..].eq_ignore_ascii_case(".mp3") {
        let ok = S_StartBackgroundTrack_Mp3(view, snd, &mut pMusicInfo, &name, qbDynamic, dmaSpeed);
        snd.tMusic_Info[track] = pMusicInfo;
        return ok;
    }

    // Not an MP3 file: open a wav and read the header.
    let mut handle: fileHandle_t = 0;
    FS_FOpenFileRead(view, &name, &mut handle, true);
    pMusicInfo.s_backgroundFile = handle;
    if pMusicInfo.s_backgroundFile == 0 {
        com_printf(
            view.common,
            &format!("^3WARNING: couldn't open music file {name}\n"),
        );
        snd.tMusic_Info[track] = pMusicInfo;
        return false;
    }

    // Skip the riff wav header.
    let mut dump = [0u8; 12];
    FS_Read(view.common, dump.as_mut_ptr() as *mut (), 12, handle);

    if S_FindWavChunk(view.common, handle, "fmt ") == 0 {
        com_printf(view.common, &format!("^3WARNING: No fmt chunk in {name}\n"));
        FS_FCloseFile(view.common, handle);
        pMusicInfo.s_backgroundFile = 0;
        snd.tMusic_Info[track] = pMusicInfo;
        return false;
    }

    // Save name for soundinfo.
    pMusicInfo.s_backgroundInfo.format = FGetLittleShort(view.common, handle);
    pMusicInfo.s_backgroundInfo.channels = FGetLittleShort(view.common, handle);
    pMusicInfo.s_backgroundInfo.rate = FGetLittleLong(view.common, handle);
    FGetLittleLong(view.common, handle);
    FGetLittleShort(view.common, handle);
    pMusicInfo.s_backgroundInfo.width = FGetLittleShort(view.common, handle) / 8;

    if pMusicInfo.s_backgroundInfo.format != WAV_FORMAT_PCM {
        FS_FCloseFile(view.common, handle);
        pMusicInfo.s_backgroundFile = 0;
        com_printf(
            view.common,
            &format!("^3WARNING: Not a microsoft PCM format wav: {name}\n"),
        );
        snd.tMusic_Info[track] = pMusicInfo;
        return false;
    }

    if pMusicInfo.s_backgroundInfo.channels != 2 || pMusicInfo.s_backgroundInfo.rate != 22050 {
        com_printf(
            view.common,
            &format!("^3WARNING: music file {name} is not 22k stereo\n"),
        );
    }

    let len = S_FindWavChunk(view.common, handle, "data");
    if len == 0 {
        FS_FCloseFile(view.common, handle);
        pMusicInfo.s_backgroundFile = 0;
        com_printf(view.common, &format!("^3WARNING: No data chunk in {name}\n"));
        snd.tMusic_Info[track] = pMusicInfo;
        return false;
    }

    pMusicInfo.s_backgroundInfo.samples =
        len / (pMusicInfo.s_backgroundInfo.width * pMusicInfo.s_backgroundInfo.channels).max(1);
    pMusicInfo.s_backgroundSamples = pMusicInfo.s_backgroundInfo.samples;

    // Start the background streaming.
    Sys_BeginStreamedFile(handle, 0x10000);

    snd.tMusic_Info[track] = pMusicInfo;
    true
}

/// The MP3 half of `S_StartBackgroundTrack_Actual`.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4088-4205`
fn S_StartBackgroundTrack_Mp3(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    pMusicInfo: &mut MusicInfo_t,
    name: &str,
    qbDynamic: bool,
    dmaSpeed: c_int,
) -> bool {
    if !pMusicInfo.pLoadedData.is_empty() {
        pMusicInfo.s_backgroundFile = -1;
    } else {
        let mut handle: fileHandle_t = 0;
        pMusicInfo.iLoadedDataLen = FS_FOpenFileRead(view, name, &mut handle, true);
        pMusicInfo.s_backgroundFile = handle;
    }

    if pMusicInfo.s_backgroundFile == 0 {
        com_printf(
            view.common,
            &format!("^1Couldn't open music file {name}\n"),
        );
        return false;
    }

    MP3MusicStream_Reset(pMusicInfo);

    // Raven: fairly arbitrary. The decoder may scan up to halfway through this
    // block looking for a floating header, so it must not be too small.
    let mut iInitialMP3ReadSize: c_int = 8192;

    if qbDynamic {
        if pMusicInfo.pLoadedData.is_empty() {
            let want = pMusicInfo.iLoadedDataLen.max(0) as usize;
            let mut data = vec![0u8; want];
            S_ClearSoundBuffer(snd);
            let handle = pMusicInfo.s_backgroundFile;
            FS_Read(
                view.common,
                data.as_mut_ptr() as *mut (),
                pMusicInfo.iLoadedDataLen,
                handle,
            );
            pMusicInfo.pLoadedData = data;
            pMusicInfo.sLoadedDataName = name.to_string();
            pMusicInfo.sLoadedDataName.truncate(MAX_QPATH - 1);
        }
        iInitialMP3ReadSize = pMusicInfo.iLoadedDataLen;
    } else {
        MP3MusicStream_ReadFromDisk(view.common, pMusicInfo, 0, iInitialMP3ReadSize);
    }

    // The decoder reads the loaded block for a dynamic track, and the disk
    // window for a streamed one.
    let segment: Vec<u8> = if qbDynamic {
        pMusicInfo.pLoadedData.clone()
    } else {
        pMusicInfo.byMP3MusicStream_DiskBuffer.clone()
    };

    if !MP3_IsValid(view.common, name, &segment, iInitialMP3ReadSize, true) {
        // `MP3_IsValid` has already printed the reason.
        if pMusicInfo.s_backgroundFile != -1 {
            FS_FCloseFile(view.common, pMusicInfo.s_backgroundFile);
        }
        pMusicInfo.s_backgroundFile = 0;
        return false;
    }

    // Init the stream struct.
    pMusicInfo.streamMP3_Bgrnd = MP3StreamState::default();
    let psError = pMusicInfo.streamMP3_Bgrnd.DecodeInit(
        &segment,
        pMusicInfo.iLoadedDataLen,
        dmaSpeed,
        true,
    );

    let Some(error) = psError else {
        // Init the sfx struct and set up the few fields actually needed.
        pMusicInfo.sfxMP3_Bgrnd = sfx_t::default();
        // Max possible positive int: music finishes when the decoder stops.
        pMusicInfo.sfxMP3_Bgrnd.iSoundLengthInSamples = 0x7fff_ffff;
        pMusicInfo.sfxMP3_Bgrnd.sSoundName = name.to_string();
        pMusicInfo.sfxMP3_Bgrnd.sSoundName.truncate(MAX_QPATH - 1);
        pMusicInfo.sfxMP3_Bgrnd.pMP3StreamHeader =
            Some(Box::new(pMusicInfo.streamMP3_Bgrnd.clone()));

        if qbDynamic {
            MP3Stream_InitPlayingTimeFields(
                view.common,
                &mut pMusicInfo.streamMP3_Bgrnd,
                name,
                &segment,
                pMusicInfo.iLoadedDataLen,
                true,
            );
        }

        // Not actually used as a format, but it cannot collide with a real one.
        pMusicInfo.s_backgroundInfo.format = WAV_FORMAT_MP3;
        // Always two channels for our MP3s when used for music, one for effects.
        pMusicInfo.s_backgroundInfo.channels = 2;
        pMusicInfo.s_backgroundInfo.rate = dmaSpeed;
        pMusicInfo.s_backgroundInfo.width = 2;
        pMusicInfo.s_backgroundInfo.samples = pMusicInfo.sfxMP3_Bgrnd.iSoundLengthInSamples;
        pMusicInfo.s_backgroundSamples = pMusicInfo.sfxMP3_Bgrnd.iSoundLengthInSamples;

        pMusicInfo.chMP3_Bgrnd = ChannelMp3State::default();
        pMusicInfo.chMP3_Bgrnd.MP3StreamHeader = pMusicInfo.streamMP3_Bgrnd.clone();

        if qbDynamic && pMusicInfo.s_backgroundFile != -1 {
            FS_FCloseFile(view.common, pMusicInfo.s_backgroundFile);
            // The special MP3 value for "valid, but not a real file".
            pMusicInfo.s_backgroundFile = -1;
        }

        pMusicInfo.bIsMP3 = true;
        return true;
    };

    com_printf(
        view.common,
        &format!("^1Error streaming file {name}: {error}\n"),
    );
    if pMusicInfo.s_backgroundFile != -1 {
        FS_FCloseFile(view.common, pMusicInfo.s_backgroundFile);
    }
    pMusicInfo.s_backgroundFile = 0;
    false
}

/// Raven `S_SwitchDynamicTracks` — move the old track into the fader and bring
/// the new one up.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4268-4299`
fn S_SwitchDynamicTracks(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    eOldState: MusicState_e,
    eNewState: MusicState_e,
    bNewTrackStartsFullVolume: bool,
) {
    let now = Com_Milliseconds(view);

    // Copy the old track into the fader. `bActive` and `bExists` come with it.
    snd.tMusic_Info[MusicState_e::eBGRNDTRACK_FADE as usize] =
        snd.tMusic_Info[eOldState as usize].clone();
    let fade = MusicState_e::eBGRNDTRACK_FADE as usize;
    snd.tMusic_Info[fade].iXFadeVolumeSeekTime = now;
    snd.tMusic_Info[fade].iXFadeVolumeSeekTo = 0;

    // ... and deactivate.
    snd.tMusic_Info[eOldState as usize].bActive = false;

    // Set the new track to either full volume or a fade up.
    let new = eNewState as usize;
    snd.tMusic_Info[new].bActive = true;
    snd.tMusic_Info[new].iXFadeVolumeSeekTime = now;
    snd.tMusic_Info[new].iXFadeVolumeSeekTo = 255;
    snd.tMusic_Info[new].iXFadeVolume = if bNewTrackStartsFullVolume { 255 } else { 0 };

    snd.eMusic_StateActual = eNewState;

    if view.common.cvar(snd.s_debugdynamic).integer != 0 {
        let psNewStateString = Music_BaseStateToString(eNewState, true).unwrap_or("<unknown>");
        com_printf(
            view.common,
            &format!("^6S_SwitchDynamicTracks( \"{psNewStateString}\" )\n"),
        );
    }
}

/// Raven `S_SetDynamicMusicState` — record the state the game asked for.
///
/// The change is applied later, because a transition may not be interruptible.
/// Source: `oracle/codemp/client/snd_dma.cpp:4306-4320`
fn S_SetDynamicMusicState(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    eNewState: MusicState_e,
) {
    if snd.eMusic_StateRequest != eNewState {
        snd.eMusic_StateRequest = eNewState;

        if view.common.cvar(snd.s_debugdynamic).integer != 0 {
            let psNewStateString = Music_BaseStateToString(eNewState, true).unwrap_or("<unknown>");
            com_printf(
                view.common,
                &format!("^6S_SetDynamicMusicState( Request: \"{psNewStateString}\" )\n"),
            );
        }
    }
}

/// Raven `S_HandleDynamicMusicStateChange` — apply a pending state change where
/// the playing track allows it.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4323-4472`
fn S_HandleDynamicMusicStateChange(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if snd.eMusic_StateRequest == snd.eMusic_StateActual {
        return;
    }
    if !Music_StateCanBeInterrupted(snd.eMusic_StateActual, snd.eMusic_StateRequest) {
        return;
    }

    let dmaSpeed = snd.dma.speed;
    let actual = snd.eMusic_StateActual;
    let request = snd.eMusic_StateRequest;
    let fPlayingTimeElapsed = playing_time_elapsed(snd, actual, dmaSpeed);

    match request {
        // From action or silence.
        MusicState_e::eBGRNDTRACK_EXPLORE => match actual {
            MusicState_e::eBGRNDTRACK_ACTION => {
                // Find the transition track to play, the entry point for explore
                // when it gets there, and whether this is a permitted exit at all.
                if let Some((eTransition, fNewTrackEntryTime)) = Music_AllowedToTransition(
                    view,
                    &snd.music,
                    fPlayingTimeElapsed,
                    MusicState_e::eBGRNDTRACK_ACTION,
                ) {
                    S_SwitchDynamicTracks(view, snd, actual, eTransition, false);

                    snd.tMusic_Info[eTransition as usize].Rewind();
                    snd.tMusic_Info[eTransition as usize].bTrackSwitchPending = true;
                    snd.tMusic_Info[eTransition as usize].eTS_NewState = request;
                    snd.tMusic_Info[eTransition as usize].fTS_NewTime = fNewTrackEntryTime;
                }
            }
            MusicState_e::eBGRNDTRACK_SILENCE => {
                S_SwitchDynamicTracks(view, snd, actual, request, false);
                snd.tMusic_Info[request as usize].Rewind();
            }
            // Raven asserts here: a state he did not expect to transition from.
            _ => {
                S_SwitchDynamicTracks(view, snd, actual, MusicState_e::eBGRNDTRACK_SILENCE, false);
            }
        },

        // From explore or action.
        MusicState_e::eBGRNDTRACK_SILENCE => match actual {
            MusicState_e::eBGRNDTRACK_ACTION | MusicState_e::eBGRNDTRACK_EXPLORE => {
                if let Some((eTransition, _)) =
                    Music_AllowedToTransition(view, &snd.music, fPlayingTimeElapsed, actual)
                {
                    S_SwitchDynamicTracks(view, snd, actual, eTransition, false);

                    snd.tMusic_Info[eTransition as usize].Rewind();
                    snd.tMusic_Info[eTransition as usize].bTrackSwitchPending = true;
                    snd.tMusic_Info[eTransition as usize].eTS_NewState = request;
                    // The entry time is irrelevant when switching to silence.
                    snd.tMusic_Info[eTransition as usize].fTS_NewTime = 0.0;
                }
            }
            // Raven asserts on an unhandled type and falls through to the boss
            // case, which just switches to silence.
            _ => {
                S_SwitchDynamicTracks(view, snd, actual, MusicState_e::eBGRNDTRACK_SILENCE, false);
            }
        },

        // Anything to action.
        MusicState_e::eBGRNDTRACK_ACTION => match actual {
            MusicState_e::eBGRNDTRACK_SILENCE => {
                S_SwitchDynamicTracks(view, snd, actual, request, false);
                snd.tMusic_Info[request as usize].Rewind();
            }
            _ => {
                S_SwitchDynamicTracks(view, snd, actual, request, true);
                let fEntryTime =
                    Music_GetRandomEntryTime(&mut snd.music, &mut view.common.qrand, request);
                MusicInfo_SeekTo(&mut snd.tMusic_Info[request as usize], dmaSpeed, fEntryTime);
            }
        },

        // Boss and death are each entered once, at the start, and cannot exit,
        // so neither needs a rewind or a fast forward.
        MusicState_e::eBGRNDTRACK_BOSS => {
            S_SwitchDynamicTracks(view, snd, actual, request, false);
        }
        MusicState_e::eBGRNDTRACK_DEATH => {
            S_SwitchDynamicTracks(view, snd, actual, request, true);
        }

        // Raven asserts on an unknown request and ignores it.
        _ => {}
    }
}

/// The play position of one dynamic track, in seconds.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4344,4400`
fn playing_time_elapsed(snd: &SoundSystem, state: MusicState_e, dmaSpeed: c_int) -> f32 {
    let stream = &snd.tMusic_Info[state as usize].chMP3_Bgrnd.MP3StreamHeader;
    MP3Stream_GetPlayingTimeInSeconds(stream)
        - MP3Stream_GetRemainingTimeInSeconds(stream, dmaSpeed)
}

/// Raven `S_RestartMusic` — replay the track the last `S_StartBackgroundTrack`
/// asked for, and put the dynamic state back.
///
/// Raven no longer tests the two saved names, because they are blank for a
/// dynamic-music level, but he still uses them.
/// Source: `oracle/codemp/client/snd_dma.cpp:4479-4490`
pub fn S_RestartMusic(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if snd.s_soundStarted != 0 && !snd.s_soundMuted {
        let ePrevState = snd.eMusic_StateRequest;
        let intro = snd.gsIntroMusic.clone();
        let loop_name = snd.gsLoopMusic.clone();
        // The default music start sets the state to EXPLORE.
        S_StartBackgroundTrack(view, snd, &intro, &loop_name, false);
        // Restore the previous state.
        S_SetDynamicMusicState(view, snd, ePrevState);
    }
}

/// Raven `S_StartBackgroundTrack` — start the music one level asked for.
///
/// A literal file name streams as one track. A name with no file behind it is a
/// dynamic-music label, so every data track loads and the explore piece starts.
/// Raven notes that some of the file-check logic only works for MP3s.
/// Source: `oracle/codemp/client/snd_dma.cpp:4499-4622`
pub fn S_StartBackgroundTrack(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    intro: &str,
    loop_name: &str,
    bCalledByCGameStart: bool,
) {
    snd.bMusic_IsDynamic = false;

    if snd.s_soundStarted == 0 {
        // We have no sound, so don't even bother trying.
        return;
    }

    let loop_name = if loop_name.is_empty() { intro } else { loop_name };

    snd.gsIntroMusic = intro.to_string();
    snd.gsIntroMusic.truncate(MAX_QPATH - 1);
    snd.gsLoopMusic = loop_name.to_string();
    snd.gsLoopMusic.truncate(MAX_QPATH - 1);

    let mut sNameIntro = snd.gsIntroMusic.clone();
    let mut sNameLoop = snd.gsLoopMusic.clone();

    COM_DefaultExtension_str(&mut sNameIntro, ".mp3");
    COM_DefaultExtension_str(&mut sNameLoop, ".mp3");

    // Where dynamic music is not allowed, stream the explore piece instead of
    // playing dynamic. Raven names `intro` here, not the ".mp3" version.
    if view.common.cvar(snd.s_allowDynamicMusic).integer == 0
        && Music_DynamicDataAvailable(view, snd, intro)
    {
        let psMusicName =
            Music_GetFileNameForState(&snd.music, MusicState_e::eBGRNDTRACK_EXPLORE);
        if let Some(psMusicName) = psMusicName {
            if S_FileExists(view, &psMusicName) {
                sNameIntro = psMusicName.clone();
                sNameLoop = psMusicName;
            }
        }
    }

    // The 'intro' track always plays; the intro-to-loop switch happens in
    // `S_UpdateBackgroundTrack`. Raven's `strstr("/")` test avoids an extra
    // file check at runtime, because a literal music name always has a slash.
    if sNameIntro.contains('/') && S_FileExists(view, &sNameIntro) {
        let psLoopName = if S_FileExists(view, &sNameLoop) {
            sNameLoop.clone()
        } else {
            sNameIntro.clone()
        };
        Com_DPrintf(
            view.common,
            &format!(
                "S_StartBackgroundTrack: Found/using non-dynamic music track '{sNameIntro}' (loop: '{psLoopName}')\n"
            ),
        );
        let dynamic = snd.bMusic_IsDynamic;
        S_StartBackgroundTrack_Actual(
            view,
            snd,
            MusicState_e::eBGRNDTRACK_NONDYNAMIC as usize,
            dynamic,
            &sNameIntro,
            &psLoopName,
        );
    } else if Music_DynamicDataAvailable(view, snd, intro) {
        snd.sInfoOnly_CurrentDynamicMusicSet = Music_GetLevelSetName(&snd.music);
        snd.sInfoOnly_CurrentDynamicMusicSet.truncate(63);

        for i in 0..MusicState_e::eBGRNDTRACK_NONDYNAMIC as usize {
            let mut bOk = false;
            let state = music_state_from_index(i);
            let psMusicName = Music_GetFileNameForState(&snd.music, state);
            if let Some(psMusicName) = psMusicName {
                let loaded = Q_stricmp(&snd.tMusic_Info[i].sLoadedDataName, &psMusicName) == 0;
                if loaded || S_FileExists(view, &psMusicName) {
                    bOk = S_StartBackgroundTrack_Actual(
                        view,
                        snd,
                        i,
                        true,
                        &psMusicName,
                        loop_name,
                    );
                }
            }

            snd.tMusic_Info[i].bExists = bOk;

            if !snd.tMusic_Info[i].bExists {
                FreeMusic(&mut snd.tMusic_Info[i]);
            }
        }

        // Default all tracks to OFF first, and set the other vars.
        for i in 0..BGRNDTRACK_NUMBEROF {
            snd.tMusic_Info[i].bActive = false;
            snd.tMusic_Info[i].bTrackSwitchPending = false;
            snd.tMusic_Info[i].fSmoothedOutVolume = 0.25;
        }

        if snd.tMusic_Info[MusicState_e::eBGRNDTRACK_EXPLORE as usize].bExists
            && snd.tMusic_Info[MusicState_e::eBGRNDTRACK_ACTION as usize].bExists
        {
            Com_DPrintf(
                view.common,
                "S_StartBackgroundTrack: Found dynamic music tracks\n",
            );
            snd.bMusic_IsDynamic = true;

            // ... then start the default music state.
            snd.eMusic_StateActual = MusicState_e::eBGRNDTRACK_EXPLORE;
            snd.eMusic_StateRequest = MusicState_e::eBGRNDTRACK_EXPLORE;

            let now = Com_Milliseconds(view);
            let track = snd.eMusic_StateActual as usize;
            snd.tMusic_Info[track].bActive = true;
            snd.tMusic_Info[track].iXFadeVolumeSeekTime = now;
            snd.tMusic_Info[track].iXFadeVolumeSeekTo = 255;
            snd.tMusic_Info[track].iXFadeVolume = 0;
        } else {
            com_printf(
                view.common,
                "^1Dynamic music did not have both 'action' and 'explore' versions, inhibiting...\n",
            );
            S_StopBackgroundTrack(view.common, snd);
        }
    } else if !sNameIntro.starts_with('.') {
        // A blank name with ".mp3" attached prints nothing.
        com_printf(
            view.common,
            &format!(
                "^1Unable to find music \"{sNameIntro}\" as explicit track or dynamic music entry!\n"
            ),
        );
        S_StopBackgroundTrack(view.common, snd);
    }

    if bCalledByCGameStart {
        S_StopBackgroundTrack(view.common, snd);
    }
}

/// The `MusicState_e` one track index names.
///
/// Raven casts the loop counter, and the twelve data tracks are the first twelve
/// enumerators.
fn music_state_from_index(i: usize) -> MusicState_e {
    match i {
        0 => MusicState_e::eBGRNDTRACK_EXPLORE,
        1 => MusicState_e::eBGRNDTRACK_ACTION,
        2 => MusicState_e::eBGRNDTRACK_BOSS,
        3 => MusicState_e::eBGRNDTRACK_DEATH,
        4 => MusicState_e::eBGRNDTRACK_ACTIONTRANS0,
        5 => MusicState_e::eBGRNDTRACK_ACTIONTRANS1,
        6 => MusicState_e::eBGRNDTRACK_ACTIONTRANS2,
        7 => MusicState_e::eBGRNDTRACK_ACTIONTRANS3,
        8 => MusicState_e::eBGRNDTRACK_EXPLORETRANS0,
        9 => MusicState_e::eBGRNDTRACK_EXPLORETRANS1,
        10 => MusicState_e::eBGRNDTRACK_EXPLORETRANS2,
        11 => MusicState_e::eBGRNDTRACK_EXPLORETRANS3,
        12 => MusicState_e::eBGRNDTRACK_NONDYNAMIC,
        13 => MusicState_e::eBGRNDTRACK_SILENCE,
        _ => MusicState_e::eBGRNDTRACK_FADE,
    }
}

/// Raven `SIZEOF_RAW_BUFFER_FOR_MP3` and the `raw[30000]` stack block.
///
/// Raven: 30000 is far too big for the window decoder to handle in one request,
/// because of the time-travel issue that normal sfx buffer painting brings, so
/// the MP3 arm asks for 4096 instead.
/// Source: `oracle/codemp/client/snd_dma.cpp:4642,4675-4676`
const RAW_BUFFER_BYTES: usize = 30000;
const SIZEOF_RAW_BUFFER_FOR_MP3: usize = 4096;

/// Raven `S_UpdateBackgroundTrack_Actual` — feed the raw ring from one music track.
///
/// Returns true only where a streamed intro wants to hand over to a dynamic loop.
/// Source: `oracle/codemp/client/snd_dma.cpp:4638-4807`
fn S_UpdateBackgroundTrack_Actual(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    track: usize,
    bFirstOrOnlyMusicTrack: bool,
    fDefaultVolume: f32,
) -> bool {
    let mut raw = vec![0u8; RAW_BUFFER_BYTES];
    let mut fMasterVol = fDefaultVolume;

    if snd.bMusic_IsDynamic {
        // Step the cross-fade volume.
        if snd.tMusic_Info[track].iXFadeVolume != snd.tMusic_Info[track].iXFadeVolumeSeekTo {
            let iFadeMillisecondsElapsed =
                Com_Milliseconds(view) - snd.tMusic_Info[track].iXFadeVolumeSeekTime;

            if iFadeMillisecondsElapsed as f32 > fDYNAMIC_XFADE_SECONDS * 1000.0 {
                snd.tMusic_Info[track].iXFadeVolume = snd.tMusic_Info[track].iXFadeVolumeSeekTo;
            } else {
                snd.tMusic_Info[track].iXFadeVolume = (255.0
                    * (iFadeMillisecondsElapsed as f32 / (fDYNAMIC_XFADE_SECONDS * 1000.0)))
                    as c_int;
                if snd.tMusic_Info[track].iXFadeVolumeSeekTo == 0 {
                    // bleurgh
                    snd.tMusic_Info[track].iXFadeVolume = 255 - snd.tMusic_Info[track].iXFadeVolume;
                }
            }
        }
        fMasterVol *= snd.tMusic_Info[track].iXFadeVolume as f32 / 255.0;
    }

    if snd.tMusic_Info[track].s_backgroundFile == 0 {
        return false;
    }

    snd.tMusic_Info[track].fSmoothedOutVolume =
        (snd.tMusic_Info[track].fSmoothedOutVolume + fMasterVol) / 2.0;

    // don't bother playing anything if musicvolume is 0
    if snd.tMusic_Info[track].fSmoothedOutVolume <= 0.0 {
        return false;
    }

    let RAWSIZE = if snd.tMusic_Info[track].bIsMP3 {
        SIZEOF_RAW_BUFFER_FOR_MP3
    } else {
        RAW_BUFFER_BYTES
    } as c_int;

    // see how many samples should be copied into the raw buffer
    if snd.s_rawend < snd.s_soundtime {
        snd.s_rawend = snd.s_soundtime;
    }

    while snd.s_rawend < snd.s_soundtime + MAX_RAW_SAMPLES as c_int {
        let bufferSamples = MAX_RAW_SAMPLES as c_int - (snd.s_rawend - snd.s_soundtime);

        // decide how much data needs to be read from the file
        let mut fileSamples =
            bufferSamples * snd.tMusic_Info[track].s_backgroundInfo.rate / snd.dma.speed;

        // don't try and read past the end of the file
        if fileSamples > snd.tMusic_Info[track].s_backgroundSamples {
            fileSamples = snd.tMusic_Info[track].s_backgroundSamples;
        }

        // our max buffer size
        let frameBytes = (snd.tMusic_Info[track].s_backgroundInfo.width
            * snd.tMusic_Info[track].s_backgroundInfo.channels)
            .max(1);
        let mut fileBytes = fileSamples * frameBytes;
        if fileBytes > RAWSIZE {
            fileBytes = RAWSIZE;
            fileSamples = fileBytes / frameBytes;
        }

        let mut qbForceFinish = false;
        if snd.tMusic_Info[track].bIsMP3 {
            // This one IS relevant.
            let iStartingSampleNum = snd.tMusic_Info[track].sfxMP3_Bgrnd.iSoundLengthInSamples
                - snd.tMusic_Info[track].s_backgroundSamples;

            let mut pcm = vec![0i16; (fileBytes / 2).max(0) as usize];
            let mut pMusicInfo = core::mem::take(&mut snd.tMusic_Info[track]);

            let stillGoing = if pMusicInfo.s_backgroundFile == -1 {
                // in-mem...
                let source = core::mem::take(&mut pMusicInfo.pLoadedData);
                let going = MP3Stream_GetSamples(
                    &mut pMusicInfo.chMP3_Bgrnd,
                    &source,
                    0,
                    iStartingSampleNum,
                    fileBytes / 2,
                    &mut pcm,
                    true,
                );
                pMusicInfo.pLoadedData = source;
                going
            } else {
                // Streaming an MP3 off disk. The `fileBytes` request size is not
                // that relevant for an MP3, because the code here cannot know
                // how much source the decoder needs.
                let readIndex = pMusicInfo.chMP3_Bgrnd.MP3StreamHeader.iSourceReadIndex;
                MP3MusicStream_ReadFromDisk(view.common, &mut pMusicInfo, readIndex, fileBytes);
                let origin = pMusicInfo.iMP3MusicStream_DiskWindowPos;
                let source = core::mem::take(&mut pMusicInfo.byMP3MusicStream_DiskBuffer);
                let going = MP3Stream_GetSamples(
                    &mut pMusicInfo.chMP3_Bgrnd,
                    &source,
                    origin,
                    iStartingSampleNum,
                    fileBytes / 2,
                    &mut pcm,
                    true,
                );
                pMusicInfo.byMP3MusicStream_DiskBuffer = source;
                going
            };
            snd.tMusic_Info[track] = pMusicInfo;
            qbForceFinish = !stillGoing;

            for (i, sample) in pcm.iter().enumerate() {
                raw[i * 2..i * 2 + 2].copy_from_slice(&sample.to_le_bytes());
            }
        } else {
            // Streaming a WAV off disk.
            let handle = snd.tMusic_Info[track].s_backgroundFile;
            let r = Sys_StreamedRead(
                view.common,
                raw.as_mut_ptr() as *mut (),
                1,
                fileBytes,
                handle,
            );
            if r != fileBytes {
                com_printf(view.common, "^1StreamedRead failure on music track\n");
                S_StopBackgroundTrack(view.common, snd);
                return false;
            }

            // Raven byte-swaps here on a big-endian host. `S_ByteSwapRawSamples`
            // returns at once on every target this tree builds for.
        }

        // add to raw buffer
        let (rate, width, channels) = (
            snd.tMusic_Info[track].s_backgroundInfo.rate,
            snd.tMusic_Info[track].s_backgroundInfo.width,
            snd.tMusic_Info[track].s_backgroundInfo.channels,
        );
        let volume = snd.tMusic_Info[track].fSmoothedOutVolume;
        S_RawSamples(
            view.common,
            snd,
            fileSamples,
            rate,
            width,
            channels,
            &raw,
            volume,
            bFirstOrOnlyMusicTrack,
        );

        snd.tMusic_Info[track].s_backgroundSamples -= fileSamples;
        if snd.tMusic_Info[track].s_backgroundSamples == 0 || qbForceFinish {
            // Loop the music, or play the next piece if we were on the intro.
            // Dynamic music can only be used for loop music, so it needs the
            // special call instead.
            if snd.bMusic_IsDynamic {
                snd.tMusic_Info[track].Rewind();
            } else {
                // For non-dynamic music, check whether `sMusic_BackgroundLoop`
                // is a real file or a dynamic-music specifier, which cannot
                // literally exist. Raven sizes the test name at `MAX_QPATH * 2`
                // so `COM_DefaultExtension` never runs out of room on this
                // "soft" test.
                let mut sTestName = snd.sMusic_BackgroundLoop.clone();
                COM_DefaultExtension_str(&mut sTestName, ".mp3");

                if S_FileExists(view, &sTestName) {
                    let loop_name = snd.sMusic_BackgroundLoop.clone();
                    S_StartBackgroundTrack_Actual(
                        view, snd, track, false, &loop_name, &loop_name,
                    );
                } else {
                    // The proposed file does not exist, but this may be a
                    // dynamic track we want to loop, so exit with the flag.
                    return true;
                }
            }
            if snd.tMusic_Info[track].s_backgroundFile == 0 {
                return false; // loop failed to restart
            }
        }
    }

    false
}

/// Raven `S_Music_GetRequestedState` — the dynamic-music state the game asked
/// for through a config string.
///
/// Raven's MP body reads no config string and answers NULL, with his own
/// "rwwFIXMEFIXME: Maybe use the above for something in MP?".
/// Source: `oracle/codemp/client/snd_dma.cpp:4812-4826`
fn S_Music_GetRequestedState() -> Option<&'static str> {
    None
}

/// Raven `S_CheckDynamicMusicState` — apply any requested state change, then run
/// the transition handling.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4834-4891`
fn S_CheckDynamicMusicState(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if let Some(psCommand) = S_Music_GetRequestedState() {
        let eNewState = if psCommand.starts_with("silence") {
            MusicState_e::eBGRNDTRACK_SILENCE
        } else if psCommand.starts_with("action") {
            MusicState_e::eBGRNDTRACK_ACTION
        } else if psCommand.starts_with("boss") {
            // Boss music is optional and may not be defined; leave the current
            // track playing where it is missing.
            if snd.tMusic_Info[MusicState_e::eBGRNDTRACK_BOSS as usize].bExists {
                MusicState_e::eBGRNDTRACK_BOSS
            } else {
                snd.eMusic_StateActual
            }
        } else if psCommand.starts_with("death") {
            // Death music is optional too, and the current track is typically
            // boss or action.
            if snd.tMusic_Info[MusicState_e::eBGRNDTRACK_DEATH as usize].bExists {
                MusicState_e::eBGRNDTRACK_DEATH
            } else {
                snd.eMusic_StateActual
            }
        } else {
            // Seems a reasonable default.
            MusicState_e::eBGRNDTRACK_EXPLORE
        };

        S_SetDynamicMusicState(view, snd, eNewState);
    }

    S_HandleDynamicMusicStateChange(view, snd);
}

/// Raven `S_UpdateBackgroundTrack` — the per-frame music step.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4893-5009`
fn S_UpdateBackgroundTrack(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if snd.bMusic_IsDynamic {
        if view.common.cvar(snd.s_debugdynamic).integer == 2 {
            DynamicMusicInfoPrint(view, snd);
        }

        S_CheckDynamicMusicState(view, snd);

        let fade = MusicState_e::eBGRNDTRACK_FADE as usize;
        if snd.eMusic_StateActual == MusicState_e::eBGRNDTRACK_SILENCE {
            // Special case: the foreground music is off but the fader is still
            // running out the previous track.
            if snd.tMusic_Info[fade].bActive {
                let volume = view.common.cvar(snd.s_musicVolume).value;
                S_UpdateBackgroundTrack_Actual(view, snd, fade, true, volume);
                if snd.tMusic_Info[fade].iXFadeVolume == 0 {
                    snd.tMusic_Info[fade].bActive = false;
                }
            }
            return;
        }

        let current = if snd.eMusic_StateActual == MusicState_e::eBGRNDTRACK_FADE {
            MusicState_e::eBGRNDTRACK_EXPLORE as usize
        } else {
            snd.eMusic_StateActual as usize
        };

        if snd.tMusic_Info[current].s_backgroundFile != -1 {
            return;
        }

        let iRawEnd = snd.s_rawend;
        let volume = view.common.cvar(snd.s_musicVolume).value;
        S_UpdateBackgroundTrack_Actual(view, snd, current, true, volume);

        if snd.tMusic_Info[fade].bActive {
            snd.s_rawend = iRawEnd;
            // The inactive check is internal to the call.
            S_UpdateBackgroundTrack_Actual(view, snd, fade, false, volume);

            // Only do this for the fader.
            if snd.tMusic_Info[fade].iXFadeVolume == 0 {
                snd.tMusic_Info[fade].bActive = false;
            }
        }

        let dmaSpeed = snd.dma.speed;
        let fRemainingTimeInSeconds = MP3Stream_GetRemainingTimeInSeconds(
            &snd.tMusic_Info[current].chMP3_Bgrnd.MP3StreamHeader,
            dmaSpeed,
        );

        if fRemainingTimeInSeconds >= fDYNAMIC_XFADE_SECONDS * 2.0 {
            return;
        }

        // Now either loop the current track, switch if a transition is
        // finishing, or stop if a death piece has finished.
        if snd.tMusic_Info[current].bTrackSwitchPending {
            snd.tMusic_Info[current].bTrackSwitchPending = false; // ack
            let eTS_NewState = snd.tMusic_Info[current].eTS_NewState;
            let fTS_NewTime = snd.tMusic_Info[current].fTS_NewTime;
            let actual = snd.eMusic_StateActual;
            S_SwitchDynamicTracks(view, snd, actual, eTS_NewState, false);
            // Don't do this if switching to silence.
            if snd.tMusic_Info[eTS_NewState as usize].bExists {
                MusicInfo_SeekTo(
                    &mut snd.tMusic_Info[eTS_NewState as usize],
                    dmaSpeed,
                    fTS_NewTime,
                );
            }
        } else {
            // Normal looping: rewind the current track, set its volume to 0 and
            // fade up to full, while the fader copy of the end section fades
            // down. A death track stays quiet instead.
            snd.tMusic_Info[fade] = snd.tMusic_Info[current].clone();
            let now = Com_Milliseconds(view);
            snd.tMusic_Info[fade].iXFadeVolumeSeekTime = now;
            snd.tMusic_Info[fade].iXFadeVolumeSeekTo = 0;

            snd.tMusic_Info[current].Rewind();
            snd.tMusic_Info[current].iXFadeVolumeSeekTime = now;
            snd.tMusic_Info[current].iXFadeVolumeSeekTo =
                if snd.eMusic_StateActual == MusicState_e::eBGRNDTRACK_DEATH {
                    0
                } else {
                    255
                };
            snd.tMusic_Info[current].iXFadeVolume = 0;
        }
        return;
    }

    // standard / non-dynamic one-track music...
    // MP's `S_Music_GetRequestedState` returns NULL, so the silence check never fires.
    let bShouldBeSilent = S_Music_GetRequestedState().is_some_and(|c| c.eq_ignore_ascii_case("silence"));
    let fDesiredVolume = if bShouldBeSilent {
        0.0
    } else {
        view.common.cvar(snd.s_musicVolume).value
    };

    // internal to this code is a volume-smoother...
    let bNewTrackDesired = S_UpdateBackgroundTrack_Actual(
        view,
        snd,
        MusicState_e::eBGRNDTRACK_NONDYNAMIC as usize,
        true,
        fDesiredVolume,
    );

    if bNewTrackDesired {
        let loop_name = snd.sMusic_BackgroundLoop.clone();
        S_StartBackgroundTrack(view, snd, &loop_name, &loop_name, false);
    }
}

// ===========================================================================
// Init and shutdown
// ===========================================================================

/// Raven `DynamicMusicInfoPrint` — the one-line dynamic-music state report.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:335-358`
fn DynamicMusicInfoPrint(view: &mut EngineHostView, snd: &SoundSystem) {
    if !snd.bMusic_IsDynamic {
        com_printf(view.common, "( Dynamic music OFF )\n");
        return;
    }

    // Raven calls this horribly lazy.
    let psRequestMusicState =
        Music_BaseStateToString(snd.eMusic_StateRequest, false).unwrap_or("<unknown>");
    let psActualMusicState =
        Music_BaseStateToString(snd.eMusic_StateActual, true).unwrap_or("<unknown>");

    let request = snd.eMusic_StateRequest as c_int;
    let actual = snd.eMusic_StateActual as c_int;
    com_printf(
        view.common,
        &format!(
            "( Dynamic music ON, request state: '{psRequestMusicState}'({request}), actual: '{psActualMusicState}' ({actual}) )\n"
        ),
    );
}

/// Raven `S_SoundInfo_f` — the `soundinfo` command.
///
/// Raven prints the ring address on the `dma buffer` line. The port prints the
/// ring size instead, because no engine dump may carry a host address.
/// Source: `oracle/codemp/client/snd_dma.cpp:360-410`
pub fn S_SoundInfo_f(view: &mut EngineHostView, snd: &SoundSystem) {
    com_printf(view.common, "----- Sound Info -----\n");

    if snd.s_soundStarted == 0 {
        com_printf(view.common, "sound system not started\n");
    } else {
        if snd.s_soundMuted {
            com_printf(view.common, "sound system is muted\n");
        }

        let stereo = snd.dma.channels - 1;
        com_printf(view.common, &format!("{stereo:5} stereo\n"));
        let samples = snd.dma.samples;
        com_printf(view.common, &format!("{samples:5} samples\n"));
        let samplebits = snd.dma.samplebits;
        com_printf(view.common, &format!("{samplebits:5} samplebits\n"));
        let chunk = snd.dma.submission_chunk;
        com_printf(view.common, &format!("{chunk:5} submission_chunk\n"));
        let speed = snd.dma.speed;
        com_printf(view.common, &format!("{speed:5} speed\n"));
        let bytes = snd.dma.buffer.len();
        com_printf(view.common, &format!("0x{bytes:x} dma buffer bytes\n"));

        if snd.bMusic_IsDynamic {
            DynamicMusicInfoPrint(view, snd);
            let set = snd.sInfoOnly_CurrentDynamicMusicSet.clone();
            com_printf(
                view.common,
                &format!("( Dynamic music set name: \"{set}\" )\n"),
            );
        } else {
            if view.common.cvar(snd.s_allowDynamicMusic).integer == 0 {
                com_printf(
                    view.common,
                    "( Dynamic music inhibited (s_allowDynamicMusic == 0) )\n",
                );
            }
            if snd.tMusic_Info[MusicState_e::eBGRNDTRACK_NONDYNAMIC as usize].s_backgroundFile != 0
            {
                let name = snd.sMusic_BackgroundLoop.clone();
                com_printf(view.common, &format!("Background file: {name}\n"));
            } else {
                com_printf(view.common, "No background file.\n");
            }
        }
    }
    S_DisplayFreeMemory(view, snd);
    com_printf(view.common, "----------------------\n");
}

/// Raven `S_Init` — register the cvars and the commands, open the device, and
/// start the ambient-sound system.
///
/// DEC-57.4 drops the OpenAL arm, so the software mixer always runs.
/// Source: `oracle/codemp/client/snd_dma.cpp:419-624`
pub fn S_Init(view: &mut EngineHostView, snd: &mut SoundSystem) {
    com_printf(view.common, "\n------- sound initialization -------\n");

    snd.s_volume = Some(Cvar_Get(view, "s_volume", "0.5", CVAR_ARCHIVE));
    snd.s_volumeVoice = Some(Cvar_Get(view, "s_volumeVoice", "1.0", CVAR_ARCHIVE));
    snd.s_musicVolume = Some(Cvar_Get(view, "s_musicvolume", "0.25", CVAR_ARCHIVE));
    snd.s_separation = Some(Cvar_Get(view, "s_separation", "0.5", CVAR_ARCHIVE));
    snd.s_khz = Some(Cvar_Get(view, "s_khz", "22", CVAR_ARCHIVE | CVAR_LATCH));
    snd.s_allowDynamicMusic = Some(Cvar_Get(view, "s_allowDynamicMusic", "1", CVAR_ARCHIVE));
    snd.s_mixahead = Some(Cvar_Get(view, "s_mixahead", "0.2", CVAR_ARCHIVE));

    snd.s_mixPreStep = Some(Cvar_Get(view, "s_mixPreStep", "0.05", CVAR_ARCHIVE));
    snd.s_show = Some(Cvar_Get(view, "s_show", "0", CVAR_CHEAT));
    snd.s_testsound = Some(Cvar_Get(view, "s_testsound", "0", CVAR_CHEAT));
    snd.s_debugdynamic = Some(Cvar_Get(view, "s_debugdynamic", "0", CVAR_CHEAT));
    snd.s_lip_threshold_1 = Some(Cvar_Get(view, "s_threshold1", "0.5", 0));
    snd.s_lip_threshold_2 = Some(Cvar_Get(view, "s_threshold2", "4.0", 0));
    snd.s_lip_threshold_3 = Some(Cvar_Get(view, "s_threshold3", "7.0", 0));
    snd.s_lip_threshold_4 = Some(Cvar_Get(view, "s_threshold4", "8.0", 0));

    snd.s_language = Some(Cvar_Get(
        view,
        "s_language",
        "english",
        CVAR_ARCHIVE | CVAR_NORESTART,
    ));

    // Raven `MP3_InitCvars`, which registers the one MP3 cvar (gh#25 uses it).
    // The default is `sizeof(MP3STREAM) + FUZZY_AMOUNT`, so it tracks the struct.
    // Source: `oracle/codemp/client/snd_mp3.cpp:226-229`
    let overhead = core::mem::size_of::<MP3STREAM>() + FUZZY_AMOUNT;
    Cvar_Get(view, "s_mp3overhead", &format!("{overhead}"), CVAR_ARCHIVE);

    // Raven caches `sys_cpuid` for the MMX blast arm, which the port does not carry.
    Cvar_Get(view, "sys_cpuid", "", 0);

    let cv = Cvar_Get(view, "s_initsound", "1", CVAR_ROM);
    if view.common.cvar(cv).integer == 0 {
        // needed in case you set s_initsound to 0 midgame then snd_restart (div0 err otherwise later)
        snd.s_soundStarted = 0;
        com_printf(view.common, "not initializing.\n");
        com_printf(view.common, "------------------------------------\n");
        return;
    }

    Cmd_AddCommand(view, "play", Some(S_Play_f));
    Cmd_AddCommand(view, "music", Some(S_Music_f));
    Cmd_AddCommand(view, "soundlist", Some(S_SoundList_f));
    Cmd_AddCommand(view, "soundinfo", Some(S_SoundInfo_f_cmd));
    Cmd_AddCommand(view, "soundstop", Some(S_StopAllSounds_f));
    Cmd_AddCommand(view, "mp3_calcvols", Some(S_MP3_CalcVols_f));
    Cmd_AddCommand(view, "s_dynamic", Some(S_SetDynamicMusic_f));

    // The `s_UseOpenAL` arm is dropped (DEC-57.4). The cvar still registers, so a
    // config that sets it still parses, and the software mixer always runs.
    Cvar_Get(view, "s_UseOpenAL", "0", CVAR_ARCHIVE | CVAR_LATCH);

    if SNDDMA_Init(view.common, snd) {
        snd.s_soundStarted = 1;
        snd.s_soundMuted = true;
        // do NOT reset s_numSfx here now!!

        snd.s_soundtime = 0;
        snd.s_paintedtime = 0;

        S_StopAllSounds(view.common, snd);

        S_SoundInfo_f(view, snd);
    }

    com_printf(view.common, "------------------------------------\n");

    com_printf(view.common, "\n--- ambient sound initialization ---\n");

    AS_Init(&mut snd.ambient);
}

/// Raven `S_Shutdown` — release every sound, close the device, and drop the commands.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:650-719`
pub fn S_Shutdown(view: &mut EngineHostView, snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 {
        return;
    }

    S_FreeAllSFXMem(snd);
    S_UnCacheDynamicMusic(snd);

    SNDDMA_Shutdown(snd);

    snd.s_soundStarted = 0;

    Cmd_RemoveCommand(view.common, "play");
    Cmd_RemoveCommand(view.common, "music");
    Cmd_RemoveCommand(view.common, "stopsound");
    Cmd_RemoveCommand(view.common, "soundlist");
    Cmd_RemoveCommand(view.common, "soundinfo");
    Cmd_RemoveCommand(view.common, "soundstop");
    Cmd_RemoveCommand(view.common, "mp3_calcvols");
    Cmd_RemoveCommand(view.common, "s_dynamic");

    AS_Free(&mut snd.ambient);
}

// ===========================================================================
// Console commands
// ===========================================================================

/// Raven `S_Play_f` — the `play` command.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:3667-3685`
fn S_Play_f(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };

    let mut i = 1;
    while i < Cmd_Argc(view.common) {
        let arg = Cmd_Argv(view.common, i).to_string();
        let name = if !arg.contains('.') {
            // Raven names `Cmd_Argv(1)` here, not `Cmd_Argv(i)`.
            format!("{}.wav", Cmd_Argv(view.common, 1))
        } else {
            arg
        };
        let h = S_RegisterSound(view, snd, &name);
        if h != 0 {
            S_StartLocalSound(view, snd, h, CHAN_LOCAL_SOUND);
        }
        i += 1;
    }
}

/// Raven `S_SoundInfo_f` — the `soundinfo` command adapter.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:360`
fn S_SoundInfo_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    S_SoundInfo_f(view, snd);
}

/// Raven's `soundstop` command, which is `S_StopAllSounds` itself.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:476`
fn S_StopAllSounds_f(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    S_StopAllSounds(view.common, snd);
}

/// Raven `sSoundCompressionMethodStrings` — the `soundlist` type column.
///
/// Raven: this table needs to be in sync with `SoundCompressionMethod_t`.
/// Source: `oracle/codemp/client/snd_dma.cpp:3775-3779`
const sSoundCompressionMethodStrings: [&str; 2] = ["16b", "mp3"];

/// Raven `S_SoundList_f` — the `soundlist` command.
///
/// The three mutually exclusive options are `wavonly`, `ShouldBeMP3`, and a
/// `1`/`2`/`3` cap on the `%d`-variant suffix a sound name may carry.
/// Source: `oracle/codemp/client/snd_dma.cpp:3780-3905`
fn S_SoundList_f(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };

    let mut iVariantCap: c_int = -1; // for %d-inquiry stuff
    let mut iTotalBytes = 0;
    let mut bWavOnly = false;
    let mut bShouldBeMP3 = false;

    if Cmd_Argc(view.common) == 2 {
        let arg = Cmd_Argv(view.common, 1).to_string();
        if arg.eq_ignore_ascii_case("shouldbeMP3") {
            bShouldBeMP3 = true;
        } else if arg.eq_ignore_ascii_case("wavonly") {
            bWavOnly = true;
        } else if arg == "1" || arg == "2" || arg == "3" {
            iVariantCap = arg.parse().unwrap_or(-1);
        }
    } else {
        com_printf(
            view.common,
            "( additional (mutually exclusive) options available:\n'wavonly', 'ShouldBeMP3', '1'/'2'/'3' for %d-variant capping )\n",
        );
    }

    let mut total = 0;

    com_printf(view.common, "\n");
    com_printf(view.common, "                    InMemory?\n");
    com_printf(view.common, "                    |\n");
    com_printf(view.common, "                    |  LevelLastUsedOn\n");
    com_printf(view.common, "                    |  |\n");
    com_printf(view.common, "                    |  |\n");
    com_printf(view.common, " Slot   Smpls Type  |  |   Name\n");

    let mp3Overhead = Cvar_Get(view, "s_mp3overhead", "0", CVAR_ARCHIVE);
    let mp3OverheadValue = view.common.cvar(mp3Overhead).integer;

    for i in 0..snd.s_knownSfx.len() {
        let bMP3DumpOverride = bShouldBeMP3
            && !snd.s_knownSfx[i].bDefaultSound
            && snd.s_knownSfx[i].pMP3StreamHeader.is_none()
            && snd.s_knownSfx[i].pSoundData.is_some()
            && SND_MemUsed(snd, i) > mp3OverheadValue;

        if !bMP3DumpOverride
            && (bShouldBeMP3
                || (bWavOnly
                    && snd.s_knownSfx[i].eSoundCompressionMethod
                        != SoundCompressionMethod_t::ct_16))
        {
            continue;
        }

        let mut bDumpThisOne = true;
        if (1..=3).contains(&iVariantCap) {
            let name = snd.s_knownSfx[i].sSoundName.clone();
            let bytes = name.as_bytes();
            if bytes.len() > 2 {
                let c = bytes[bytes.len() - 1];
                let c2 = bytes[bytes.len() - 2];
                // A quick way to avoid names like "pain75".
                if !c2.is_ascii_digit()
                    && c.is_ascii_digit()
                    && c_int::from(c - b'0') > iVariantCap
                {
                    // Skip this one where a %1 variant of it exists.
                    let mut sFindName = name.clone();
                    sFindName.replace_range(sFindName.len() - 1.., "1");
                    bDumpThisOne = !snd
                        .s_knownSfx
                        .iter()
                        .any(|sfx2| sfx2.sSoundName.eq_ignore_ascii_case(&sFindName));
                }
            }
        }

        let size = snd.s_knownSfx[i].iSoundLengthInSamples;
        if snd.s_knownSfx[i].bDefaultSound {
            let name = snd.s_knownSfx[i].sSoundName.clone();
            com_printf(view.common, &format!("{i:5} Missing file: \"{name}\"\n"));
            continue;
        }

        if bDumpThisOne {
            if snd.s_knownSfx[i].bInMemory {
                iTotalBytes += SND_MemUsed(snd, i);
                if snd.s_knownSfx[i].pMP3StreamHeader.is_some() {
                    iTotalBytes += core::mem::size_of::<MP3STREAM>() as c_int;
                }
                total += size;
            }
        }

        let method = snd.s_knownSfx[i].eSoundCompressionMethod as usize;
        let type_name = sSoundCompressionMethodStrings
            .get(method)
            .copied()
            .unwrap_or("???");
        let inMemory = if snd.s_knownSfx[i].bInMemory { "y" } else { "n" };
        let level = snd.s_knownSfx[i].iLastLevelUsedOn;
        let name = snd.s_knownSfx[i].sSoundName.clone();
        com_printf(
            view.common,
            &format!("{i:5} {size:7} [{type_name}] {inMemory} {level:2} {name}"),
        );

        if !bDumpThisOne {
            com_printf(view.common, "   ( Skipping, variant capped )");
        }
        com_printf(view.common, "\n");
    }
    com_printf(view.common, " Slot   Smpls Type  In? Lev  Name\n");

    let wavOnlyNote = if bWavOnly { "(WAV only)" } else { "" };
    com_printf(
        view.common,
        &format!("Total resident samples: {total} {wavOnlyNote} ( not mem usage, see 'meminfo' ).\n"),
    );
    let used = snd.s_knownSfx.len();
    com_printf(
        view.common,
        &format!("{used} out of {MAX_SFX} sfx_t slots used\n"),
    );
    let megs = iTotalBytes as f32 / 1024.0 / 1024.0;
    com_printf(
        view.common,
        &format!("{megs:.2}MB bytes used when counting sfx_t->pSoundData + MP3 headers (if any)\n"),
    );
    S_DisplayFreeMemory(view, snd);
}

/// Raven `S_Music_f` — the `music` command.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:3687-3700`
fn S_Music_f(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };

    let c = Cmd_Argc(view.common);

    if c == 2 {
        let name = Cmd_Argv(view.common, 1).to_string();
        S_StartBackgroundTrack(view, snd, &name, &name, false);
    } else if c == 3 {
        let intro = Cmd_Argv(view.common, 1).to_string();
        let loop_name = Cmd_Argv(view.common, 2).to_string();
        S_StartBackgroundTrack(view, snd, &intro, &loop_name, false);
    } else {
        com_printf(view.common, "music <musicfile> [loopfile]\n");
    }
}

/// Raven `S_SetDynamicMusic_f` — the `s_dynamic` command.
///
/// Raven calls it a debug function that does no harm left in. Explore, action,
/// and silence always exist where music is dynamic; boss and death are optional.
/// Source: `oracle/codemp/client/snd_dma.cpp:3704-3770`
fn S_SetDynamicMusic_f(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };

    if Cmd_Argc(view.common) == 2 {
        if !snd.bMusic_IsDynamic {
            DynamicMusicInfoPrint(view, snd); // print "inactive" string
            return;
        }

        let arg = Cmd_Argv(view.common, 1).to_string();
        if arg.eq_ignore_ascii_case("explore") {
            S_SetDynamicMusicState(view, snd, MusicState_e::eBGRNDTRACK_EXPLORE);
            return;
        }
        if arg.eq_ignore_ascii_case("action") {
            S_SetDynamicMusicState(view, snd, MusicState_e::eBGRNDTRACK_ACTION);
            return;
        }
        if arg.eq_ignore_ascii_case("silence") {
            S_SetDynamicMusicState(view, snd, MusicState_e::eBGRNDTRACK_SILENCE);
            return;
        }
        if arg.eq_ignore_ascii_case("boss") {
            if snd.tMusic_Info[MusicState_e::eBGRNDTRACK_BOSS as usize].bExists {
                S_SetDynamicMusicState(view, snd, MusicState_e::eBGRNDTRACK_BOSS);
            } else {
                com_printf(view.common, "No 'boss' music defined in current dynamic set\n");
            }
            return;
        }
        if arg.eq_ignore_ascii_case("death") {
            if snd.tMusic_Info[MusicState_e::eBGRNDTRACK_DEATH as usize].bExists {
                S_SetDynamicMusicState(view, snd, MusicState_e::eBGRNDTRACK_DEATH);
            } else {
                com_printf(view.common, "No 'death' music defined in current dynamic set\n");
            }
            return;
        }
    }

    // show usage...
    com_printf(
        view.common,
        "Usage: s_dynamic <explore/action/silence/boss/death>\n",
    );
    DynamicMusicInfoPrint(view, snd);
}

/// Raven `S_MP3_CalcVols_f` — the `mp3_calcvols` development command.
///
/// Source: `oracle/codemp/client/snd_mem.cpp:495-545`
fn S_MP3_CalcVols_f(view: &mut EngineHostView) {
    S_MP3_CalcVols_f_body(view);
}
