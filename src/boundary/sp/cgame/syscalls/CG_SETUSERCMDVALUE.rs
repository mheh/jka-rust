use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_SETUSERCMDVALUE`.
///
/// Raven wrapper sends `stateValue` plus three `PASSFLOAT` values.
/// Raven transport: `CL_SetUserCmdValue(args[1], VMF(2), VMF(3), VMF(4));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:475-476`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:770-772`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgSetusercmdvalueArgs {
    state_value: c_int,
    sensitivity_scale: f32,
    m_pitch_override: f32,
    m_yaw_override: f32,
}

impl CgSetusercmdvalueArgs {
    pub const fn new(
        state_value: c_int,
        sensitivity_scale: f32,
        m_pitch_override: f32,
        m_yaw_override: f32,
    ) -> Self {
        Self {
            state_value,
            sensitivity_scale,
            m_pitch_override,
            m_yaw_override,
        }
    }
}

/// `CG_SETUSERCMDVALUE` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:160`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:475-476`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:770-772`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:770-772`
pub struct CgSetusercmdvalue;

impl OutboundSysCall for CgSetusercmdvalue {
    type Import = SpCgameImport;
    type Args = CgSetusercmdvalueArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_SETUSERCMDVALUE;
}

impl EncodeSysCall for CgSetusercmdvalue {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.state_value as isize,
            pass_float(args.sensitivity_scale),
            pass_float(args.m_pitch_override),
            pass_float(args.m_yaw_override),
        ])
    }
}

impl DecodeSysCallReturn for CgSetusercmdvalue {
    fn decode_return(_word: isize) -> Self::Output {}
}
