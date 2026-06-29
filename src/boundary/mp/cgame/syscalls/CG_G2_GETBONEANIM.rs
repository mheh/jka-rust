use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_GETBONEANIM`.
///
/// Raven wrapper: `return syscall(CG_G2_GETBONEANIM, ghoul2, boneName, currentTime, currentFrame, startFrame, endFrame, flags, animSpeed, modelList, modelIndex);`
/// Raven transport: `CGhoul2Info_v &g2 = *((CGhoul2Info_v *)args[1]); int modelIndex = args[10]; return G2API_GetBoneAnim(&g2[modelIndex], (const char*)VMA(2), args[3], (float *)VMA(4), (int *)VMA(5), (int *)VMA(6), (int *)VMA(7), (float *)VMA(8), (int *)VMA(9));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:874-877`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2555-2556`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:874,877`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1386-1392`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1386-1392`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetboneanimArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    current_time: c_int,
    current_frame: *mut f32,
    start_frame: *mut c_int,
    end_frame: *mut c_int,
    flags: *mut c_int,
    anim_speed: *mut f32,
    model_list: *mut c_int,
    model_index: c_int,
}

impl CgG2GetboneanimArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        bone_name: *const c_char,
        current_time: c_int,
        current_frame: *mut f32,
        start_frame: *mut c_int,
        end_frame: *mut c_int,
        flags: *mut c_int,
        anim_speed: *mut f32,
        model_list: *mut c_int,
        model_index: c_int,
    ) -> Self {
        Self {
            ghoul2,
            bone_name,
            current_time,
            current_frame,
            start_frame,
            end_frame,
            flags,
            anim_speed,
            model_list,
            model_index,
        }
    }
}

/// `CG_G2_GETBONEANIM` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:270`
pub struct CgG2Getboneanim;

impl OutboundSysCall for CgG2Getboneanim {
    type Import = MpCgameImport;
    type Args = CgG2GetboneanimArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETBONEANIM;
}

impl EncodeSysCall for CgG2Getboneanim {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2 as *const _),
            ptr_to_word(args.bone_name),
            args.current_time as isize,
            ptr_to_word(args.current_frame as *const _),
            ptr_to_word(args.start_frame as *const _),
            ptr_to_word(args.end_frame as *const _),
            ptr_to_word(args.flags as *const _),
            ptr_to_word(args.anim_speed as *const _),
            ptr_to_word(args.model_list as *const _),
            args.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Getboneanim {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
