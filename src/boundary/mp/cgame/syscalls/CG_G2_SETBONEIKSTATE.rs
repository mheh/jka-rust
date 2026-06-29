use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::sharedSetBoneIKStateParams_t;
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_SETBONEIKSTATE`.
///
/// Raven wrapper: `return syscall(CG_G2_SETBONEIKSTATE, ghoul2, time, boneName, ikState, params);`
/// Raven transport: `return G2API_SetBoneIKState(*((CGhoul2Info_v *)args[1]), args[2], (const char *)VMA(3), args[4], (sharedSetBoneIKStateParams_t *)VMA(5));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1040-1042`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2581`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1608-1609`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetboneikstateArgs {
    ghoul2: *mut c_void,
    time: c_int,
    bone_name: *const c_char,
    ik_state: c_int,
    params: *mut sharedSetBoneIKStateParams_t,
}

impl CgG2SetboneikstateArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        time: c_int,
        bone_name: *const c_char,
        ik_state: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> Self {
        Self {
            ghoul2,
            time,
            bone_name,
            ik_state,
            params,
        }
    }
}

/// `CG_G2_SETBONEIKSTATE` MP cgame imports syscall boundary token.
///
/// Raven: rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
/// Raven: by using the majority of gil's existing bone angling stuff from the ragdoll code.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:316`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1040-1042`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1608-1609`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1608-1609`
pub struct CgG2Setboneikstate;

impl OutboundSysCall for CgG2Setboneikstate {
    type Import = MpCgameImport;
    type Args = CgG2SetboneikstateArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETBONEIKSTATE;
}

impl EncodeSysCall for CgG2Setboneikstate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.time as isize,
            ptr_to_word(args.bone_name),
            args.ik_state as isize,
            ptr_to_word(args.params),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Setboneikstate {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
