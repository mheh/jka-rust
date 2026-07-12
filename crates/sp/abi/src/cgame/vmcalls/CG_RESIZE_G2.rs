use core::ffi::c_int;

use super::super::{types::CGhoul2Info_v, SpCgameExport};
use abi_transport::generic::{
    word_to_c_int, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// Arguments for `CG_RESIZE_G2`.
///
/// Raven vmMain: `CG_ResizeG2((CGhoul2Info_v *)arg0, arg1);`
///
/// Args source: `oracle/code/cgame/cg_main.cpp:119`
/// Type definition source: `oracle/code/game/ghoul2_shared.h:311`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgResizeG2Args {
    ghoul2: *mut CGhoul2Info_v,
    new_count: c_int,
}

impl CgResizeG2Args {
    pub const fn new(ghoul2: *mut CGhoul2Info_v, new_count: c_int) -> Self {
        Self { ghoul2, new_count }
    }

    pub const fn ghoul2(self) -> *mut CGhoul2Info_v {
        self.ghoul2
    }

    pub const fn new_count(self) -> c_int {
        self.new_count
    }
}

/// `CG_RESIZE_G2` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_ResizeG2( CGhoul2Info_v *ghoul2, int newCount );`
/// Enum value source: `oracle/code/client/vmachine.h:26`
/// Args source: `oracle/code/cgame/cg_main.cpp:40`, `oracle/code/cgame/cg_main.cpp:119`
/// Output source: `oracle/code/cgame/cg_main.cpp:119`
/// VM_Main switch source: `oracle/code/cgame/cg_main.cpp:118-130`
/// Raven: `CGhoul2Info_v` is defined in `game/ghoul2_shared.h` as `class CGhoul2Info_v`
/// at `oracle/code/game/ghoul2_shared.h:311`.
pub struct CgResizeG2;

impl InboundVmCall for CgResizeG2 {
    type Command = SpCgameExport;
    type Args = CgResizeG2Args;
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2;
}

impl DecodeVmMain for CgResizeG2 {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgResizeG2Args::new(
            word_to_mut_ptr(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgResizeG2 {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
