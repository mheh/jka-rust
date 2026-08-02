//! Pending-lane declarations for the sound stack.
//!
//! Tickets gh#24 (`snd_dma`, `snd_mem`, `snd_mix`) and gh#25 (music, ambient,
//! and the minimp3 wrap) own the real port, under DEC-57.
//! The client dispatchers and `cl_main` call this surface today, so each
//! function declares the shape its call sites need and panics when it runs.
//! No stub here is a silent no-op.
//!
//! Source: `oracle/codemp/client/snd_public.h`,
//! `oracle/codemp/client/snd_ambient.h`

use core::ffi::c_int;

use mp_engine_qcommon::common::common::Common;
use native_types::sfxHandle_t;
use native_math::vector::vec3_t;
use native_types::byte;
use native_types::qboolean;

use crate::client_host::Client;

//TODO: Port S_Init
// Source: oracle/codemp/client/snd_public.h:4
pub fn S_Init(_cl: &mut Client) {
    todo!("Port S_Init — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_Shutdown
// Source: oracle/codemp/client/snd_public.h:5
pub fn S_Shutdown(_cl: &mut Client) {
    todo!("Port S_Shutdown — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_MuteSound
// Source: oracle/codemp/client/snd_public.h:10
pub fn S_MuteSound(_cl: &mut Client, _entityNum: c_int, _entchannel: c_int) {
    todo!("Port S_MuteSound — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_StartSound
// Source: oracle/codemp/client/snd_public.h:11
pub fn S_StartSound(
    _cl: &mut Client,
    _origin: *mut f32,
    _entnum: c_int,
    _entchannel: c_int,
    _sfx: sfxHandle_t,
) {
    todo!("Port S_StartSound — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_StartLocalSound
// Source: oracle/codemp/client/snd_public.h:12
pub fn S_StartLocalSound(_cl: &mut Client, _sfx: sfxHandle_t, _channelNum: c_int) {
    todo!("Port S_StartLocalSound — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_RestartMusic
// Source: oracle/codemp/client/snd_public.h:16
pub fn S_RestartMusic(_cl: &mut Client) {
    todo!("Port S_RestartMusic — oracle/codemp/client/snd_music.cpp (gh#25)")
}

//TODO: Port S_StartBackgroundTrack
// Source: oracle/codemp/client/snd_public.h:17
pub fn S_StartBackgroundTrack(
    _cl: &mut Client,
    _intro: &str,
    _loop_track: &str,
    _bCalledByCGameStart: qboolean,
) {
    todo!("Port S_StartBackgroundTrack — oracle/codemp/client/snd_music.cpp (gh#25)")
}

//TODO: Port S_StopBackgroundTrack
// Source: oracle/codemp/client/snd_public.h:18
pub fn S_StopBackgroundTrack(_cl: &mut Client) {
    todo!("Port S_StopBackgroundTrack — oracle/codemp/client/snd_music.cpp (gh#25)")
}

//TODO: Port S_RawSamples
// Source: oracle/codemp/client/snd_public.h:23
pub fn S_RawSamples(
    _cl: &mut Client,
    _samples: c_int,
    _rate: c_int,
    _width: c_int,
    _s_channels: c_int,
    _data: *mut byte,
    _volume: f32,
    _bFirstOrOnlyUpdateThisFrame: c_int,
) {
    todo!("Port S_RawSamples — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_StopAllSounds
// Source: oracle/codemp/client/snd_public.h:27
pub fn S_StopAllSounds(_cl: &mut Client) {
    todo!("Port S_StopAllSounds — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_ClearLoopingSounds
// Source: oracle/codemp/client/snd_public.h:33
pub fn S_ClearLoopingSounds(_cl: &mut Client) {
    todo!("Port S_ClearLoopingSounds — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_StopLoopingSound
// Source: oracle/codemp/client/snd_public.h:34
pub fn S_StopLoopingSound(_cl: &mut Client, _entityNum: c_int) {
    todo!("Port S_StopLoopingSound — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_AddLoopingSound
// Source: oracle/codemp/client/snd_public.h:36-38
pub fn S_AddLoopingSound(
    _cl: &mut Client,
    _entityNum: c_int,
    _origin: vec3_t,
    _velocity: vec3_t,
    _sfx: sfxHandle_t,
) {
    todo!("Port S_AddLoopingSound — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_Respatialize
// Source: oracle/codemp/client/snd_public.h:43
pub fn S_Respatialize(
    _cl: &mut Client,
    _entityNum: c_int,
    _head: vec3_t,
    _axis: *mut vec3_t,
    _inwater: c_int,
) {
    todo!("Port S_Respatialize — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_UpdateEntityPosition
// Source: oracle/codemp/client/snd_public.h:46
pub fn S_UpdateEntityPosition(_cl: &mut Client, _entityNum: c_int, _origin: vec3_t) {
    todo!("Port S_UpdateEntityPosition — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_Update
// Source: oracle/codemp/client/snd_public.h:48
pub fn S_Update(_common: &mut Common, _cl: &mut Client) {
    todo!("Port S_Update — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_DisableSounds
// Source: oracle/codemp/client/snd_public.h:50
pub fn S_DisableSounds(_cl: &mut Client) {
    todo!("Port S_DisableSounds — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_BeginRegistration
// Source: oracle/codemp/client/snd_public.h:52
pub fn S_BeginRegistration(_cl: &mut Client) {
    todo!("Port S_BeginRegistration — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_RegisterSound
// Source: oracle/codemp/client/snd_public.h:57
pub fn S_RegisterSound(_cl: &mut Client, _sample: &str) -> sfxHandle_t {
    todo!("Port S_RegisterSound — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_ClearSoundBuffer
// Source: oracle/codemp/client/snd_dma.cpp:1718
pub fn S_ClearSoundBuffer(_cl: &mut Client) {
    todo!("Port S_ClearSoundBuffer — oracle/codemp/client/snd_dma.cpp (gh#24)")
}

//TODO: Port S_UpdateAmbientSet
// Source: oracle/codemp/client/snd_ambient.h:113
pub fn S_UpdateAmbientSet(
    _common: &mut Common,
    _cl: &mut Client,
    _name: &str,
    _origin: *mut f32,
) {
    todo!("Port S_UpdateAmbientSet — oracle/codemp/client/snd_ambient.cpp (gh#25)")
}

//TODO: Port S_AddLocalSet
// Source: oracle/codemp/client/snd_ambient.h:114
pub fn S_AddLocalSet(
    _common: &mut Common,
    _cl: &mut Client,
    _name: &str,
    _listener_origin: *mut f32,
    _origin: *mut f32,
    _entID: c_int,
    _time: c_int,
) -> c_int {
    todo!("Port S_AddLocalSet — oracle/codemp/client/snd_ambient.cpp (gh#25)")
}

//TODO: Port AS_ParseSets
// Source: oracle/codemp/client/snd_ambient.h:110
pub fn AS_ParseSets(_cl: &mut Client) {
    todo!("Port AS_ParseSets — oracle/codemp/client/snd_ambient.cpp (gh#25)")
}

//TODO: Port AS_AddPrecacheEntry
// Source: oracle/codemp/client/snd_ambient.h:111
pub fn AS_AddPrecacheEntry(_cl: &mut Client, _name: &str) {
    todo!("Port AS_AddPrecacheEntry — oracle/codemp/client/snd_ambient.cpp (gh#25)")
}

//TODO: Port AS_GetBModelSound
// Source: oracle/codemp/client/snd_ambient.h:116
pub fn AS_GetBModelSound(_cl: &mut Client, _name: &str, _stage: c_int) -> sfxHandle_t {
    todo!("Port AS_GetBModelSound — oracle/codemp/client/snd_ambient.cpp (gh#25)")
}
