use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_REGISTERSKIN`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERSKIN, name );`
/// Raven transport: `return re.RegisterSkin( (const char *)VMA(1) );`
///
/// Raven comment: `returns all white if not found`.
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:270-271`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2250`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:865-866`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegisterskinArgs {
    name: *const c_char,
}

impl CgRRegisterskinArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_R_REGISTERSKIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:118`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:270-271`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2250`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:865-866`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:865-866`
pub struct CgRRegisterskin;

impl OutboundSysCall for CgRRegisterskin {
    type Import = MpCgameImport;
    type Args = CgRRegisterskinArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERSKIN;
}

impl EncodeSysCall for CgRRegisterskin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgRRegisterskin {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
