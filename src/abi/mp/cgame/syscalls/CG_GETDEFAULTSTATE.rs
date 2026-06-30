use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::entityState_t;
use crate::shared::qboolean;

/// Arguments for `CG_GETDEFAULTSTATE`.
///
/// Raven: rwwRMG - added [NEWTRAP].
/// Raven wrapper: `return syscall( CG_GETDEFAULTSTATE, entityIndex, state );`
/// Raven transport: `return CL_GetDefaultState(args[1], (entityState_t *)VMA(2));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:476-479`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2332`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:965-966`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgGetdefaultstateArgs {
    entity_index: c_int,
    state: *mut entityState_t,
}

impl CgGetdefaultstateArgs {
    pub const fn new(entity_index: c_int, state: *mut entityState_t) -> Self {
        Self {
            entity_index,
            state,
        }
    }
}

/// `CG_GETDEFAULTSTATE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:183`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:476-479`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:965-966`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:965-966`
pub struct CgGetdefaultstate;

impl OutboundSysCall for CgGetdefaultstate {
    type Import = MpCgameImport;
    type Args = CgGetdefaultstateArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETDEFAULTSTATE;
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
