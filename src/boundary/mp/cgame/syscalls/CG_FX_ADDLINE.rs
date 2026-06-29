use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::{codemp::game::q_shared_h::vec3_t, ffi::syscalls::pass_float};

/// Arguments for `CG_FX_ADDLINE`.
///
/// Raven wrapper: `syscall( CG_FX_ADDLINE, start, end, PASSFLOAT(size1), PASSFLOAT(size2), PASSFLOAT(sizeParm), PASSFLOAT(alpha1), PASSFLOAT(alpha2), PASSFLOAT(alphaParm), sRGB, eRGB, PASSFLOAT(rgbParm), killTime, shader, flags);`
/// Raven transport: `FX_AddLine( (float *)VMA(1), (float *)VMA(2), VMF(3), VMF(4), VMF(5), VMF(6), VMF(7), VMF(8), (float *)VMA(9), (float *)VMA(10), VMF(11), args[12], args[13], args[14]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:450-459`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2308-2311`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1099-1104`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAddlineArgs {
    start: *const vec3_t,
    end: *const vec3_t,
    size1: f32,
    size2: f32,
    size_parm: f32,
    alpha1: f32,
    alpha2: f32,
    alpha_parm: f32,
    s_rgb: *const vec3_t,
    e_rgb: *const vec3_t,
    rgb_parm: f32,
    kill_time: c_int,
    shader: c_int,
    flags: c_int,
}

impl CgFxAddlineArgs {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        start: *const vec3_t,
        end: *const vec3_t,
        size1: f32,
        size2: f32,
        size_parm: f32,
        alpha1: f32,
        alpha2: f32,
        alpha_parm: f32,
        s_rgb: *const vec3_t,
        e_rgb: *const vec3_t,
        rgb_parm: f32,
        kill_time: c_int,
        shader: c_int,
        flags: c_int,
    ) -> Self {
        Self {
            start,
            end,
            size1,
            size2,
            size_parm,
            alpha1,
            alpha2,
            alpha_parm,
            s_rgb,
            e_rgb,
            rgb_parm,
            kill_time,
            shader,
            flags,
        }
    }
}

/// `CG_FX_ADDLINE` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:177`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:450-459`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1099-1104`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1099-1104`
pub struct CgFxAddline;

impl OutboundSysCall for CgFxAddline {
    type Import = MpCgameImport;
    type Args = CgFxAddlineArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADDLINE;
}

impl EncodeSysCall for CgFxAddline {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.start),
            ptr_to_word(args.end),
            pass_float(args.size1),
            pass_float(args.size2),
            pass_float(args.size_parm),
            pass_float(args.alpha1),
            pass_float(args.alpha2),
            pass_float(args.alpha_parm),
            ptr_to_word(args.s_rgb),
            ptr_to_word(args.e_rgb),
            pass_float(args.rgb_parm),
            args.kill_time as isize,
            args.shader as isize,
            args.flags as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFxAddline {
    fn decode_return(_word: isize) -> Self::Output {}
}
