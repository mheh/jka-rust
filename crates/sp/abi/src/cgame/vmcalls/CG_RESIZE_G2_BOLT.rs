use core::ffi::c_int;

use super::super::{types::boltInfo_v, SpCgameExport};
use abi_transport::generic::{
    word_to_c_int, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// Arguments for `CG_RESIZE_G2_BOLT`.
///
/// Raven vmMain: `CG_ResizeG2Bolt((boltInfo_v *)arg0, arg1);`
///
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:122`
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:203`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgResizeG2BoltArgs {
    bolt: *mut boltInfo_v,
    new_count: c_int,
}

impl CgResizeG2BoltArgs {
    pub const fn new(bolt: *mut boltInfo_v, new_count: c_int) -> Self {
        Self { bolt, new_count }
    }

    pub const fn bolt(self) -> *mut boltInfo_v {
        self.bolt
    }

    pub const fn new_count(self) -> c_int {
        self.new_count
    }
}

/// `CG_RESIZE_G2_BOLT` SP cgame exports vmMain ABI token.
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
    type Args = CgResizeG2BoltArgs;
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_RESIZE_G2_BOLT;
}

impl DecodeVmMain for CgResizeG2Bolt {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgResizeG2BoltArgs::new(
            word_to_mut_ptr(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgResizeG2Bolt {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
