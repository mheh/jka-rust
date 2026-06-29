use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_GET_ENTITY_TOKEN`.
///
/// Raven wrapper: `return syscall( CG_GET_ENTITY_TOKEN, buffer, bufferSize );`
/// Raven transport: `return re.GetEntityToken( (char *)VMA(1), args[2] );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:612-613`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2393`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1093-1094`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgGetEntityTokenArgs {
    buffer: *mut c_char,
    buffer_size: c_int,
}

impl CgGetEntityTokenArgs {
    pub const fn new(buffer: *mut c_char, buffer_size: c_int) -> Self {
        Self {
            buffer,
            buffer_size,
        }
    }
}

/// `CG_GET_ENTITY_TOKEN` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:216`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:612-613`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:612-613`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1093-1094`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1093-1094`
pub struct CgGetEntityToken;

impl OutboundSysCall for CgGetEntityToken {
    type Import = MpCgameImport;
    type Args = CgGetEntityTokenArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_GET_ENTITY_TOKEN;
}

impl EncodeSysCall for CgGetEntityToken {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buffer), args.buffer_size as isize])
    }
}

impl DecodeSysCallReturn for CgGetEntityToken {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
