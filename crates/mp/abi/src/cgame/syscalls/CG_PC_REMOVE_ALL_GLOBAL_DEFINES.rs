use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_PC_REMOVE_ALL_GLOBAL_DEFINES`.
///
/// Raven wrapper: `syscall ( CG_PC_REMOVE_ALL_GLOBAL_DEFINES );`
/// Raven transport: `botlib_export->PC_RemoveAllGlobalDefines ( ); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:566-568`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1011-1013`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgPcRemoveAllGlobalDefinesArgs;

impl CgPcRemoveAllGlobalDefinesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_PC_REMOVE_ALL_GLOBAL_DEFINES` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:205`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:566-568`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1011-1013`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1011-1013`
pub struct CgPcRemoveAllGlobalDefines;

impl OutboundSysCall for CgPcRemoveAllGlobalDefines {
    type Import = MpCgameImport;
    type Args = CgPcRemoveAllGlobalDefinesArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_REMOVE_ALL_GLOBAL_DEFINES;
}

impl EncodeSysCall for CgPcRemoveAllGlobalDefines {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgPcRemoveAllGlobalDefines {
    fn decode_return(_word: isize) -> Self::Output {}
}
