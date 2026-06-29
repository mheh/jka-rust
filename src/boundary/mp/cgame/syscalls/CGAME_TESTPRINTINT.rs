use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CGAME_TESTPRINTINT`.
///
/// Raven shared trap token: `TRAP_TESTPRINTINT`.
/// Raven cgame debug wrapper: `syscall( CG_TESTPRINTINT, string, i );`
/// Raven MP cgame switch: `case TRAP_TESTPRINTINT: return 0;`
///
/// The MP cgame import enum also contains later `CG_TESTPRINTINT`, but the
/// searched MP engine switch handles the shared `TRAP_TESTPRINTINT` slot that
/// corresponds to `CGAME_TESTPRINTINT`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:513-514`
/// Transport source: `oracle/oracle/codemp/qcommon/qcommon.h:295`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:678-679`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgameTestprintintArgs {
    string: *const c_char,
    i: c_int,
}

impl CgameTestprintintArgs {
    pub const fn new(string: *const c_char, i: c_int) -> Self {
        Self { string, i }
    }
}

/// `CGAME_TESTPRINTINT` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:143`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:513-514`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:678-679`
/// Transport source: `oracle/oracle/codemp/qcommon/qcommon.h:295`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:678-679`
pub struct CgameTestprintint;

impl OutboundSysCall for CgameTestprintint {
    type Import = MpCgameImport;
    type Args = CgameTestprintintArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_TESTPRINTINT;
}

impl EncodeSysCall for CgameTestprintint {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.string), args.i as isize])
    }
}

impl DecodeSysCallReturn for CgameTestprintint {
    fn decode_return(_word: isize) -> Self::Output {}
}
