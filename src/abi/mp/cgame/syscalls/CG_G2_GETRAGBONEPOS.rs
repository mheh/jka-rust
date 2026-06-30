use core::ffi::{c_char, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;
use crate::shared::vec3_t;

/// Arguments for `CG_G2_GETRAGBONEPOS`.
///
/// Raven wrapper: `return syscall(CG_G2_GETRAGBONEPOS, ghoul2, boneName, pos, entAngles, entPos, entScale);`
/// Raven transport: `return G2API_GetRagBonePos(*((CGhoul2Info_v *)args[1]), (const char *)VMA(2), (float *)VMA(3), (float *)VMA(4), (float *)VMA(5), (float *)VMA(6));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1025-1027`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2577`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1601-1602`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetragboneposArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    pos: *mut vec3_t,
    ent_angles: *const vec3_t,
    ent_pos: *const vec3_t,
    ent_scale: *const vec3_t,
}

impl CgG2GetragboneposArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        bone_name: *const c_char,
        pos: *mut vec3_t,
        ent_angles: *const vec3_t,
        ent_pos: *const vec3_t,
        ent_scale: *const vec3_t,
    ) -> Self {
        Self {
            ghoul2,
            bone_name,
            pos,
            ent_angles,
            ent_pos,
            ent_scale,
        }
    }
}

/// `CG_G2_GETRAGBONEPOS` MP cgame imports syscall ABI token.
///
/// Raven: current position of said bone is put into pos (world coordinates)
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:310`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1025-1027`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1601-1602`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1601-1602`
pub struct CgG2Getragbonepos;

impl OutboundSysCall for CgG2Getragbonepos {
    type Import = MpCgameImport;
    type Args = CgG2GetragboneposArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETRAGBONEPOS;
}

impl EncodeSysCall for CgG2Getragbonepos {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.bone_name),
            ptr_to_word(args.pos),
            ptr_to_word(args.ent_angles),
            ptr_to_word(args.ent_pos),
            ptr_to_word(args.ent_scale),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Getragbonepos {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
