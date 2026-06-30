use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qhandle_t;

/// Arguments for `CG_R_REGISTERMODEL`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERMODEL, name );`
/// Raven transport: `return re.RegisterModel( (const char *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:303-304`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:655-656`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegistermodelArgs {
    name: *const c_char,
}

impl CgRRegistermodelArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_R_REGISTERMODEL` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:118`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:303-304`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:655-656`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:655-656`
pub struct CgRRegistermodel;

impl OutboundSysCall for CgRRegistermodel {
    type Import = SpCgameImport;
    type Args = CgRRegistermodelArgs;
    type Output = qhandle_t;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_REGISTERMODEL;
}

impl EncodeSysCall for CgRRegistermodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgRRegistermodel {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
