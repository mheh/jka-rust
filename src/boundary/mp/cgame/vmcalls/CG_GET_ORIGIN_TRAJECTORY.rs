use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::boundary::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};
use crate::codemp::game::q_shared_h::trajectory_t;

/// Arguments for `CG_GET_ORIGIN_TRAJECTORY`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:293-295`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgGetOriginTrajectoryArgs {
    ent_num: c_int,
}

impl CgGetOriginTrajectoryArgs {
    pub const fn new(ent_num: c_int) -> Self {
        Self { ent_num }
    }

    pub const fn ent_num(self) -> c_int {
        self.ent_num
    }
}

/// `CG_GET_ORIGIN_TRAJECTORY` MP cgame exports vmMain boundary token.
///
/// Raven: int entnum
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:421`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:293-295`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:293-295`
/// Output type source: `oracle/oracle/codemp/game/q_shared.h:2654-2660`
/// Transport/call-site source: no engine call-site found in initial search; module vmMain switch proves arg slots.
pub struct CgGetOriginTrajectory;

impl InboundVmCall for CgGetOriginTrajectory {
    type Command = MpCgameExport;
    type Args = CgGetOriginTrajectoryArgs;
    type Output = *mut trajectory_t;

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ORIGIN_TRAJECTORY;
}

impl DecodeVmMain for CgGetOriginTrajectory {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgGetOriginTrajectoryArgs::new(word_to_c_int(transport.arg(0)))
    }
}

impl EncodeVmMainReturn for CgGetOriginTrajectory {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
