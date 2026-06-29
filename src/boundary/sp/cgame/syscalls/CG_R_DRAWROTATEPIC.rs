use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_R_DRAWROTATEPIC`.
///
/// Raven wrapper: `cgi_R_DrawRotatePic( float x, float y, float w, float h, float s1, float t1, float s2, float t2, float a, qhandle_t hShader )`
/// Raven transport: `re.DrawRotatePic( VMF(1), VMF(2), VMF(3), VMF(4), VMF(5), VMF(6), VMF(7), VMF(8), VMF(9), args[10] );`
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:145`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:414-417`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:726-728`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:726-728`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRDrawrotatepicArgs {
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

impl CgRDrawrotatepicArgs {
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

    pub const fn x(&self) -> f32 {
        self.x
    }

    pub const fn y(&self) -> f32 {
        self.y
    }

    pub const fn w(&self) -> f32 {
        self.w
    }

    pub const fn h(&self) -> f32 {
        self.h
    }

    pub const fn s1(&self) -> f32 {
        self.s1
    }

    pub const fn t1(&self) -> f32 {
        self.t1
    }

    pub const fn s2(&self) -> f32 {
        self.s2
    }

    pub const fn t2(&self) -> f32 {
        self.t2
    }

    pub const fn a(&self) -> f32 {
        self.a
    }

    pub const fn h_shader(&self) -> qhandle_t {
        self.h_shader
    }
}

/// `CG_R_DRAWROTATEPIC` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:145`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:414-417`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:726-728`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:726-728`
pub struct CgRDrawrotatepic;

impl OutboundSysCall for CgRDrawrotatepic {
    type Import = SpCgameImport;
    type Args = CgRDrawrotatepicArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_DRAWROTATEPIC;
}

impl EncodeSysCall for CgRDrawrotatepic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.x()),
            pass_float(args.y()),
            pass_float(args.w()),
            pass_float(args.h()),
            pass_float(args.s1()),
            pass_float(args.t1()),
            pass_float(args.s2()),
            pass_float(args.t2()),
            pass_float(args.a()),
            args.h_shader() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRDrawrotatepic {
    fn decode_return(_word: isize) -> Self::Output {}
}
