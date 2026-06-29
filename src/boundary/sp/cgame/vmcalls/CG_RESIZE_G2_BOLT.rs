use core::ffi::{c_int, c_void};

use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RESIZE_G2_BOLT` SP cgame exports vmMain boundary token.
///
/// Raven: Ghoul2 Insert Start
/// Raven: `void CG_ResizeG2Bolt( boltInfo_v *bolt, int newCount );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:25`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:40`, `oracle/oracle/code/cgame/cg_main.cpp:122`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:122`
/// VM_Main switch source: `oracle/oracle/code/cgame/cg_main.cpp:118-130`
/// Raven: `boltInfo_v` is a vector type in `game/ghoul2_shared.h:203`.
pub struct CgResizeG2Bolt;

impl InboundVmCall for CgResizeG2Bolt {
    type Command = SpCgameExport;
    /// FIXME: create type `boltInfo_v` in Rust (Raven source: `oracle/oracle/code/game/ghoul2_shared.h:203`).
    type Args = (*mut c_void, c_int);
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_BOLT;
}
