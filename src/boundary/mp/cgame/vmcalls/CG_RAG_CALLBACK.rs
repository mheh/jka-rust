use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::boundary::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// Arguments for `CG_RAG_CALLBACK`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:256-257`
/// Shared-buffer callback data source: `oracle/oracle/codemp/cgame/cg_main.c:507-566`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgRagCallbackArgs {
    call_type: c_int,
}

impl CgRagCallbackArgs {
    pub const fn new(call_type: c_int) -> Self {
        Self { call_type }
    }

    pub const fn call_type(self) -> c_int {
        self.call_type
    }
}

/// `CG_RAG_CALLBACK` MP cgame exports vmMain boundary token.
///
/// Raven: handle ragdoll callbacks, for events and debugging -rww
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:412`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:256-257`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:256-257`
/// Transport/call-site source: `oracle/oracle/codemp/ghoul2/G2_bones.cpp:2700`
/// Transport/call-site source: `oracle/oracle/codemp/ghoul2/G2_bones.cpp:2899`
/// Transport/call-site source: `oracle/oracle/codemp/ghoul2/G2_bones.cpp:2921`
/// Transport/call-site source: `oracle/oracle/codemp/ghoul2/G2_bones.cpp:3064`
/// Transport/call-site source: `oracle/oracle/codemp/ghoul2/G2_bones.cpp:3966`
pub struct CgRagCallback;

impl InboundVmCall for CgRagCallback {
    type Command = MpCgameExport;
    type Args = CgRagCallbackArgs;
    type Output = c_int;

    const COMMAND: MpCgameExport = MpCgameExport::CG_RAG_CALLBACK;
}

impl DecodeVmMain for CgRagCallback {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgRagCallbackArgs::new(word_to_c_int(transport.arg(0)))
    }
}

impl EncodeVmMainReturn for CgRagCallback {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
