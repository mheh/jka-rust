use core::ffi::c_int;

use super::super::shared_buffer::{autoMapInput_t, SharedBufferPayload};
use super::super::MpCgameExport;
use crate::abi::generic::{EncodeVmMainReturn, InboundVmCall};

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

/// `CG_AUTOMAP_INPUT` transport arg plus shared-buffer payload.
///
/// Engine sends this call with `arg0=0` for map/keyboard style updates and
/// `arg0!=0` for mouse motion events.
///
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:442-449`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CgAutomapInputArgs {
    mode: c_int,
    payload: SharedBufferPayload<autoMapInput_t>,
}

impl CgAutomapInputArgs {
    pub const fn new(mode: c_int, payload: SharedBufferPayload<autoMapInput_t>) -> Self {
        Self { mode, payload }
    }

    pub const fn mode(self) -> c_int {
        self.mode
    }

    pub const fn payload(self) -> SharedBufferPayload<autoMapInput_t> {
        self.payload
    }
}

impl InboundVmCall for CgAutomapInput {
    type Command = MpCgameExport;
    type Args = CgAutomapInputArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_AUTOMAP_INPUT;
}

impl EncodeVmMainReturn for CgAutomapInput {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
