use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::abi::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_DRAW_ACTIVE_FRAME`.
///
/// `stereoFrame_t` is a Raven `typedef int` in `oracle/oracle/codemp/cgame/tr_types.h:283`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgDrawActiveFrameArgs {
    server_time: c_int,
    stereo_view: c_int,
    demo_playback: qboolean,
}

impl CgDrawActiveFrameArgs {
    pub const fn new(server_time: c_int, stereo_view: c_int, demo_playback: qboolean) -> Self {
        Self {
            server_time,
            stereo_view,
            demo_playback,
        }
    }

    pub const fn server_time(self) -> c_int {
        self.server_time
    }

    pub const fn stereo_view(self) -> c_int {
        self.stereo_view
    }

    pub const fn demo_playback(self) -> qboolean {
        self.demo_playback
    }
}

/// `CG_DRAW_ACTIVE_FRAME` MP cgame exports vmMain ABI token.
///
/// Raven: void (*CG_DrawActiveFrame)( int serverTime, stereoFrame_t stereoView, qboolean demoPlayback );
/// Raven: Generates and draws a game scene and status information at the given time.
/// Raven: If demoPlayback is set, local movement prediction will not be enabled
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:373-376`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:201-203`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:201-203`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_cgame.cpp:1830-1843`
pub struct CgDrawActiveFrame;

impl InboundVmCall for CgDrawActiveFrame {
    type Command = MpCgameExport;
    type Args = CgDrawActiveFrameArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_DRAW_ACTIVE_FRAME;
}

impl DecodeVmMain for CgDrawActiveFrame {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgDrawActiveFrameArgs::new(
            word_to_c_int(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
            word_to_c_int(transport.arg(2)),
        )
    }
}

impl EncodeVmMainReturn for CgDrawActiveFrame {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
