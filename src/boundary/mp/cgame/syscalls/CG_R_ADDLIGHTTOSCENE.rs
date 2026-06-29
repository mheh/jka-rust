use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_R_ADDLIGHTTOSCENE`.
///
/// Raven wrapper:
/// `syscall( CG_R_ADDLIGHTTOSCENE, org, PASSFLOAT(intensity), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b) );`
/// Raven transport: forwards `org` through `VMA(1)` and scalar values through
/// `VMF(2..5)`, then returns 0.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:352-353`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2274`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:908-914`
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
}

/// `CG_R_ADDLIGHTTOSCENE` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:156`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:352-353`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:908-914`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:908-914`
pub struct CgRAddlighttoscene;

impl OutboundSysCall for CgRAddlighttoscene {
    type Import = MpCgameImport;
    type Args = CgRAddlighttosceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDLIGHTTOSCENE;
}

impl EncodeSysCall for CgRAddlighttoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.org),
            pass_float(args.intensity),
            pass_float(args.r),
            pass_float(args.g),
            pass_float(args.b),
        ])
    }
}

impl DecodeSysCallReturn for CgRAddlighttoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
