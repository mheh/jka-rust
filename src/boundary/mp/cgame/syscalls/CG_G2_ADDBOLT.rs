use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_ADDBOLT`.
///
/// Raven wrapper: `return syscall(CG_G2_ADDBOLT, ghoul2, modelIndex, boneName);`
/// Raven transport: `return G2API_AddBolt(*((CGhoul2Info_v *)args[1]), args[2], (const char *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:940-942`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2544`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1475-1476`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2AddboltArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bone_name: *const c_char,
}

impl CgG2AddboltArgs {
    pub const fn new(ghoul2: *mut c_void, model_index: c_int, bone_name: *const c_char) -> Self {
        Self {
            ghoul2,
            model_index,
            bone_name,
        }
    }
}

/// `CG_G2_ADDBOLT` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:283`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:940-942`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1475-1476`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1475-1476`
pub struct CgG2Addbolt;

impl OutboundSysCall for CgG2Addbolt {
    type Import = MpCgameImport;
    type Args = CgG2AddboltArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ADDBOLT;
}

impl EncodeSysCall for CgG2Addbolt {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            ptr_to_word(args.bone_name),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Addbolt {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
