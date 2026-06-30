use crate::shared::qboolean;

use super::super::MpCgameExport;
use crate::abi::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `CG_GET_USEABLE_FORCE` MP cgame exports vmMain ABI token.
///
/// Raven: qboolean CG_NoUseableForce(void)
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:416`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:282-283`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:282-283`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_input.cpp:1298`
pub struct CgGetUseableForce;

impl InboundVmCall for CgGetUseableForce {
    type Command = MpCgameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_USEABLE_FORCE;
}

impl DecodeVmMain for CgGetUseableForce {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgGetUseableForce {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
