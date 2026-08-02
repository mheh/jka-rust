//! Pending-lane declarations for the music and ambient-set half of the sound stack.
//!
//! Ticket gh#24 landed the mixer (`snd_dma`, `snd_mem`, `snd_mix`) under DEC-57.
//! Ticket gh#25 owns what is left: the background-music loader, the ambient-set
//! parser, and the minimp3 wrap.
//! The client dispatchers and `cl_main` call this surface today, so each function
//! declares the shape its call sites need and panics when it runs.
//! No stub here is a silent no-op.
//!
//! Source: `oracle/codemp/client/snd_public.h`,
//! `oracle/codemp/client/snd_ambient.h`

use core::ffi::c_int;

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_qshared::shared::qboolean;
use native_types::sfxHandle_t;

use crate::snd::sound_system::SoundSystem;

//TODO: Port S_RestartMusic
// Source: oracle/codemp/client/snd_public.h:16
pub fn S_RestartMusic(_view: &mut EngineHostView, _snd: &mut SoundSystem) {
    todo!("Port S_RestartMusic — oracle/codemp/client/snd_dma.cpp:4479 (gh#25)")
}

//TODO: Port S_StartBackgroundTrack
// Source: oracle/codemp/client/snd_public.h:17
pub fn S_StartBackgroundTrack(
    _view: &mut EngineHostView,
    _snd: &mut SoundSystem,
    _intro: &str,
    _loop_track: &str,
    _bCalledByCGameStart: qboolean,
) {
    todo!("Port S_StartBackgroundTrack — oracle/codemp/client/snd_dma.cpp:4499 (gh#25)")
}

//TODO: Port S_UpdateAmbientSet
// Source: oracle/codemp/client/snd_ambient.h:113
pub fn S_UpdateAmbientSet(_view: &mut EngineHostView, _name: &str, _origin: *mut f32) {
    todo!("Port S_UpdateAmbientSet — oracle/codemp/client/snd_ambient.cpp:876 (gh#25)")
}

//TODO: Port S_AddLocalSet
// Source: oracle/codemp/client/snd_ambient.h:114
pub fn S_AddLocalSet(
    _view: &mut EngineHostView,
    _name: &str,
    _listener_origin: *mut f32,
    _origin: *mut f32,
    _entID: c_int,
    _time: c_int,
) -> c_int {
    todo!("Port S_AddLocalSet — oracle/codemp/client/snd_ambient.cpp:912 (gh#25)")
}

//TODO: Port AS_ParseSets
// Source: oracle/codemp/client/snd_ambient.h:110
pub fn AS_ParseSets(_view: &mut EngineHostView) {
    todo!("Port AS_ParseSets — oracle/codemp/client/snd_ambient.cpp:792 (gh#25)")
}

//TODO: Port AS_AddPrecacheEntry
// Source: oracle/codemp/client/snd_ambient.h:111
pub fn AS_AddPrecacheEntry(_view: &mut EngineHostView, _name: &str) {
    todo!("Port AS_AddPrecacheEntry — oracle/codemp/client/snd_ambient.cpp:772 (gh#25)")
}

//TODO: Port AS_GetBModelSound
// Source: oracle/codemp/client/snd_ambient.h:116
pub fn AS_GetBModelSound(_view: &mut EngineHostView, _name: &str, _stage: c_int) -> sfxHandle_t {
    todo!("Port AS_GetBModelSound — oracle/codemp/client/snd_ambient.cpp:1002 (gh#25)")
}
