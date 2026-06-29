use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_SETTIME`.
///
/// Raven wrapper: `syscall(CG_G2_SETTIME, time, clock);`
/// Raven transport: `G2API_SetTime(args[1], args[2]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:986-988`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1523-1525`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SettimeArgs {
    time: c_int,
    clock: c_int,
}

impl CgG2SettimeArgs {
    pub const fn new(time: c_int, clock: c_int) -> Self {
        Self { time, clock }
    }
}

/// `CG_G2_SETTIME` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:293`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:986-988`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1523-1525`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1523-1525`
pub struct CgG2Settime;

impl OutboundSysCall for CgG2Settime {
    type Import = MpCgameImport;
    type Args = CgG2SettimeArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETTIME;
}

impl EncodeSysCall for CgG2Settime {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.time as isize, args.clock as isize])
    }
}

impl DecodeSysCallReturn for CgG2Settime {
    fn decode_return(_word: isize) -> Self::Output {}
}
