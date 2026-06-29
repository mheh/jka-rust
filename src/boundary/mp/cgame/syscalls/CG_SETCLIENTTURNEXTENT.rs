use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_SETCLIENTTURNEXTENT`.
///
/// Raven wrapper: `syscall( CG_SETCLIENTTURNEXTENT, PASSFLOAT(turnAdd), PASSFLOAT(turnSub), turnTime );`
/// Raven transport: the MP client switch currently returns `0` without reading
/// `args[1..=3]`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:503-505`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2352`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:980-981`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgSetclientturnextentArgs {
    turn_add: f32,
    turn_sub: f32,
    turn_time: c_int,
}

impl CgSetclientturnextentArgs {
    pub const fn new(turn_add: f32, turn_sub: f32, turn_time: c_int) -> Self {
        Self {
            turn_add,
            turn_sub,
            turn_time,
        }
    }
}

/// `CG_SETCLIENTTURNEXTENT` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:189`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:503-505`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:980-981`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:980-981`
pub struct CgSetclientturnextent;

impl OutboundSysCall for CgSetclientturnextent {
    type Import = MpCgameImport;
    type Args = CgSetclientturnextentArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_SETCLIENTTURNEXTENT;
}

impl EncodeSysCall for CgSetclientturnextent {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.turn_add),
            pass_float(args.turn_sub),
            args.turn_time as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSetclientturnextent {
    fn decode_return(_word: isize) -> Self::Output {}
}
