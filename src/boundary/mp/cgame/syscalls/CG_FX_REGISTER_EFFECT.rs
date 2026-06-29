use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_REGISTER_EFFECT`.
///
/// Raven wrapper: `return syscall( CG_FX_REGISTER_EFFECT, file);`
/// Raven transport: `return FX_RegisterEffect((const char *)VMA(1));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:621-623`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2398`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1105-1106`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxRegisterEffectArgs {
    file: *const c_char,
}

impl CgFxRegisterEffectArgs {
    pub const fn new(file: *const c_char) -> Self {
        Self { file }
    }
}

/// `CG_FX_REGISTER_EFFECT` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:219`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:621-623`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1105-1106`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1105-1106`
pub struct CgFxRegisterEffect;

impl OutboundSysCall for CgFxRegisterEffect {
    type Import = MpCgameImport;
    type Args = CgFxRegisterEffectArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_REGISTER_EFFECT;
}

impl EncodeSysCall for CgFxRegisterEffect {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.file)])
    }
}

impl DecodeSysCallReturn for CgFxRegisterEffect {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
