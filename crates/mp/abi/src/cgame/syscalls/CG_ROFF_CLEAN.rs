use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_ROFF_CLEAN`.
///
/// Raven wrapper: `return syscall( CG_ROFF_CLEAN );`
/// Raven transport: `return theROFFSystem.Clean(qtrue);`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:730-732`
/// Args source: `oracle/codemp/cgame/cg_local.h:2430`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1268-1269`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgRoffCleanArgs;

impl CgRoffCleanArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_ROFF_CLEAN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:242`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:730-732`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:730-732`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1268-1269`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1268-1269`
pub struct CgRoffClean;

impl OutboundSysCall for CgRoffClean {
    type Import = MpCgameImport;
    type Args = CgRoffCleanArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_CLEAN;
}

impl EncodeSysCall for CgRoffClean {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgRoffClean {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
