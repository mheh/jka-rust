//! `snd_dma.cpp` — channels, spatialization, the sfx cache, and the frame driver.
//!
//! DEC-57.4 drops the OpenAL and EAX arm, so every `s_UseOpenAL` branch is gone
//! and only the software mixer is ported. DEC-57.1 dissolves the five `SNDDMA_*`
//! functions into the device end: `SoundSystem` owns the ring and the read
//! cursor, and the device end writes the cursor.
//! The background-music loader is gh#25 (DEC-57.3), so this file carries only
//! the music paths the mixer itself runs.
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
use mp_qshared::shared::cvar::{CVAR_ARCHIVE, CVAR_CHEAT, CVAR_LATCH, CVAR_NORESTART, CVAR_ROM};
use mp_qshared::shared::error_parm::errorParm_t;
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
use native_platform::sys_main::Sys_LowPhysicalMemory;
use native_string::q_string::Q_stricmp;

use crate::client_host::snd_from_view;
use crate::mp3::mp3_stream::MP3STREAM;
use crate::snd::channel_t::{channel_t, START_SAMPLE_IMMEDIATE};
use crate::snd::loop_sound_t::MAX_LOOP_SOUNDS;
use crate::snd::music_state_e::MusicState_e;
use crate::snd::sound_system::{
    SoundSystem, BGRNDTRACK_NUMBEROF, LOOP_HASH, MAX_CHANNELS, MAX_RAW_SAMPLES, MAX_SFX,
};
use crate::snd_device::SoundDevice;
use crate::snd_mem::S_LoadSound;
use crate::snd_mix::S_PaintChannels;

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

// ===========================================================================
// The device end (DEC-57.1)
// ===========================================================================

/// Raven `SNDDMA_Init` - pick the output format, allocate the ring, and open
/// the output device.
///
/// The port keeps the retail secondary-buffer shape and owns the ring outright.
/// The device is cpal (DEC-57.1), and a host that offers none keeps the ring
/// and paints in silence, exactly as before the device end landed.
/// Source: `oracle/codemp/win32/win_snd.cpp:105-257`
fn SNDDMA_Init(common: &mut Common, snd: &mut SoundSystem) -> bool {
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

    snd.device = None;
    if snd.device_enabled {
        match SoundDevice::open(snd.dma.speed, snd.dma.channels, DMA_BUFFER_BYTES) {
            Ok(device) => {
                com_printf(common, &format!("sound device: {}\n", device.description()));
                snd.device = Some(device);
            }
            Err(reason) => {
                com_printf(common, &format!("sound device unavailable: {reason}\n"));
            }
        }
    }

    true
}

/// Raven `SNDDMA_Shutdown` - release the ring and close the device.
///
/// Source: `oracle/codemp/win32/win_snd.cpp:51-95`
fn SNDDMA_Shutdown(snd: &mut SoundSystem) {
    // Dropping the stream stops playback, the whole of Raven's teardown.
    snd.device = None;
    snd.dma.buffer = Vec::new();
}

/// Raven `SNDDMA_GetDMAPos` — the device read cursor, masked to the ring.
///
/// With no device open the cursor is whatever last wrote `dma_pos`, so a
/// headless rig drives the mix clock itself.
/// Source: `oracle/codemp/win32/win_snd.cpp:267-286`
fn SNDDMA_GetDMAPos(snd: &mut SoundSystem) -> c_int {
    if let Some(device) = snd.device.as_ref() {
        snd.dma_pos = device.play_cursor();
    }
    snd.dma_pos & (snd.dma.samples - 1)
}

// Raven's `SNDDMA_BeginPainting` locks the DirectSound secondary buffer. The
// port owns the ring outright, so it has no body and its call sites drop it.

/// Raven `SNDDMA_Submit` - hand the painted ring to the device.
///
/// Raven only unlocked the buffer here, because DirectSound played the very
/// memory the paint chain wrote. The engine owns its ring now, so this is where
/// the device gets the bytes.
/// Source: `oracle/codemp/win32/win_snd.cpp:350-355`
fn SNDDMA_Submit(snd: &SoundSystem) {
    if let Some(device) = snd.device.as_ref() {
        device.publish(&snd.dma.buffer);
    }
}

// ===========================================================================
// Channels and the sfx cache
// ===========================================================================

/// Raven `Channel_Clear` — reset one channel.
///
/// Raven skips the MP3 sliding-decode buffer in the middle of the struct. That
/// buffer is gh#25 and no field carries it yet, so the whole channel resets.
/// Source: `oracle/codemp/client/snd_dma.cpp:321-330`
fn Channel_Clear(snd: &mut SoundSystem, channel: usize) {
    snd.s_channels[channel] = channel_t::default();
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
        .expect("SND_malloc seated the default sound's block");
    for i in 0..data.len() {
        data[i] = i as i16;
    }
}

