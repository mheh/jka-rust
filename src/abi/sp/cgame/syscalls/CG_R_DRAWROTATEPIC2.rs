use super::super::SpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::abi::pass_float;
use crate::shared::qhandle_t;

/// Arguments for `CG_R_DRAWROTATEPIC2`.
///
/// Raven wrapper packs the coordinates and UVs with `PASSFLOAT`.
/// Raven transport: `re.DrawRotatePic2(VMF(1), ..., VMF(9), args[10]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:420-423`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:729-731`
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
    #[allow(clippy::too_many_arguments)]
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

/// `CG_R_DRAWROTATEPIC2` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:146`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:420-423`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:729-731`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:729-731`
pub struct CgRDrawrotatepic2;

impl OutboundSysCall for CgRDrawrotatepic2 {
    type Import = SpCgameImport;
    type Args = CgRDrawrotatepic2Args;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_DRAWROTATEPIC2;
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
