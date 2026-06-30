use core::ffi::{c_char, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;
use crate::shared::vec3_t;

/// Arguments for `CG_G2_RAGEFFECTORGOAL`.
///
/// Raven wrapper: `qboolean trap_G2API_RagEffectorGoal(void *ghoul2, const char *boneName, vec3_t pos)`
/// Raven transport: `return G2API_RagEffectorGoal(*((CGhoul2Info_v *)args[1]), (const char *)VMA(2), (float *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1020-1022`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2576`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1599-1600`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2RageffectorgoalArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    pos: *mut vec3_t,
}

impl CgG2RageffectorgoalArgs {
    pub const fn new(ghoul2: *mut c_void, bone_name: *const c_char, pos: *mut vec3_t) -> Self {
        Self {
            ghoul2,
            bone_name,
            pos,
        }
    }
}

/// `CG_G2_RAGEFFECTORGOAL` MP cgame imports syscall ABI token.
///
/// Raven: additional ragdoll options -rww
/// Raven: override an effector bone's goal position (world coordinates)
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:309`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1020-1022`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1599-1600`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1599-1600`
pub struct CgG2Rageffectorgoal;

impl OutboundSysCall for CgG2Rageffectorgoal {
    type Import = MpCgameImport;
    type Args = CgG2RageffectorgoalArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGEFFECTORGOAL;
}

impl EncodeSysCall for CgG2Rageffectorgoal {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.bone_name),
            ptr_to_word(args.pos as *const vec3_t),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Rageffectorgoal {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
