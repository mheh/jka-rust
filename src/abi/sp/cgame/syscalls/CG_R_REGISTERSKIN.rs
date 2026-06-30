use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qhandle_t;

/// Arguments for `CG_R_REGISTERSKIN`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERSKIN, name );`
/// Raven transport: `return re.RegisterSkin( (const char *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:307-308`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:657-658`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegisterskinArgs {
    name: *const c_char,
}

impl CgRRegisterskinArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_R_REGISTERSKIN` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:119`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:307-308`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:657-658`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:657-658`
pub struct CgRRegisterskin;

impl OutboundSysCall for CgRRegisterskin {
    type Import = SpCgameImport;
    type Args = CgRRegisterskinArgs;
    type Output = qhandle_t;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_REGISTERSKIN;
}

impl EncodeSysCall for CgRRegisterskin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgRRegisterskin {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
