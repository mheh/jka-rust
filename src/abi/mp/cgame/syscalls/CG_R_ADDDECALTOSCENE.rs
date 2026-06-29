use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;

/// Arguments for `CG_R_ADDDECALTOSCENE`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRAdddecaltosceneArgs {
    shader: c_int,
    origin: *const vec3_t,
    dir: *const vec3_t,
    orientation: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    alpha_fade: qboolean,
    radius: f32,
    temporary: qboolean,
}

impl CgRAdddecaltosceneArgs {
    /// # Safety
    /// `origin` and `dir` must be readable 3-float vectors for the duration of the syscall.
    pub const unsafe fn new(
        shader: c_int,
        origin: *const vec3_t,
        dir: *const vec3_t,
        orientation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        alpha_fade: qboolean,
        radius: f32,
        temporary: qboolean,
    ) -> Self {
        Self {
            shader,
            origin,
            dir,
            orientation,
            r,
            g,
            b,
            a,
            alpha_fade,
            radius,
            temporary,
        }
    }

    pub const fn shader(&self) -> c_int {
        self.shader
    }
    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }
    pub const fn dir(&self) -> *const vec3_t {
        self.dir
    }
    pub const fn orientation(&self) -> f32 {
        self.orientation
    }
    pub const fn r(&self) -> f32 {
        self.r
    }
    pub const fn g(&self) -> f32 {
        self.g
    }
    pub const fn b(&self) -> f32 {
        self.b
    }
    pub const fn a(&self) -> f32 {
        self.a
    }
    pub const fn alpha_fade(&self) -> qboolean {
        self.alpha_fade
    }
    pub const fn radius(&self) -> f32 {
        self.radius
    }
    pub const fn temporary(&self) -> qboolean {
        self.temporary
    }
}

/// `CG_R_ADDDECALTOSCENE` MP cgame imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:154`
pub struct CgRAdddecaltoscene;

impl OutboundSysCall for CgRAdddecaltoscene {
    type Import = MpCgameImport;
    type Args = CgRAdddecaltosceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDDECALTOSCENE;
}

impl EncodeSysCall for CgRAdddecaltoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.shader() as isize,
            ptr_to_word(args.origin()),
            ptr_to_word(args.dir()),
            crate::ffi::syscalls::pass_float(args.orientation()),
            crate::ffi::syscalls::pass_float(args.r()),
            crate::ffi::syscalls::pass_float(args.g()),
            crate::ffi::syscalls::pass_float(args.b()),
            crate::ffi::syscalls::pass_float(args.a()),
            args.alpha_fade() as isize,
            crate::ffi::syscalls::pass_float(args.radius()),
            args.temporary() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRAdddecaltoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
