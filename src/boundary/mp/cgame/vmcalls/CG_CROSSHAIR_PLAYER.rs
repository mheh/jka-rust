use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::boundary::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `CG_CROSSHAIR_PLAYER` MP cgame exports vmMain boundary token.
///
/// Raven: int (*CG_CrosshairPlayer)( void );
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:378-379`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:204-205`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:204-205`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_console.cpp:76-85`
pub struct CgCrosshairPlayer;

impl InboundVmCall for CgCrosshairPlayer {
    type Command = MpCgameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpCgameExport = MpCgameExport::CG_CROSSHAIR_PLAYER;
}

impl DecodeVmMain for CgCrosshairPlayer {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgCrosshairPlayer {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
