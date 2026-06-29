use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::sharedIKMoveParams_t;
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_IKMOVE`.
///
/// Raven wrapper: `return syscall(CG_G2_IKMOVE, ghoul2, time, params);`
/// Raven transport: `return G2API_IKMove(*((CGhoul2Info_v *)args[1]), args[2], (sharedIKMoveParams_t *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1045-1047`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2582`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1610-1611`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2IkmoveArgs {
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedIKMoveParams_t,
}

impl CgG2IkmoveArgs {
    pub const fn new(ghoul2: *mut c_void, time: c_int, params: *mut sharedIKMoveParams_t) -> Self {
        Self {
            ghoul2,
            time,
            params,
        }
    }
}

/// `CG_G2_IKMOVE` MP cgame imports syscall boundary token.
///
/// Raven: rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
/// Raven: by using the majority of gil's existing bone angling stuff from the ragdoll code.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:317`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1045-1047`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1610-1611`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1610-1611`
pub struct CgG2Ikmove;

impl OutboundSysCall for CgG2Ikmove {
    type Import = MpCgameImport;
    type Args = CgG2IkmoveArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_IKMOVE;
}

impl EncodeSysCall for CgG2Ikmove {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.time as isize,
            ptr_to_word(args.params),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Ikmove {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
