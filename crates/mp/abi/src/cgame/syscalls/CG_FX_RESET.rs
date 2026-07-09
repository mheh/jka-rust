use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_FX_RESET`.
///
/// Raven wrapper: `syscall ( CG_FX_RESET );`
/// Raven transport: `FX_Free ( false ); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:684-686`
/// Args source: `oracle/codemp/cgame/cg_local.h:2409`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1163-1165`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgFxResetArgs;

impl CgFxResetArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_FX_RESET` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:232`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:684-686`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1163-1165`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1163-1165`
pub struct CgFxReset;

impl OutboundSysCall for CgFxReset {
    type Import = MpCgameImport;
    type Args = CgFxResetArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_RESET;
}

impl EncodeSysCall for CgFxReset {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgFxReset {
    fn decode_return(_word: isize) -> Self::Output {}
}
