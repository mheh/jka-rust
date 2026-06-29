use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_WORLDEFFECTCOMMAND`.
///
/// Raven wrapper: `syscall(CG_R_WORLDEFFECTCOMMAND, cmd);`
/// Raven transport: `R_WorldEffectCommand((const char *)VMA(1)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1110-1113`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2427`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1720-1722`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRWorldeffectcommandArgs {
    cmd: *const c_char,
}

impl CgRWorldeffectcommandArgs {
    pub const fn new(cmd: *const c_char) -> Self {
        Self { cmd }
    }
}

/// `CG_R_WORLDEFFECTCOMMAND` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:334`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1110-1113`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1720-1722`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1720-1722`
pub struct CgRWorldeffectcommand;

impl OutboundSysCall for CgRWorldeffectcommand {
    type Import = MpCgameImport;
    type Args = CgRWorldeffectcommandArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_WORLDEFFECTCOMMAND;
}

impl EncodeSysCall for CgRWorldeffectcommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.cmd)])
    }
}

impl DecodeSysCallReturn for CgRWorldeffectcommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
