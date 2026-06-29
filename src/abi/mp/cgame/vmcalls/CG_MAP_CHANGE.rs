use super::super::MpCgameExport;
use crate::abi::generic::{
    DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// `CG_MAP_CHANGE` MP cgame exports vmMain ABI token.
///
/// Raven: this trap map be called more than once for a given map change, as the
/// Raven: server is going to attempt to send out multiple broadcasts in hopes that
/// Raven: the client will receive one of them
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:431`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:307-312`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:307-312`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_parse.cpp:918-922`
pub struct CgMapChange;

impl InboundVmCall for CgMapChange {
    type Command = MpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_MAP_CHANGE;
}

impl DecodeVmMain for CgMapChange {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgMapChange {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
