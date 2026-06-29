use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;

/// `CG_R_REGISTERSHADER` SP cgame imports syscall boundary token.
///
/// Arguments for `CG_R_REGISTERSHADER`.
///
/// Raven wrapper: `qhandle_t hShader = syscall( CG_R_REGISTERSHADER, name );`
/// Raven transport: `return re.RegisterShader( (const char *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:311-314`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:659-660`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegistershaderArgs {
    name: *const c_char,
}

impl CgRRegistershaderArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_R_REGISTERSHADER` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:120`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:311-314`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:659-660`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:659-660`
/// Raven comment: `assert (hShader);`
pub struct CgRRegistershader;

impl OutboundSysCall for CgRRegistershader {
    type Import = SpCgameImport;
    type Args = CgRRegistershaderArgs;
    type Output = qhandle_t;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_REGISTERSHADER;
}

impl EncodeSysCall for CgRRegistershader {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgRRegistershader {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
