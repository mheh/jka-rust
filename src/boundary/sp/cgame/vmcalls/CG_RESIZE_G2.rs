use core::ffi::{c_int, c_void};

use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RESIZE_G2` SP cgame exports vmMain boundary token.
///
/// Raven: `void CG_ResizeG2( CGhoul2Info_v *ghoul2, int newCount );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:26`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:40`, `oracle/oracle/code/cgame/cg_main.cpp:119`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:119`
/// VM_Main switch source: `oracle/oracle/code/cgame/cg_main.cpp:118-130`
/// Raven: `CGhoul2Info_v` is defined in `game/ghoul2_shared.h` as `class CGhoul2Info_v`
/// at `oracle/oracle/code/game/ghoul2_shared.h:311`.
pub struct CgResizeG2;

impl InboundVmCall for CgResizeG2 {
    type Command = SpCgameExport;
    /// FIXME: create type `CGhoul2Info_v` in Rust (Raven source: `oracle/oracle/code/game/ghoul2_shared.h:311`).
    type Args = (*mut c_void, c_int);
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2;
}
