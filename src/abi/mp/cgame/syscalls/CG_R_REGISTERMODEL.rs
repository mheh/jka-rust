use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;

/// Arguments for `CG_R_REGISTERMODEL`.
///
/// Raven: returns rgb axis if not found.
/// Raven wrapper: `return syscall( CG_R_REGISTERMODEL, name );`
/// Raven transport: `return re.RegisterModel( (const char *)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:266-267`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2249`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:863-864`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegistermodelArgs {
    name: *const c_char,
}

impl CgRRegistermodelArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_R_REGISTERMODEL` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:117`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:266-267`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2249`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:863-864`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:863-864`
pub struct CgRRegistermodel;

impl OutboundSysCall for CgRRegistermodel {
    type Import = MpCgameImport;
    type Args = CgRRegistermodelArgs;
    type Output = qhandle_t;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERMODEL;
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
