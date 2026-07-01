use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_CM_NUMINLINEMODELS`.
///
/// `trap_CM_NumInlineModels` takes no arguments; the transport carries only the
/// syscall token.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:127`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:128`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:781`
#[derive(Debug, Default)]
pub struct CgCmNuminlinemodelsArgs;

impl CgCmNuminlinemodelsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_CM_NUMINLINEMODELS` MP cgame imports syscall ABI token.
///
/// C signature: `int trap_CM_NumInlineModels(void)`.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:84`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:127`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:127`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:128`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:782`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:781`
pub struct CgCmNuminlinemodels;

impl OutboundSysCall for CgCmNuminlinemodels {
    type Import = MpCgameImport;
    type Args = CgCmNuminlinemodelsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_NUMINLINEMODELS;
}

impl EncodeSysCall for CgCmNuminlinemodels {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgCmNuminlinemodels {
    // `trap_CM_NumInlineModels` returns `int`; the engine's return word is that value.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
