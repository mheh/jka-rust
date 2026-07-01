use core::ffi::c_int;

use super::super::{types::surfaceInfo_v, SpCgameExport};
use abi_transport::generic::{
    word_to_c_int, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// Arguments for `CG_RESIZE_G2_SURFACE`.
///
/// Raven vmMain: `CG_ResizeG2Surface((surfaceInfo_v *)arg0, arg1);`
///
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:128`
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:201`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgResizeG2SurfaceArgs {
    surface: *mut surfaceInfo_v,
    new_count: c_int,
}

impl CgResizeG2SurfaceArgs {
    pub const fn new(surface: *mut surfaceInfo_v, new_count: c_int) -> Self {
        Self { surface, new_count }
    }

    pub const fn surface(self) -> *mut surfaceInfo_v {
        self.surface
    }

    pub const fn new_count(self) -> c_int {
        self.new_count
    }
}

/// `CG_RESIZE_G2_SURFACE` SP cgame exports vmMain ABI token.
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
    type Args = CgResizeG2SurfaceArgs;
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_SURFACE;
}

impl DecodeVmMain for CgResizeG2Surface {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgResizeG2SurfaceArgs::new(
            word_to_mut_ptr(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgResizeG2Surface {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
