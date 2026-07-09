use core::ffi::c_int;

use super::super::MpCgameExport;
use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};
use mp_qshared::shared::trajectory_t;

/// Arguments for `CG_GET_ANGLE_TRAJECTORY`.
///
/// Args source: `oracle/codemp/cgame/cg_main.c:296-297`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgGetAngleTrajectoryArgs {
    ent_num: c_int,
}

impl CgGetAngleTrajectoryArgs {
    pub const fn new(ent_num: c_int) -> Self {
        Self { ent_num }
    }

    pub const fn ent_num(self) -> c_int {
        self.ent_num
    }
}

/// `CG_GET_ANGLE_TRAJECTORY` MP cgame exports vmMain ABI token.
///
/// Raven: int entnum
/// Enum value source: `oracle/codemp/cgame/cg_public.h:422`
/// Args source: `oracle/codemp/cgame/cg_main.c:296-297`
/// Output source: `oracle/codemp/cgame/cg_main.c:296-297`
/// Output type source: `oracle/codemp/game/q_shared.h:2654-2660`
/// Transport/call-site source: `oracle/codemp/qcommon/RoffSystem.cpp:838`
/// Transport/call-site source: `oracle/codemp/qcommon/RoffSystem.cpp:984`
pub struct CgGetAngleTrajectory;

impl InboundVmCall for CgGetAngleTrajectory {
    type Command = MpCgameExport;
    type Args = CgGetAngleTrajectoryArgs;
    type Output = *mut trajectory_t;

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ANGLE_TRAJECTORY;
}

impl DecodeVmMain for CgGetAngleTrajectory {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgGetAngleTrajectoryArgs::new(word_to_c_int(transport.arg(0)))
    }
}

impl EncodeVmMainReturn for CgGetAngleTrajectory {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
