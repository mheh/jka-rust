use core::ffi::c_int;

use super::super::MpCgameExport;
use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// `CG_EVENT_HANDLING` MP cgame exports vmMain ABI token.
///
/// Raven: void (*CG_EventHandling)(int type);
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:389-390`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:216-218`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:216-218`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_keys.cpp:1529`
pub struct CgEventHandling;

impl InboundVmCall for CgEventHandling {
    type Command = MpCgameExport;
    type Args = c_int;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_EVENT_HANDLING;
}

impl DecodeVmMain for CgEventHandling {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        word_to_c_int(transport.arg(0))
    }
}

impl EncodeVmMainReturn for CgEventHandling {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
