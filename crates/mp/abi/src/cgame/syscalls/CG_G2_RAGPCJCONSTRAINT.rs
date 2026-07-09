use core::ffi::{c_char, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_G2_RAGPCJCONSTRAINT`.
///
/// Raven wrapper: `qboolean trap_G2API_RagPCJConstraint(void *ghoul2, const char *boneName, vec3_t min, vec3_t max)`
/// Raven transport: `return G2API_RagPCJConstraint(*((CGhoul2Info_v *)args[1]), (const char *)VMA(2), (float *)VMA(3), (float *)VMA(4));`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1010-1012`
/// Args source: `oracle/codemp/cgame/cg_local.h:2574`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1595-1596`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2RagpcjconstraintArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    min: *mut vec3_t,
    max: *mut vec3_t,
}

impl CgG2RagpcjconstraintArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        bone_name: *const c_char,
        min: *mut vec3_t,
        max: *mut vec3_t,
    ) -> Self {
        Self {
            ghoul2,
            bone_name,
            min,
            max,
        }
    }
}

/// `CG_G2_RAGPCJCONSTRAINT` MP cgame imports syscall ABI token.
///
/// Raven: rww - RAGDOLL_END
/// Raven: additional ragdoll options -rww
/// Enum value source: `oracle/codemp/cgame/cg_public.h:307`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1010-1012`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1595-1596`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1595-1596`
pub struct CgG2Ragpcjconstraint;

impl OutboundSysCall for CgG2Ragpcjconstraint {
    type Import = MpCgameImport;
    type Args = CgG2RagpcjconstraintArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGPCJCONSTRAINT;
}

impl EncodeSysCall for CgG2Ragpcjconstraint {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.bone_name),
            ptr_to_word(args.min as *const vec3_t),
            ptr_to_word(args.max as *const vec3_t),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Ragpcjconstraint {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
