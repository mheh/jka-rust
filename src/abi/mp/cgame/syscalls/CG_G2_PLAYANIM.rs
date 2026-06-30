use core::ffi::{c_int, c_void};
use std::ffi::CString;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::abi::pass_float;
use crate::shared::qboolean;

/// Arguments for `CG_G2_PLAYANIM`.
///
/// Raven wrapper: `qboolean trap_G2API_SetBoneAnim(...)`.
/// Raven transport: `return G2API_SetBoneAnim(*((CGhoul2Info_v *)args[1]), args[2], (const char *)VMA(3), args[4], args[5],
///     args[6], VMF(7), args[8], VMF(9), args[10]);`.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:269`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:868-871`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2553-2554`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1382-1384`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1382-1384`
#[derive(Debug, Clone, PartialEq)]
pub struct CgG2PlayanimArgs {
    /// Ghoul2 handle, decoded by Raven as `*((CGhoul2Info_v *)args[1])`.
    ghoul2: *mut c_void,
    /// Model index.
    model_index: c_int,
    /// Bone name transported as a NUL-terminated string pointer.
    bone_name: CString,
    /// Start frame.
    start_frame: c_int,
    /// End frame.
    end_frame: c_int,
    /// Animation flags.
    flags: c_int,
    /// Animation speed, transported with `VMF(7)` / `PASSFLOAT`.
    anim_speed: f32,
    /// Current time.
    current_time: c_int,
    /// Set frame, transported with `VMF(9)` / `PASSFLOAT`.
    set_frame: f32,
    /// Blend time.
    blend_time: c_int,
}

impl CgG2PlayanimArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        bone_name: CString,
        start_frame: c_int,
        end_frame: c_int,
        flags: c_int,
        anim_speed: f32,
        current_time: c_int,
        set_frame: f32,
        blend_time: c_int,
    ) -> Self {
        Self {
            ghoul2,
            model_index,
            bone_name,
            start_frame,
            end_frame,
            flags,
            anim_speed,
            current_time,
            set_frame,
            blend_time,
        }
    }
}

/// `CG_G2_PLAYANIM` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:269`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:868-871`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2553-2554`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1382-1384`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1382-1384`
pub struct CgG2Playanim;

impl OutboundSysCall for CgG2Playanim {
    type Import = MpCgameImport;
    type Args = CgG2PlayanimArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_PLAYANIM;
}

impl EncodeSysCall for CgG2Playanim {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            ptr_to_word(args.bone_name.as_ptr()),
            args.start_frame as isize,
            args.end_frame as isize,
            args.flags as isize,
            pass_float(args.anim_speed),
            args.current_time as isize,
            pass_float(args.set_frame),
            args.blend_time as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Playanim {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
