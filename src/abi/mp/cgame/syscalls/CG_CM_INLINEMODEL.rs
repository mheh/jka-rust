use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_CM_INLINEMODEL`.
///
/// C ABI: `clipHandle_t trap_CM_InlineModel(int index)`.
/// Raven's wrapper forwards `index` as the only payload word, and the client
/// switch reads it from `args[1]`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:131-132`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:783-784`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgCmInlinemodelArgs {
    /// Inline collision model index, read by Raven as `args[1]`.
    index: c_int,
}

impl CgCmInlinemodelArgs {
    pub const fn new(index: c_int) -> Self {
        Self { index }
    }

    pub const fn index(&self) -> c_int {
        self.index
    }
}

/// `CG_CM_INLINEMODEL` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall( CG_CM_INLINEMODEL, index );`
/// Raven transport: `return CM_InlineModel( args[1] );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:85`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:131-132`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:131-132`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:783-784`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:783-784`
pub struct CgCmInlinemodel;

impl OutboundSysCall for CgCmInlinemodel {
    type Import = MpCgameImport;
    type Args = CgCmInlinemodelArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_INLINEMODEL;
}

impl EncodeSysCall for CgCmInlinemodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.index() as isize])
    }
}

impl DecodeSysCallReturn for CgCmInlinemodel {
    // `clipHandle_t` is an int-compatible Raven handle returned in the syscall word.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
