use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::abi::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// `CG_AUTOMAP_INPUT` MP cgame exports vmMain ABI token.
///
/// Raven: special input during automap mode -rww
/// Raven: shared-buffer payload is `autoMapInput_t` (`mMode` is transport arg0)
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:433`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:442-449`
/// Args/source source: `oracle/oracle/codemp/cgame/cg_main.c:314-340`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:314-340`
/// Call-site source: `oracle/oracle/codemp/client/cl_input.cpp:635-641`
/// Call-site source: `oracle/oracle/codemp/client/cl_input.cpp:995-1001`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:340`
pub struct CgAutomapInput;

/// `CG_AUTOMAP_INPUT` transport arg.
///
/// Engine sends this call with `arg0=0` for map/keyboard style updates and
/// `arg0!=0` for mouse motion events.
/// FIXME: create type `autoMapInput_t` in Rust.
/// - `oracle/oracle/codemp/cgame/cg_public.h:442-449`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgAutomapInputArgs {
    mode: c_int,
}

impl CgAutomapInputArgs {
    pub const fn new(mode: c_int) -> Self {
        Self { mode }
    }

    pub const fn mode(self) -> c_int {
        self.mode
    }
}

impl InboundVmCall for CgAutomapInput {
    type Command = MpCgameExport;
    type Args = CgAutomapInputArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_AUTOMAP_INPUT;
}

impl DecodeVmMain for CgAutomapInput {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgAutomapInputArgs::new(word_to_c_int(transport.arg(0)))
    }
}

impl EncodeVmMainReturn for CgAutomapInput {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
