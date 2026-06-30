use core::ffi::c_int;

use super::super::SpCgameExport;
use crate::abi::generic::{
    word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};
use crate::shared::vec3_t;

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
    type Args = *mut vec3_t;
    type Output = c_int;

    const COMMAND: SpCgameExport = SpCgameExport::CG_CAMERA_ANG;
}

impl DecodeVmMain for CgCameraAng {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        word_to_mut_ptr(transport.arg(0))
    }
}

impl EncodeVmMainReturn for CgCameraAng {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
