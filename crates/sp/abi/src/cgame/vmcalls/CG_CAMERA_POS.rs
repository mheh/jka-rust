use core::ffi::c_int;

use super::super::SpCgameExport;
use abi_transport::generic::{
    word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};
use sp_qshared::shared::vec3_t;

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
    type Args = *mut vec3_t;
    type Output = c_int;

    const COMMAND: SpCgameExport = SpCgameExport::CG_CAMERA_POS;
}

impl DecodeVmMain for CgCameraPos {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        word_to_mut_ptr(transport.arg(0))
    }
}

impl EncodeVmMainReturn for CgCameraPos {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
