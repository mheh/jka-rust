use core::ffi::{c_int, c_void};

use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RESIZE_G2_TEMPBONE` SP cgame exports vmMain boundary token.
///
/// Raven: `void CG_ResizeG2TempBone( mdxaBone_v *tempBone, int newCount );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:29`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:44`, `oracle/oracle/code/cgame/cg_main.cpp:130-131`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:202-204`
/// VM_Main switch source: `oracle/oracle/code/cgame/cg_main.cpp:118-132`
/// Raven: `mdxaBone_v` is a vector type in `oracle/oracle/code/game/ghoul2_shared.h:204`.
pub struct CgResizeG2Tempbone;

impl InboundVmCall for CgResizeG2Tempbone {
    type Command = SpCgameExport;
    /// FIXME: create type `mdxaBone_v` in Rust (Raven source: `oracle/oracle/code/game/ghoul2_shared.h:204`).
    type Args = (*mut c_void, c_int);
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_TEMPBONE;
}
