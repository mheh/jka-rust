use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::abi::pass_float;

/// Arguments for `CG_R_DRAWSTRETCHPIC`.
///
/// Raven wrapper packs floats with `PASSFLOAT`; the client switch decodes them
/// with `VMF`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:368-370`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2278-2279`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:928-930`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRDrawstretchpicArgs {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    h_shader: c_int,
}

impl CgRDrawstretchpicArgs {
    pub const fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        h_shader: c_int,
    ) -> Self {
        Self {
            x,
            y,
            w,
            h,
            s1,
            t1,
            s2,
            t2,
            h_shader,
        }
    }
}

/// `CG_R_DRAWSTRETCHPIC` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:160`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:368-370`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:928-930`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:928-930`
pub struct CgRDrawstretchpic;

impl OutboundSysCall for CgRDrawstretchpic {
    type Import = MpCgameImport;
    type Args = CgRDrawstretchpicArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_DRAWSTRETCHPIC;
}

impl EncodeSysCall for CgRDrawstretchpic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.x),
            pass_float(args.y),
            pass_float(args.w),
            pass_float(args.h),
            pass_float(args.s1),
            pass_float(args.t1),
            pass_float(args.s2),
            pass_float(args.t2),
            args.h_shader as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRDrawstretchpic {
    fn decode_return(_word: isize) -> Self::Output {}
}
