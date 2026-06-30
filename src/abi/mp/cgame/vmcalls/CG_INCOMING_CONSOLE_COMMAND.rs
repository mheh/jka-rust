use crate::shared::qboolean;

use super::super::MpCgameExport;
use crate::abi::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `CG_INCOMING_CONSOLE_COMMAND` MP cgame exports vmMain ABI token.
///
/// Raven: TCGIncomingConsoleCommand shares `conCommand` through `cg.sharedBuffer`
/// Raven: so the client can filter or rewrite local console commands before execution.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:414`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:506-509`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:259-280`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:259-280`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_keys.cpp:844`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_keys.cpp:1629`
pub struct CgIncomingConsoleCommand;

impl InboundVmCall for CgIncomingConsoleCommand {
    type Command = MpCgameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpCgameExport = MpCgameExport::CG_INCOMING_CONSOLE_COMMAND;
}

impl DecodeVmMain for CgIncomingConsoleCommand {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgIncomingConsoleCommand {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
