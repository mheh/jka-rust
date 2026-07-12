use core::ffi::c_int;

use super::super::{types::mdxaBone_v, SpCgameExport};
use abi_transport::generic::{
    word_to_c_int, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// Arguments for `CG_RESIZE_G2_TEMPBONE`.
///
/// Raven vmMain: `CG_ResizeG2TempBone((mdxaBone_v *)arg0, arg1);`
///
/// Args source: `oracle/code/cgame/cg_main.cpp:130-131`
/// Type definition source: `oracle/code/game/ghoul2_shared.h:204`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgResizeG2TempboneArgs {
    temp_bone: *mut mdxaBone_v,
    new_count: c_int,
}

impl CgResizeG2TempboneArgs {
    pub const fn new(temp_bone: *mut mdxaBone_v, new_count: c_int) -> Self {
        Self {
            temp_bone,
            new_count,
        }
    }

    pub const fn temp_bone(self) -> *mut mdxaBone_v {
        self.temp_bone
    }

    pub const fn new_count(self) -> c_int {
        self.new_count
    }
}

/// `CG_RESIZE_G2_TEMPBONE` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_ResizeG2TempBone( mdxaBone_v *tempBone, int newCount );`
/// Enum value source: `oracle/code/client/vmachine.h:29`
/// Args source: `oracle/code/cgame/cg_main.cpp:44`, `oracle/code/cgame/cg_main.cpp:130-131`
/// Output source: `oracle/code/cgame/cg_main.cpp:202-204`
/// VM_Main switch source: `oracle/code/cgame/cg_main.cpp:118-132`
/// Raven: `mdxaBone_v` is a vector type in `oracle/code/game/ghoul2_shared.h:204`.
pub struct CgResizeG2Tempbone;

impl InboundVmCall for CgResizeG2Tempbone {
    type Command = SpCgameExport;
    type Args = CgResizeG2TempboneArgs;
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_TEMPBONE;
}

impl DecodeVmMain for CgResizeG2Tempbone {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgResizeG2TempboneArgs::new(
            word_to_mut_ptr(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgResizeG2Tempbone {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
