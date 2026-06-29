use super::super::MpCgameExport;
use crate::boundary::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `CG_SHUTDOWN` MP cgame exports vmMain boundary token.
///
/// Raven: void (*CG_Shutdown)( void );
/// Raven: opportunity to flush and close any open files
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:362-364`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:196-198`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:196-198`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_cgame.cpp:595-602`
pub struct CgShutdown;

impl InboundVmCall for CgShutdown {
    type Command = MpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_SHUTDOWN;
}

impl DecodeVmMain for CgShutdown {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgShutdown {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
