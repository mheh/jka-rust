use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_SETCLIENTFORCEANGLE`.
///
/// Raven wrapper: `syscall( CG_SETCLIENTFORCEANGLE, time, angle );`
/// Raven transport: `CL_SetClientForceAngle(args[1], (float *)VMA(2)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:498-500`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2351`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:977-979`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSetclientforceangleArgs {
    time: c_int,
    angle: *mut vec3_t,
}

impl CgSetclientforceangleArgs {
    pub const fn new(time: c_int, angle: *mut vec3_t) -> Self {
        Self { time, angle }
    }
}

/// `CG_SETCLIENTFORCEANGLE` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:188`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:498-500`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:977-979`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:977-979`
pub struct CgSetclientforceangle;

impl OutboundSysCall for CgSetclientforceangle {
    type Import = MpCgameImport;
    type Args = CgSetclientforceangleArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_SETCLIENTFORCEANGLE;
}

impl EncodeSysCall for CgSetclientforceangle {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.time as isize, ptr_to_word(args.angle)])
    }
}

impl DecodeSysCallReturn for CgSetclientforceangle {
    fn decode_return(_word: isize) -> Self::Output {}
}
