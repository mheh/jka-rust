use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CG_R_SCISSOR`.
///
/// Raven wrapper: `syscall( CG_R_SCISSOR, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h));`
/// Raven transport: `re.Scissor(VMF(1), VMF(2), VMF(3), VMF(4));`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:437-439`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:746-748`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRScissorArgs {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl CgRScissorArgs {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

/// `CG_R_SCISSOR` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:149`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:437-439`
/// Output source: `oracle/code/client/cl_cgame.cpp:746-748`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:746-748`
pub struct CgRScissor;

impl OutboundSysCall for CgRScissor {
    type Import = SpCgameImport;
    type Args = CgRScissorArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SCISSOR;
}

impl EncodeSysCall for CgRScissor {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.x),
            pass_float(args.y),
            pass_float(args.w),
            pass_float(args.h),
        ])
    }
}

impl DecodeSysCallReturn for CgRScissor {
    fn decode_return(_word: isize) -> Self::Output {}
}
