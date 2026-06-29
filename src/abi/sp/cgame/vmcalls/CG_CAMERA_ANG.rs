use core::ffi::{c_float, c_int};

use super::super::SpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_CAMERA_ANG` SP cgame exports vmMain ABI token.
///
/// Raven: `int CG_GetCameraAng( vec3_t cameraAng );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:20`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:28`, `oracle/oracle/code/cgame/cg_main.cpp:114`,
/// `oracle/oracle/code/game/q_shared.h:316`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:517`
/// VM_Call/vmMain switch source: `oracle/oracle/code/server/sv_snapshot.cpp:283`,
/// `oracle/oracle/code/cgame/cg_main.cpp:94-115`
pub struct CgCameraAng;

impl InboundVmCall for CgCameraAng {
    type Command = SpCgameExport;
    /// FIXME: create type `vec3_t` in Rust (Raven source: `oracle/oracle/code/game/q_shared.h:316`).
    /// Using `*mut c_float` keeps transport compatibility for this pointer payload.
    type Args = *mut c_float;
    type Output = c_int;

    const COMMAND: SpCgameExport = SpCgameExport::CG_CAMERA_ANG;
}
