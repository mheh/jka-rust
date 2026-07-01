use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_R_ADDADDITIVELIGHTTOSCENE`.
///
/// Raven wrapper:
/// `syscall( CG_R_ADDADDITIVELIGHTTOSCENE, org, PASSFLOAT(intensity), PASSFLOAT(r), PASSFLOAT(g), PASSFLOAT(b) );`
/// Raven transport: forwards `org` through `VMA(1)` and scalar values through
/// `VMF(2..5)`, then returns 0.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:356-357`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:915-921`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRAddadditivelighttosceneArgs {
    org: *const vec3_t,
    intensity: f32,
    r: f32,
    g: f32,
    b: f32,
}

impl CgRAddadditivelighttosceneArgs {
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

/// `CG_R_ADDADDITIVELIGHTTOSCENE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:157`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:356-357`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:915-921`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:915-921`
pub struct CgRAddadditivelighttoscene;

impl OutboundSysCall for CgRAddadditivelighttoscene {
    type Import = MpCgameImport;
    type Args = CgRAddadditivelighttosceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDADDITIVELIGHTTOSCENE;
}

impl EncodeSysCall for CgRAddadditivelighttoscene {
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

impl DecodeSysCallReturn for CgRAddadditivelighttoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
