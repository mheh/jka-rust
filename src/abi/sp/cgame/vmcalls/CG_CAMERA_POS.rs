use core::ffi::{c_float, c_int};

use super::super::SpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_CAMERA_POS` SP cgame exports vmMain ABI token.
///
/// Raven: `int CG_GetCameraPos( vec3_t camerapos );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:19`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:28`, `oracle/oracle/code/cgame/cg_main.cpp:112`,
/// `oracle/oracle/code/game/q_shared.h:316`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:474`
/// VM_Call/vmMain switch source: `oracle/oracle/code/server/sv_snapshot.cpp:281-283`,
/// `oracle/oracle/code/server/sv_snapshot.cpp:541`, `oracle/oracle/code/cgame/cg_main.cpp:94-115`
pub struct CgCameraPos;

impl InboundVmCall for CgCameraPos {
    type Command = SpCgameExport;
    /// FIXME: create type `vec3_t` in Rust (Raven source: `oracle/oracle/code/game/q_shared.h:316`).
    /// Using `*mut c_float` keeps transport compatibility for this pointer payload.
    type Args = *mut c_float;
    type Output = c_int;

    const COMMAND: SpCgameExport = SpCgameExport::CG_CAMERA_POS;
}
