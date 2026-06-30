use super::super::MpCgameExport;
use crate::abi::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};
use crate::shared::qhandle_t;

/// `CG_GET_MODEL_LIST` MP cgame exports vmMain ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:400`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:236-237`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:236-237`
/// Transport/call-site source: no engine call-site found in initial search; module vmMain switch proves no arg slots.
pub struct CgGetModelList;

impl InboundVmCall for CgGetModelList {
    type Command = MpCgameExport;
    type Args = ();
    type Output = *mut qhandle_t;

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_MODEL_LIST;
}

impl DecodeVmMain for CgGetModelList {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgGetModelList {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
