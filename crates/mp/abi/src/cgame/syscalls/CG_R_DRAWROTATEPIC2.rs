use mp_qshared::shared::qhandle_t;

use super::super::MpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CG_R_DRAWROTATEPIC2`.
///
/// Raven wrapper packs the coordinates and UVs with `PASSFLOAT`.
/// Raven comment: "rotates image around exact center point of passed in
/// coords".
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:388-391`
/// Args source: `oracle/codemp/cgame/cg_local.h:2286-2288`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:939-941`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRDrawrotatepic2Args {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    a: f32,
    h_shader: qhandle_t,
}

impl CgRDrawrotatepic2Args {
    pub const fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        a: f32,
        h_shader: qhandle_t,
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
            a,
            h_shader,
        }
    }
}

/// `CG_R_DRAWROTATEPIC2` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:164`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:388-391`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:939-941`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:939-941`
pub struct CgRDrawrotatepic2;

impl OutboundSysCall for CgRDrawrotatepic2 {
    type Import = MpCgameImport;
    type Args = CgRDrawrotatepic2Args;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_DRAWROTATEPIC2;
}

impl EncodeSysCall for CgRDrawrotatepic2 {
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
            pass_float(args.a),
            args.h_shader as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRDrawrotatepic2 {
    fn decode_return(_word: isize) -> Self::Output {}
}
