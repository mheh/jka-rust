use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_REMOVEBONE`.
///
/// Raven wrapper: `return syscall(CG_G2_REMOVEBONE, ghoul2, boneName, modelIndex);`
/// Raven transport: `return G2API_RemoveBone(&g2[args[3]], (const char *)VMA(2));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1050-1052`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2585`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1613-1618`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2RemoveboneArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    model_index: c_int,
}

impl CgG2RemoveboneArgs {
    pub const fn new(ghoul2: *mut c_void, bone_name: *const c_char, model_index: c_int) -> Self {
        Self {
            ghoul2,
            bone_name,
            model_index,
        }
    }
}

/// `CG_G2_REMOVEBONE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:319`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1050-1052`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1613-1618`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1613-1618`
pub struct CgG2Removebone;

impl OutboundSysCall for CgG2Removebone {
    type Import = MpCgameImport;
    type Args = CgG2RemoveboneArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_REMOVEBONE;
}

impl EncodeSysCall for CgG2Removebone {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.bone_name),
            args.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Removebone {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
