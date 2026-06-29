use super::super::MpCgameExport;
use crate::boundary::generic::{EncodeVmMainReturn, InboundVmCall};

/// `CG_AUTOMAP_INPUT` MP cgame exports vmMain boundary token.
///
/// Raven: special input during automap mode -rww
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:433`
/// Args source: `oracle/oracle/codemp/cgame/cg_public.h:442-449`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:314-340`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:314-340`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_input.cpp:632-640`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_input.cpp:994-1000`
pub struct CgAutomapInput;

impl InboundVmCall for CgAutomapInput {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is autoMapInput_t in cg.sharedBuffer/cl.mSharedMemory plus arg0 mode.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_AUTOMAP_INPUT;
}

impl EncodeVmMainReturn for CgAutomapInput {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
