use core::ffi::c_int;

use super::super::MpCgameExport;
use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// Arguments for `CG_MOUSE_EVENT`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:211-215`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_input.cpp:1005-1008`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgMouseEventArgs {
    dx: c_int,
    dy: c_int,
}

impl CgMouseEventArgs {
    pub const fn new(dx: c_int, dy: c_int) -> Self {
        Self { dx, dy }
    }

    pub const fn dx(self) -> c_int {
        self.dx
    }

    pub const fn dy(self) -> c_int {
        self.dy
    }
}

/// `CG_MOUSE_EVENT` MP cgame exports vmMain ABI token.
///
/// Raven: void (*CG_MouseEvent)( int dx, int dy );
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:387-388`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:211-215`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:211-215`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_input.cpp:1005-1008`
pub struct CgMouseEvent;

impl InboundVmCall for CgMouseEvent {
    type Command = MpCgameExport;
    type Args = CgMouseEventArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_MOUSE_EVENT;
}

impl DecodeVmMain for CgMouseEvent {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgMouseEventArgs::new(
            word_to_c_int(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgMouseEvent {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
