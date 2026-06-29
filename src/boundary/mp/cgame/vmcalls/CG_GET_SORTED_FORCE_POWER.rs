use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::boundary::generic::word_to_c_int;
use crate::boundary::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `CG_GET_SORTED_FORCE_POWER` MP cgame exports vmMain boundary token.
///
/// Raven: forcePowerSorted[arg0]
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:437`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:246-247`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:246-247`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:246-247` (no engine call site found)
pub struct CgGetSortedForcePower;

impl InboundVmCall for CgGetSortedForcePower {
    type Command = MpCgameExport;
    type Args = c_int;
    type Output = c_int;

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_SORTED_FORCE_POWER;
}

impl DecodeVmMain for CgGetSortedForcePower {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        word_to_c_int(transport.arg(0))
    }
}

impl EncodeVmMainReturn for CgGetSortedForcePower {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
