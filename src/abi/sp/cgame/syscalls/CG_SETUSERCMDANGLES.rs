use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_SETUSERCMDANGLES`.
///
/// Raven wrapper sends three `PASSFLOAT` angle override values.
/// Raven transport: `CL_SetUserCmdAngles(VMF(1), VMF(2), VMF(3));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:479-480`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:773-775`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgSetusercmdanglesArgs {
    pitch_override: f32,
    yaw_override: f32,
    roll_override: f32,
}

impl CgSetusercmdanglesArgs {
    pub const fn new(pitch_override: f32, yaw_override: f32, roll_override: f32) -> Self {
        Self {
            pitch_override,
            yaw_override,
            roll_override,
        }
    }
}

/// `CG_SETUSERCMDANGLES` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:161`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:479-480`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:773-775`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:773-775`
pub struct CgSetusercmdangles;

impl OutboundSysCall for CgSetusercmdangles {
    type Import = SpCgameImport;
    type Args = CgSetusercmdanglesArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_SETUSERCMDANGLES;
}

impl EncodeSysCall for CgSetusercmdangles {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.pitch_override),
            pass_float(args.yaw_override),
            pass_float(args.roll_override),
        ])
    }
}

impl DecodeSysCallReturn for CgSetusercmdangles {
    fn decode_return(_word: isize) -> Self::Output {}
}
