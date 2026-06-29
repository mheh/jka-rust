use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_GETDEFAULTSTATE`.
///
/// Raven wrapper: `return syscall( CG_GETDEFAULTSTATE, entityIndex, state );`
/// Raven transport: `return CL_GetDefaultState(args[1], (entityState_t *)VMA(2));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:458-460`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:761-762`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgGetdefaultstateArgs {
    entity_index: c_int,
    state: *mut c_void,
}

impl CgGetdefaultstateArgs {
    pub const fn new(entity_index: c_int, state: *mut c_void) -> Self {
        Self {
            entity_index,
            state,
        }
    }
}

/// `CG_GETDEFAULTSTATE` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:155`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:458-460`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:761-762`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:761-762`
pub struct CgGetdefaultstate;

impl OutboundSysCall for CgGetdefaultstate {
    type Import = SpCgameImport;
    type Args = CgGetdefaultstateArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETDEFAULTSTATE;
}

impl EncodeSysCall for CgGetdefaultstate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.entity_index as isize, ptr_to_word(args.state)])
    }
}

impl DecodeSysCallReturn for CgGetdefaultstate {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
