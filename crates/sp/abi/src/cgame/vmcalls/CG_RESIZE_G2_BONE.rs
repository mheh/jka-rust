use core::ffi::c_int;

use super::super::{types::boneInfo_v, SpCgameExport};
use abi_transport::generic::{
    word_to_c_int, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// Arguments for `CG_RESIZE_G2_BONE`.
///
/// Raven vmMain: `CG_ResizeG2Bone((boneInfo_v *)arg0, arg1);`
///
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:125`
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:202`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgResizeG2BoneArgs {
    bone: *mut boneInfo_v,
    new_count: c_int,
}

impl CgResizeG2BoneArgs {
    pub const fn new(bone: *mut boneInfo_v, new_count: c_int) -> Self {
        Self { bone, new_count }
    }

    pub const fn bone(self) -> *mut boneInfo_v {
        self.bone
    }

    pub const fn new_count(self) -> c_int {
        self.new_count
    }
}

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
    type Args = CgResizeG2BoneArgs;
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_BONE;
}

impl DecodeVmMain for CgResizeG2Bone {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgResizeG2BoneArgs::new(
            word_to_mut_ptr(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgResizeG2Bone {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
