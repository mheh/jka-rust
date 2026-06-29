use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_DOESBONEEXIST`.
///
/// Raven: check if a bone exists on skeleton without actually adding to the bone list -rww.
/// Raven wrapper: `return syscall(CG_G2_DOESBONEEXIST, ghoul2, modelIndex, boneName);`
/// Raven transport: `return G2API_DoesBoneExist(&g2[args[2]], (const char *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:971-973`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2562`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1507-1511`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2DoesboneexistArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bone_name: *const c_char,
}

impl CgG2DoesboneexistArgs {
    pub const fn new(ghoul2: *mut c_void, model_index: c_int, bone_name: *const c_char) -> Self {
        Self {
            ghoul2,
            model_index,
            bone_name,
        }
    }
}

/// `CG_G2_DOESBONEEXIST` MP cgame imports syscall boundary token.
///
/// Raven: check if a bone exists on skeleton without actually adding to the bone list -rww
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:289`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:971-973`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1507-1511`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1507-1511`
pub struct CgG2Doesboneexist;

impl OutboundSysCall for CgG2Doesboneexist {
    type Import = MpCgameImport;
    type Args = CgG2DoesboneexistArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_DOESBONEEXIST;
}

impl EncodeSysCall for CgG2Doesboneexist {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            ptr_to_word(args.bone_name),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Doesboneexist {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
