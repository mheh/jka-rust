#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::refdef_t::refdef_t;

/// Raven `SFxHelper` — the fx system's abstraction over engine services
/// (time, sound, tracing, scene/decal registration).
///
/// Raven: (none).
/// Type definition source: `oracle/codemp/client/FxSystem.h:49-219`
#[repr(C)]
pub struct SFxHelper {
    pub mTime: i32,
    pub mOldTime: i32,
    pub mFrameTime: i32,
    pub mTimeFrozen: bool,
    pub mRealTime: f32,
    pub refdef: *mut refdef_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<SFxHelper>() == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SFxHelper, mTime) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SFxHelper, mOldTime) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SFxHelper, mFrameTime) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SFxHelper, mTimeFrozen) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SFxHelper, mRealTime) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SFxHelper, refdef) == 24);
