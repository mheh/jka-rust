use super::super::types::clipHandle_t;
use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_CM_TEMPBOXMODEL`.
///
/// Raven wrapper: `return syscall( CG_CM_TEMPBOXMODEL, mins, maxs );`
/// Raven transport: `return CM_TempBoxModel( (const float *) VMA(1), (const float *) VMA(2) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:143-145`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:533-534`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTempboxmodelArgs {
    mins: *const vec3_t,
    maxs: *const vec3_t,
}

impl CgCmTempboxmodelArgs {
    pub const fn new(mins: *const vec3_t, maxs: *const vec3_t) -> Self {
        Self { mins, maxs }
    }
}

/// `CG_CM_TEMPBOXMODEL` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:84`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:143-145`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:533-534`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:533-534`
/// Type definition source: `oracle/oracle/code/game/q_shared.h:188`
pub struct CgCmTempboxmodel;

impl OutboundSysCall for CgCmTempboxmodel {
    type Import = SpCgameImport;
    type Args = CgCmTempboxmodelArgs;
    type Output = clipHandle_t;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_TEMPBOXMODEL;
}

impl EncodeSysCall for CgCmTempboxmodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mins), ptr_to_word(args.maxs)])
    }
}

impl DecodeSysCallReturn for CgCmTempboxmodel {
    fn decode_return(word: isize) -> Self::Output {
        word as clipHandle_t
    }
}
