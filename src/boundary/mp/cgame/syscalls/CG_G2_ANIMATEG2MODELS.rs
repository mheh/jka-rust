use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::sharedRagDollUpdateParams_t;

/// Arguments for `CG_G2_ANIMATEG2MODELS`.
///
/// Raven wrapper: `syscall(CG_G2_ANIMATEG2MODELS, ghoul2, time, params);`
/// Raven transport: `sharedRagDollUpdateParams_t *rduParamst = (sharedRagDollUpdateParams_t *)VMA(3);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1003-1005`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2571`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1571-1591`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2Animateg2modelsArgs {
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedRagDollUpdateParams_t,
}

impl CgG2Animateg2modelsArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    ) -> Self {
        Self {
            ghoul2,
            time,
            params,
        }
    }
}

/// `CG_G2_ANIMATEG2MODELS` MP cgame imports syscall boundary token.
///
/// Raven: rww - RAGDOLL_BEGIN
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:297-301`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1003-1005`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1571-1591`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1571-1591`
pub struct CgG2Animateg2models;

impl OutboundSysCall for CgG2Animateg2models {
    type Import = MpCgameImport;
    type Args = CgG2Animateg2modelsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ANIMATEG2MODELS;
}

impl EncodeSysCall for CgG2Animateg2models {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.time as isize,
            ptr_to_word(args.params),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Animateg2models {
    fn decode_return(_word: isize) -> Self::Output {}
}
