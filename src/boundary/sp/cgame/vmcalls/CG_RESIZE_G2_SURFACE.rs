use core::ffi::{c_int, c_void};

use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RESIZE_G2_SURFACE` SP cgame exports vmMain boundary token.
///
/// Raven: `void CG_ResizeG2Surface( surfaceInfo_v *surface, int newCount );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:28`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:41`, `oracle/oracle/code/cgame/cg_main.cpp:128`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:128`
/// VM_Main switch source: `oracle/oracle/code/cgame/cg_main.cpp:118-130`
/// Raven: `surfaceInfo_v` is a vector type in `game/ghoul2_shared.h:201`.
pub struct CgResizeG2Surface;

impl InboundVmCall for CgResizeG2Surface {
    type Command = SpCgameExport;
    /// FIXME: create type `surfaceInfo_v` in Rust (Raven source: `oracle/oracle/code/game/ghoul2_shared.h:201`).
    type Args = (*mut c_void, c_int);
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_SURFACE;
}
