#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int, c_ulong, c_void};

use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::vec3_t;

// Raven's `DWORD` (`oracle/oracle/codemp/qcommon/platform.h:13-20`): an unsigned
// 32-bit integer on the platforms this codebase targets.
type DWORD = c_ulong;

/// Raven `interface_export_t` — function table Icarus calls into the game/engine
/// through.
///
/// Raven: (unnamed).
/// Type definition source: `oracle/oracle/codemp/game/../icarus/interface.h:17-70`
#[repr(C)]
pub struct interface_export_s {
    // General
    pub I_LoadFile:
        Option<unsafe extern "C" fn(name: *const c_char, buf: *mut *mut c_void) -> c_int>,
    pub I_CenterPrint: Option<unsafe extern "C" fn(format: *const c_char, ...)>,
    pub I_DPrintf: Option<unsafe extern "C" fn(arg1: c_int, arg2: *const c_char, ...)>,
    /// Polls the engine for the sequencer of the entity matching the name passed
    pub I_GetEntityByName: Option<unsafe extern "C" fn(name: *const c_char) -> *mut sharedEntity_t>,
    /// Gets the current time
    pub I_GetTime: Option<unsafe extern "C" fn() -> DWORD>,
    pub I_GetTimeScale: Option<unsafe extern "C" fn() -> DWORD>,
    pub I_PlaySound: Option<
        unsafe extern "C" fn(
            taskID: c_int,
            entID: c_int,
            name: *const c_char,
            channel: *const c_char,
        ) -> c_int,
    >,
    pub I_Lerp2Pos: Option<
        unsafe extern "C" fn(
            taskID: c_int,
            entID: c_int,
            origin: *mut vec3_t,
            angles: *mut vec3_t,
            duration: c_float,
        ),
    >,
    pub I_Lerp2Origin: Option<
        unsafe extern "C" fn(taskID: c_int, entID: c_int, origin: *mut vec3_t, duration: c_float),
    >,
    pub I_Lerp2Angles: Option<
        unsafe extern "C" fn(taskID: c_int, entID: c_int, angles: *mut vec3_t, duration: c_float),
    >,
    pub I_GetTag: Option<
        unsafe extern "C" fn(
            entID: c_int,
            name: *const c_char,
            lookup: c_int,
            info: *mut vec3_t,
        ) -> c_int,
    >,
    pub I_Lerp2Start: Option<unsafe extern "C" fn(taskID: c_int, entID: c_int, duration: c_float)>,
    pub I_Lerp2End: Option<unsafe extern "C" fn(taskID: c_int, entID: c_int, duration: c_float)>,
    pub I_Set: Option<
        unsafe extern "C" fn(
            taskID: c_int,
            entID: c_int,
            type_name: *const c_char,
            data: *const c_char,
        ),
    >,
    pub I_Use: Option<unsafe extern "C" fn(entID: c_int, name: *const c_char)>,
    pub I_Kill: Option<unsafe extern "C" fn(entID: c_int, name: *const c_char)>,
    pub I_Remove: Option<unsafe extern "C" fn(entID: c_int, name: *const c_char)>,
    pub I_Random: Option<unsafe extern "C" fn(min: c_float, max: c_float) -> c_float>,
    pub I_Play: Option<
        unsafe extern "C" fn(taskID: c_int, entID: c_int, r#type: *const c_char, name: *const c_char),
    >,

    // Camera functions
    pub I_CameraPan:
        Option<unsafe extern "C" fn(angles: *mut vec3_t, dir: *mut vec3_t, duration: c_float)>,
    pub I_CameraMove: Option<unsafe extern "C" fn(origin: *mut vec3_t, duration: c_float)>,
    pub I_CameraZoom: Option<unsafe extern "C" fn(fov: c_float, duration: c_float)>,
    pub I_CameraRoll: Option<unsafe extern "C" fn(angle: c_float, duration: c_float)>,
    pub I_CameraFollow:
        Option<unsafe extern "C" fn(name: *const c_char, speed: c_float, initLerp: c_float)>,
    pub I_CameraTrack:
        Option<unsafe extern "C" fn(name: *const c_char, speed: c_float, initLerp: c_float)>,
    pub I_CameraDistance: Option<unsafe extern "C" fn(dist: c_float, initLerp: c_float)>,
    pub I_CameraFade: Option<
        unsafe extern "C" fn(
            sr: c_float,
            sg: c_float,
            sb: c_float,
            sa: c_float,
            dr: c_float,
            dg: c_float,
            db: c_float,
            da: c_float,
            duration: c_float,
        ),
    >,
    pub I_CameraPath: Option<unsafe extern "C" fn(name: *const c_char)>,
    pub I_CameraEnable: Option<unsafe extern "C" fn()>,
    pub I_CameraDisable: Option<unsafe extern "C" fn()>,
    pub I_CameraShake: Option<unsafe extern "C" fn(intensity: c_float, duration: c_int)>,

    pub I_GetFloat: Option<
        unsafe extern "C" fn(
            entID: c_int,
            r#type: c_int,
            name: *const c_char,
            value: *mut c_float,
        ) -> c_int,
    >,
    pub I_GetVector: Option<
        unsafe extern "C" fn(
            entID: c_int,
            r#type: c_int,
            name: *const c_char,
            value: *mut vec3_t,
        ) -> c_int,
    >,
    pub I_GetString: Option<
        unsafe extern "C" fn(
            entID: c_int,
            r#type: c_int,
            name: *const c_char,
            value: *mut *mut c_char,
        ) -> c_int,
    >,

    pub I_Evaluate: Option<
        unsafe extern "C" fn(
            p1Type: c_int,
            p1: *const c_char,
            p2Type: c_int,
            p2: *const c_char,
            operatorType: c_int,
        ) -> c_int,
    >,

    pub I_DeclareVariable: Option<unsafe extern "C" fn(r#type: c_int, name: *const c_char)>,
    pub I_FreeVariable: Option<unsafe extern "C" fn(name: *const c_char)>,

    // Save / Load functions
    pub I_WriteSaveData:
        Option<unsafe extern "C" fn(chid: c_ulong, data: *mut c_void, length: c_int) -> c_int>,
    // Below changed by BTO (VV). Visual C++ 7.1 compiler no longer allows default args
    // on function pointers. Ack.
    pub I_ReadSaveData: Option<
        unsafe extern "C" fn(
            chid: c_ulong,
            address: *mut c_void,
            length: c_int, /* , addressptr: *mut *mut c_void = NULL */
        ) -> c_int,
    >,
    //TODO: Port CSequencer
    // Source: oracle/oracle/codemp/game/../icarus/interface.h:68
    //TODO: Port CTaskManager
    // Source: oracle/oracle/codemp/game/../icarus/interface.h:68
    pub I_LinkEntity: Option<
        unsafe extern "C" fn(
            entID: c_int,
            sequencer: *mut c_void,
            taskManager: *mut c_void,
        ) -> c_int,
    >,
}

/// Raven `interface_export_t` typedef alias.
pub type interface_export_t = interface_export_s;

const _: () = assert!(core::mem::size_of::<interface_export_t>() == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_LoadFile) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CenterPrint) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_DPrintf) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_GetEntityByName) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_GetTime) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_GetTimeScale) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_PlaySound) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Lerp2Pos) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Lerp2Origin) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Lerp2Angles) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_GetTag) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Lerp2Start) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Lerp2End) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Set) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Use) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Kill) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Remove) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Random) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Play) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraPan) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraMove) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraZoom) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraRoll) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraFollow) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraTrack) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraDistance) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraFade) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraPath) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraEnable) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraDisable) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_CameraShake) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_GetFloat) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_GetVector) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_GetString) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_Evaluate) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_DeclareVariable) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_FreeVariable) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_WriteSaveData) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_ReadSaveData) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(interface_export_t, I_LinkEntity) == 312);
