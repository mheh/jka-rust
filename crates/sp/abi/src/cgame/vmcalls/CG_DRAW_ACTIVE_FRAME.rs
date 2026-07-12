use core::ffi::c_int;

use super::super::{types::stereoFrame_t, SpCgameExport};
use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// Arguments for `CG_DRAW_ACTIVE_FRAME`.
///
/// Raven vmMain: `CG_DrawActiveFrame( arg0, (stereoFrame_t) arg1 );`
///
/// Args source: `oracle/code/cgame/cg_main.cpp:107`
/// Type definition source: `oracle/code/renderer/tr_types.h:183-187`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgDrawActiveFrameArgs {
    server_time: c_int,
    stereo_view: stereoFrame_t,
}

impl CgDrawActiveFrameArgs {
    pub const fn new(server_time: c_int, stereo_view: stereoFrame_t) -> Self {
        Self {
            server_time,
            stereo_view,
        }
    }

    pub const fn server_time(self) -> c_int {
        self.server_time
    }

    pub const fn stereo_view(self) -> stereoFrame_t {
        self.stereo_view
    }
}

/// `CG_DRAW_ACTIVE_FRAME` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_DrawActiveFrame( int serverTime, stereoFrame_t stereoView );`
/// Enum value source: `oracle/code/client/vmachine.h:17`
/// Args source: `oracle/code/cgame/cg_local.h:663`, `oracle/code/cgame/cg_main.cpp:107`
/// Output source: `oracle/code/cgame/cg_local.h:663`, `oracle/code/cgame/cg_main.cpp:107`
/// VM_Call/vmMain switch source: `oracle/code/client/cl_cgame.cpp:1109`, `oracle/code/cgame/cg_main.cpp:94-115`
pub struct CgDrawActiveFrame;

impl InboundVmCall for CgDrawActiveFrame {
    type Command = SpCgameExport;
    type Args = CgDrawActiveFrameArgs;
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_ACTIVE_FRAME;
}

impl DecodeVmMain for CgDrawActiveFrame {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgDrawActiveFrameArgs::new(
            word_to_c_int(transport.arg(0)),
            stereoFrame_t::from_wire(word_to_c_int(transport.arg(1))),
        )
    }
}

impl EncodeVmMainReturn for CgDrawActiveFrame {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
