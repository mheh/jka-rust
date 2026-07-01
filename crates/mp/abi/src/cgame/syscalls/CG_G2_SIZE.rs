use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_SIZE`.
///
/// Raven wrapper: `int trap_G2API_Ghoul2Size(void* ghlInfo)`.
/// Raven transport: `return G2API_Ghoul2Size(*((CGhoul2Info_v *)args[1]));`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:282`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:935-937`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1471-1473`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1471-1473`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SizeArgs {
    /// Raw Ghoul2 handle word, decoded by Raven as `args[1]`.
    ghl_info: *mut c_void,
}

impl CgG2SizeArgs {
    pub const fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }
}

/// `CG_G2_SIZE` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghlInfo` is passed as a raw `args[1]` pointer word and
/// the switch returns the int size directly.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:282`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:935-937`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1471-1473`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1471-1473`
pub struct CgG2Size;

impl OutboundSysCall for CgG2Size {
    type Import = MpCgameImport;
    type Args = CgG2SizeArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SIZE;
}

impl EncodeSysCall for CgG2Size {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info)])
    }
}

impl DecodeSysCallReturn for CgG2Size {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
