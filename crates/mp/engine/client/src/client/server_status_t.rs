#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::limits::BIG_INFO_STRING;
use mp_qshared::shared::qboolean;

/// Raven `serverStatus_t` — one slot of the client's server-status request
/// cache. `cl_main.cpp` owns this type, and the `ui` module declares an
/// unrelated struct of the same name.
///
/// Type definition source: `oracle/codemp/client/cl_main.cpp:115-123`
#[repr(C)]
pub struct serverStatus_t {
    pub string: [c_char; BIG_INFO_STRING],
    pub address: netadr_t,
    pub time: i32,
    pub startTime: i32,
    pub pending: qboolean,
    pub print: qboolean,
    pub retrieved: qboolean,
}

const _: () = assert!(core::mem::size_of::<serverStatus_t>() == 8232);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, address) == 8192);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, time) == 8212);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, startTime) == 8216);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, pending) == 8220);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, print) == 8224);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, retrieved) == 8228);

// Every field is a scalar or a scalar array, and `netadr_t`'s zero discriminant
// is `NA_BOT`, so the all-zero image is a valid inhabitant. Raven's
// `cl_serverStatusList` is a zero-filled file static.
unsafe impl native_platform::ZeroValid for serverStatus_t {}
