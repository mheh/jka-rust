use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_R_ADDLIGHTTOSCENE`.
///
/// Raven wrapper:
/// `syscall( CG_R_ADDLIGHTTOSCENE, org, PASSFLOAT(intensity), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b) );`
/// Raven transport forwards `org` through `VMA(1)` and scalar values through `VMF(2..5)`.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:384-385`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:701-707`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRAddlighttosceneArgs {
    org: *const vec3_t,
    intensity: f32,
    r: f32,
    g: f32,
    b: f32,
}

impl CgRAddlighttosceneArgs {
    pub const fn new(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32) -> Self {
        Self {
            org,
            intensity,
            r,
            g,
            b,
        }
    }

    pub const fn org(&self) -> *const vec3_t {
        self.org
    }

    pub const fn intensity(&self) -> f32 {
        self.intensity
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
}

/// `CG_R_ADDLIGHTTOSCENE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:138`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:384-385`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:701-707`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:701-707`
pub struct CgRAddlighttoscene;

impl OutboundSysCall for CgRAddlighttoscene {
    type Import = SpCgameImport;
    type Args = CgRAddlighttosceneArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_ADDLIGHTTOSCENE;
}

impl EncodeSysCall for CgRAddlighttoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.org()),
            pass_float(args.intensity()),
            pass_float(args.r()),
            pass_float(args.g()),
            pass_float(args.b()),
        ])
    }
}

impl DecodeSysCallReturn for CgRAddlighttoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