/// Raven `S_DisableSounds` — stop everything until the next `S_BeginRegistration`.
///
/// Raven: this is called when the hunk is cleared and the sounds are no longer valid.
/// Source: `oracle/codemp/client/snd_dma.cpp:890-893`
pub fn S_DisableSounds(snd: &mut SoundSystem) {
    S_StopAllSounds(snd);
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
    SNDDMA_Submit(snd);
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
pub fn S_StopAllSounds(snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 {
        return;
    }
    // stop the background music
    S_StopBackgroundTrack(snd);

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

//TODO: Port S_ByteSwapRawSamples
// Source: oracle/codemp/client/snd_dma.cpp:2071. Only the streamed-WAV music
// path calls it, and that path is gh#25. The body is a no-op on a little-endian
// host, which is every target this tree builds for.

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
                        snd.s_knownSfx[sfx].pSoundData.as_ref()?.get(index).copied()
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
pub fn S_Update(common: &mut Common, snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

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
    S_UpdateBackgroundTrack(common, snd);

    // mix some sound
    S_Update_(common, snd);
}

/// Raven `S_GetSoundtime` — read the device cursor and set the mix window.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:2743-2784`
fn S_GetSoundtime(common: &Common, snd: &mut SoundSystem) {
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
            S_StopAllSounds(snd);
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
pub fn S_Update_(common: &mut Common, snd: &mut SoundSystem) {
    if snd.s_soundStarted == 0 || snd.s_soundMuted {
        return;
    }

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

    SNDDMA_Submit(snd);

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
    snd.s_knownSfx[sfx].pSoundData = Some(vec![0i16; (iSize / 2).max(0) as usize]);
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
fn SND_MemUsed(snd: &SoundSystem, sfx: usize) -> c_int {
    match &snd.s_knownSfx[sfx].pSoundData {
        Some(data) => (data.len() * 2) as c_int,
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

    snd.s_knownSfx[sfx].bInMemory = false;

    iBytesFreed
}

/// Raven `S_DisplayFreeMemory` — print the audio pool totals, and print nothing
/// where the pool is empty.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:5117-5144`
fn S_DisplayFreeMemory(view: &mut EngineHostView, snd: &SoundSystem) {
    let iSoundDataSize = snd.sndRawDataBytes;
    // The dynamic-music tag is gh#25, so its total is zero here.
    let iMusicDataSize = 0;

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

/// Raven `S_StopBackgroundTrack_Actual` — close one music track's file.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:4023-4034`
fn S_StopBackgroundTrack_Actual(snd: &mut SoundSystem, track: usize) {
    if snd.tMusic_Info[track].s_backgroundFile != 0 {
        if snd.tMusic_Info[track].s_backgroundFile != -1 {
            //TODO: Port S_StopBackgroundTrack_Actual streamed-file close
            // Source: oracle/codemp/client/snd_dma.cpp:4029. The streamed-file
            // seam arrives with gh#25, and no gh#24 path opens a track.
            todo!("Port Sys_EndStreamedFile — oracle/codemp/client/snd_dma.cpp:4029 (gh#25)")
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
pub fn S_StopBackgroundTrack(snd: &mut SoundSystem) {
    for track in 0..BGRNDTRACK_NUMBEROF {
        S_StopBackgroundTrack_Actual(snd, track);
    }

    snd.s_rawend = 0;
}

/// Raven `S_UpdateBackgroundTrack_Actual` — feed the raw ring from one music track.
///
/// A track with no open file returns at once, which is every gh#24 path.
/// Source: `oracle/codemp/client/snd_dma.cpp:4638-4807`
fn S_UpdateBackgroundTrack_Actual(snd: &mut SoundSystem, track: usize, fDefaultVolume: f32) -> bool {
    let fMasterVol = fDefaultVolume;

    // The dynamic cross-fade step above this line runs only under
    // `bMusic_IsDynamic`, which gh#24 never sets.

    if snd.tMusic_Info[track].s_backgroundFile == 0 {
        return false;
    }

    snd.tMusic_Info[track].fSmoothedOutVolume =
        (snd.tMusic_Info[track].fSmoothedOutVolume + fMasterVol) / 2.0;

    // don't bother playing anything if musicvolume is 0
    if snd.tMusic_Info[track].fSmoothedOutVolume <= 0.0 {
        return false;
    }

    //TODO: Port S_UpdateBackgroundTrack_Actual streaming body
    // Source: oracle/codemp/client/snd_dma.cpp:4686-4801. The streamed-file and
    // MP3 reads arrive with gh#25, and no gh#24 path opens a track.
    todo!("Port S_UpdateBackgroundTrack_Actual — oracle/codemp/client/snd_dma.cpp:4686 (gh#25)")
}

/// Raven `S_UpdateBackgroundTrack` — the per-frame music step.
///
/// MP's `S_Music_GetRequestedState` always answers NULL, so the non-dynamic arm
/// runs at the plain music volume.
/// Source: `oracle/codemp/client/snd_dma.cpp:4893-5009`
fn S_UpdateBackgroundTrack(common: &mut Common, snd: &mut SoundSystem) {
    if snd.bMusic_IsDynamic {
        //TODO: Port S_CheckDynamicMusicState
        // Source: oracle/codemp/client/snd_dma.cpp:4895-4991. Dynamic music is
        // gh#25, and no gh#24 path sets `bMusic_IsDynamic`.
        todo!("Port dynamic S_UpdateBackgroundTrack — oracle/codemp/client/snd_dma.cpp:4895 (gh#25)")
    }

    // standard / non-dynamic one-track music...
    // MP's `S_Music_GetRequestedState` returns NULL, so the silence check never fires.
    let fDesiredVolume = common.cvar(snd.s_musicVolume).value;

    // internal to this code is a volume-smoother...
    let bNewTrackDesired = S_UpdateBackgroundTrack_Actual(
        snd,
        MusicState_e::eBGRNDTRACK_NONDYNAMIC as usize,
        fDesiredVolume,
    );

    if bNewTrackDesired {
        //TODO: Port S_StartBackgroundTrack
        // Source: oracle/codemp/client/snd_dma.cpp:4499. gh#25 owns the loader.
        todo!("Port S_StartBackgroundTrack — oracle/codemp/client/snd_dma.cpp:4499 (gh#25)")
    }
}

// ===========================================================================
// Init and shutdown
// ===========================================================================

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
            //TODO: Port DynamicMusicInfoPrint
            // Source: oracle/codemp/client/snd_dma.cpp:335. Dynamic music is gh#25.
            todo!("Port DynamicMusicInfoPrint — oracle/codemp/client/snd_dma.cpp:335 (gh#25)")
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

        S_StopAllSounds(snd);

        S_SoundInfo_f(view, snd);
    }

    com_printf(view.common, "------------------------------------\n");

    com_printf(view.common, "\n--- ambient sound initialization ---\n");

    AS_Init();
}

/// Raven `AS_Init` — seat the ambient-set container.
///
/// The container has no observable effect until a set file parses, and the
/// parser is gh#25.
/// Source: `oracle/codemp/client/snd_ambient.cpp:752-764`
fn AS_Init() {
    //TODO: Port AS_Init
    // Source: oracle/codemp/client/snd_ambient.cpp:752. The ambient-set parser is
    // gh#25; the allocation Raven does here has no other observable effect.
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
    // Raven `AS_Free`, which frees the ambient-set container gh#25 fills.
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
    S_StopAllSounds(snd);
}

/// Raven `S_SoundList_f` — the `soundlist` command.
///
//TODO: Port S_SoundList_f
// Source: oracle/codemp/client/snd_dma.cpp:3780. The listing reads the MP3
// overhead cvar and the MP3 header sizes, which arrive with gh#25.
fn S_SoundList_f(_view: &mut EngineHostView) {
    todo!("Port S_SoundList_f — oracle/codemp/client/snd_dma.cpp:3780 (gh#25)")
}

/// Raven `S_Music_f` — the `music` command.
///
//TODO: Port S_Music_f
// Source: oracle/codemp/client/snd_dma.cpp:3687. gh#25 owns the music loader.
fn S_Music_f(_view: &mut EngineHostView) {
    todo!("Port S_Music_f — oracle/codemp/client/snd_dma.cpp:3687 (gh#25)")
}

/// Raven `S_SetDynamicMusic_f` — the `s_dynamic` command.
///
//TODO: Port S_SetDynamicMusic_f
// Source: oracle/codemp/client/snd_dma.cpp:3704. gh#25 owns dynamic music.
fn S_SetDynamicMusic_f(_view: &mut EngineHostView) {
    todo!("Port S_SetDynamicMusic_f — oracle/codemp/client/snd_dma.cpp:3704 (gh#25)")
}

/// Raven `S_MP3_CalcVols_f` — the `mp3_calcvols` development command.
///
//TODO: Port S_MP3_CalcVols_f
// Source: oracle/codemp/client/snd_mem.cpp:495. The MP3 re-tag pass is gh#25.
fn S_MP3_CalcVols_f(_view: &mut EngineHostView) {
    todo!("Port S_MP3_CalcVols_f — oracle/codemp/client/snd_mem.cpp:495 (gh#25)")
}
