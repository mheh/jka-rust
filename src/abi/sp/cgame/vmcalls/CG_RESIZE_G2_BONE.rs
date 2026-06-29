use core::ffi::{c_int, c_void};

use super::super::SpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_RESIZE_G2_BONE` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_ResizeG2Bone( boneInfo_v *bone, int newCount );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:27`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:42`, `oracle/oracle/code/cgame/cg_main.cpp:125`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:125`
/// VM_Main switch source: `oracle/oracle/code/cgame/cg_main.cpp:118-130`
/// Raven: `boneInfo_v` is a vector type in `game/ghoul2_shared.h:202`.
pub struct CgResizeG2Bone;

impl InboundVmCall for CgResizeG2Bone {
    type Command = SpCgameExport;
    /// FIXME: create type `boneInfo_v` in Rust (Raven source: `oracle/oracle/code/game/ghoul2_shared.h:202`).
    type Args = (*mut c_void, c_int);
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_BONE;
}
