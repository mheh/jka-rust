use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::pc_token_t;

/// Arguments for `CG_PC_READ_TOKEN`.
///
/// Raven wrapper: `return syscall( CG_PC_READ_TOKEN, handle, pc_token );`
/// Raven transport: `return botlib_export->PC_ReadTokenHandle( args[1], (struct pc_token_s *)VMA(2) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:553-554`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1005-1006`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPcReadTokenArgs {
    handle: c_int,
    pc_token: *mut pc_token_t,
}

impl CgPcReadTokenArgs {
    pub const fn new(handle: c_int, pc_token: *mut pc_token_t) -> Self {
        Self { handle, pc_token }
    }
}

/// `CG_PC_READ_TOKEN` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:202`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:553-554`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1005-1006`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1005-1006`
pub struct CgPcReadToken;

impl OutboundSysCall for CgPcReadToken {
    type Import = MpCgameImport;
    type Args = CgPcReadTokenArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_READ_TOKEN;
}

impl EncodeSysCall for CgPcReadToken {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle as isize, ptr_to_word(args.pc_token)])
    }
}

impl DecodeSysCallReturn for CgPcReadToken {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
