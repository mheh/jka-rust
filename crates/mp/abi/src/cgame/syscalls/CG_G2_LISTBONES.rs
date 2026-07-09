use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_LISTBONES`.
///
/// Raven wrapper: `syscall( CG_G2_LISTBONES, ghlInfo, frame);`
/// Raven transport: `G2API_ListBones( (CGhoul2Info *) args[1], args[2]);`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:776-778`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1300-1302`
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

/// `CG_G2_LISTBONES` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghlInfo` is passed as a raw `args[1]` pointer word, not VMA.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:257`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:776-778`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1300-1302`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1300-1302`
pub struct CgG2Listbones;

impl OutboundSysCall for CgG2Listbones {
    type Import = MpCgameImport;
    type Args = CgG2ListbonesArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_LISTBONES;
}

impl EncodeSysCall for CgG2Listbones {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info), args.frame as isize])
    }
}

impl DecodeSysCallReturn for CgG2Listbones {
    fn decode_return(_word: isize) -> Self::Output {}
}
