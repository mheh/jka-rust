use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_LISTBONES`.
///
/// Raven wrapper: `syscall( CG_G2_LISTBONES, ghlInfo, frame );`
/// Raven transport: `G2API_ListBones( (CGhoul2Info *) args[1], args[2]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:787-788`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:787-789`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2ListbonesArgs {
    ghl_info: *mut c_void,
    frame: c_int,
}

impl CgG2ListbonesArgs {
    pub const fn new(ghl_info: *mut c_void, frame: c_int) -> Self {
        Self { ghl_info, frame }
    }
}

/// `CG_G2_LISTBONES` SP cgame imports syscall ABI token.
///
/// Raven: Ghoul2 Insert Start
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:172`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:787-788`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:787-789`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:787-789`
pub struct CgG2Listbones;

impl OutboundSysCall for CgG2Listbones {
    type Import = SpCgameImport;
    type Args = CgG2ListbonesArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_LISTBONES;
}

impl EncodeSysCall for CgG2Listbones {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info), args.frame as isize])
    }
}

impl DecodeSysCallReturn for CgG2Listbones {
    fn decode_return(_word: isize) -> Self::Output {}
}
