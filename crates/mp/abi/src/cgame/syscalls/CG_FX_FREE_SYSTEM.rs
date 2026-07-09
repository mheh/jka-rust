use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_FX_FREE_SYSTEM`.
///
/// Raven wrapper: `return syscall( CG_FX_FREE_SYSTEM );`
/// Raven transport: `return FX_FreeSystem();`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:679-681`
/// Args source: `oracle/codemp/cgame/cg_local.h:2407`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1156-1157`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgFxFreeSystemArgs;

impl CgFxFreeSystemArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_FX_FREE_SYSTEM` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:229`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:679-681`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1156-1157`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1156-1157`
pub struct CgFxFreeSystem;

impl OutboundSysCall for CgFxFreeSystem {
    type Import = MpCgameImport;
    type Args = CgFxFreeSystemArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_FREE_SYSTEM;
}

impl EncodeSysCall for CgFxFreeSystem {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgFxFreeSystem {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
