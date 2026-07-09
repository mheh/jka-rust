use core::ffi::{c_char, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_RAGPCJGRADIENTSPEED`.
///
/// Raven wrapper: `qboolean trap_G2API_RagPCJGradientSpeed(void *ghoul2, const char *boneName, const float speed)`
/// Raven transport: `return G2API_RagPCJGradientSpeed(*((CGhoul2Info_v *)args[1]), (const char *)VMA(2), VMF(3));`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1015-1017`
/// Args source: `oracle/codemp/cgame/cg_local.h:2575`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1597-1598`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgG2RagpcjgradientspeedArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    speed: f32,
}

impl CgG2RagpcjgradientspeedArgs {
    pub const fn new(ghoul2: *mut c_void, bone_name: *const c_char, speed: f32) -> Self {
        Self {
            ghoul2,
            bone_name,
            speed,
        }
    }
}

/// `CG_G2_RAGPCJGRADIENTSPEED` MP cgame imports syscall ABI token.
///
/// Raven: additional ragdoll options -rww
/// Enum value source: `oracle/codemp/cgame/cg_public.h:308`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1015-1017`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1597-1598`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1597-1598`
pub struct CgG2Ragpcjgradientspeed;

impl OutboundSysCall for CgG2Ragpcjgradientspeed {
    type Import = MpCgameImport;
    type Args = CgG2RagpcjgradientspeedArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGPCJGRADIENTSPEED;
}

impl EncodeSysCall for CgG2Ragpcjgradientspeed {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.bone_name),
            pass_float(args.speed),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Ragpcjgradientspeed {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
