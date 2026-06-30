use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qhandle_t;

/// Arguments for `CG_R_REGISTERSHADERNOMIP`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERSHADERNOMIP, name );`
/// Raven transport: `return re.RegisterShaderNoMip((const char *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:317-318`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:661-662`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegistershadernomipArgs {
    name: *const c_char,
}

impl CgRRegistershadernomipArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }

    pub const fn name(&self) -> *const c_char {
        self.name
    }
}

/// `CG_R_REGISTERSHADERNOMIP` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:121`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:317-318`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:661-662`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:661-662`
pub struct CgRRegistershadernomip;

impl OutboundSysCall for CgRRegistershadernomip {
    type Import = SpCgameImport;
    type Args = CgRRegistershadernomipArgs;
    type Output = qhandle_t;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_REGISTERSHADERNOMIP;
}

impl EncodeSysCall for CgRRegistershadernomip {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name())])
    }
}

impl DecodeSysCallReturn for CgRRegistershadernomip {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
