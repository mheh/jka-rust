use super::super::SpCgameImport;
use super::super::types::clipHandle_t;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_CM_INLINEMODEL`.
///
/// Raven wrapper: `return syscall( CG_CM_INLINEMODEL, index );`
/// Raven transport: `return CM_InlineModel( args[1] );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:139-141`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:531-532`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmInlinemodelArgs {
    index: i32,
}

impl CgCmInlinemodelArgs {
    pub const fn new(index: i32) -> Self {
        Self { index }
    }
}

/// `CG_CM_INLINEMODEL` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:83`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:139-141`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:531-532`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:531-532`
/// Type definition source: `oracle/oracle/code/game/q_shared.h:188`
pub struct CgCmInlinemodel;

impl OutboundSysCall for CgCmInlinemodel {
    type Import = SpCgameImport;
    type Args = CgCmInlinemodelArgs;
    type Output = clipHandle_t;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_INLINEMODEL;
}

impl EncodeSysCall for CgCmInlinemodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.index as isize])
    }
}

impl DecodeSysCallReturn for CgCmInlinemodel {
    fn decode_return(word: isize) -> Self::Output {
        word as clipHandle_t
    }
}
