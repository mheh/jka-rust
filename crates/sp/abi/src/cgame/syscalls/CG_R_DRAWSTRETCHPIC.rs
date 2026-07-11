use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;
use sp_qshared::shared::qhandle_t;

/// Arguments for `CG_R_DRAWSTRETCHPIC`.
///
/// Raven wrapper: `cgi_R_DrawStretchPic( float x, float y, float w, float h, float s1, float t1, float s2, float t2, qhandle_t hShader )`
/// Raven transport: `re.DrawStretchPic( VMF(1), VMF(2), VMF(3), VMF(4), VMF(5), VMF(6), VMF(7), VMF(8), args[9] );`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:398`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:714-716`
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
    h_shader: qhandle_t,
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

    pub const fn h_shader(&self) -> qhandle_t {
        self.h_shader
    }
}

/// `CG_R_DRAWSTRETCHPIC` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:141`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:398`
/// Output source: `oracle/code/client/cl_cgame.cpp:714-716`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:714-716`
pub struct CgRDrawstretchpic;

impl OutboundSysCall for CgRDrawstretchpic {
    type Import = SpCgameImport;
    type Args = CgRDrawstretchpicArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_DRAWSTRETCHPIC;
}

impl EncodeSysCall for CgRDrawstretchpic {
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
            args.h_shader() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRDrawstretchpic {
    fn decode_return(_word: isize) -> Self::Output {}
}
