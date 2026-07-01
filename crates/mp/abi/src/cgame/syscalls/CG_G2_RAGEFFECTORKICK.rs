use core::ffi::{c_char, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_G2_RAGEFFECTORKICK`.
///
/// Raven wrapper: `qboolean trap_G2API_RagEffectorKick(void *ghoul2, const char *boneName, vec3_t velocity)`
/// Raven transport: `return G2API_RagEffectorKick(*((CGhoul2Info_v *)args[1]), (const char *)VMA(2), (float *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1030-1032`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2578`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1603-1604`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2RageffectorkickArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    velocity: *mut vec3_t,
}

impl CgG2RageffectorkickArgs {
    pub const fn new(ghoul2: *mut c_void, bone_name: *const c_char, velocity: *mut vec3_t) -> Self {
        Self {
            ghoul2,
            bone_name,
            velocity,
        }
    }
}

/// `CG_G2_RAGEFFECTORKICK` MP cgame imports syscall ABI token.
///
/// Raven: additional ragdoll options -rww
/// Raven: add velocity to a rag bone
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:311`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1030-1032`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1603-1604`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1603-1604`
pub struct CgG2Rageffectorkick;

impl OutboundSysCall for CgG2Rageffectorkick {
    type Import = MpCgameImport;
    type Args = CgG2RageffectorkickArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGEFFECTORKICK;
}

impl EncodeSysCall for CgG2Rageffectorkick {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.bone_name),
            ptr_to_word(args.velocity as *const vec3_t),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Rageffectorkick {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
