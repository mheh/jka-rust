use core::ffi::c_int;

use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_ACTIVE_FRAME` SP cgame exports vmMain boundary token.
///
/// Raven: `void CG_DrawActiveFrame( int serverTime, stereoFrame_t stereoView );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:17`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:663`, `oracle/oracle/code/cgame/cg_main.cpp:107`
/// Output source: `oracle/oracle/code/cgame/cg_local.h:663`, `oracle/oracle/code/cgame/cg_main.cpp:107`
/// VM_Call/vmMain switch source: `oracle/oracle/code/client/cl_cgame.cpp:1109`, `oracle/oracle/code/cgame/cg_main.cpp:94-115`
pub struct CgDrawActiveFrame;

impl InboundVmCall for CgDrawActiveFrame {
    type Command = SpCgameExport;
    /// `stereoFrame_t` is an enum in C; `c_int` preserves the wire ABI.
    /// FIXME: create type `stereoFrame_t` in Rust (Raven source: `oracle/oracle/code/renderer/tr_types.h:183`).
    type Args = (c_int, c_int);
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_ACTIVE_FRAME;
}
