use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_TESTPRINTINT`.
///
/// Raven shared trap token: `TRAP_TESTPRINTINT`.
/// Raven cgame debug wrapper: `testPrintInt( char *string, int i );`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgTestprintintArgs {
    string: *const c_char,
    i: c_int,
}

impl CgTestprintintArgs {
    pub const fn new(string: *const c_char, i: c_int) -> Self {
        Self { string, i }
    }
}

/// `CG_TESTPRINTINT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:191`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:513-514`
/// Transport enum/source: `oracle/oracle/codemp/qcommon/qcommon.h:295-296`
/// Engine switch/source: `oracle/oracle/codemp/client/cl_cgame.cpp:678-681`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:678-679`
///
/// Raven's searched MP cgame engine switch handles shared `TRAP_TESTPRINTINT`
/// at `oracle/oracle/codemp/client/cl_cgame.cpp:678-679`, which corresponds to
/// `CGAME_TESTPRINTINT`. Keep this as the closest contract match; this export enum
/// value appears later in `cg_public.h`.
pub struct CgTestprintint;

impl OutboundSysCall for CgTestprintint {
    type Import = MpCgameImport;
    type Args = CgTestprintintArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_TESTPRINTINT;
}

impl EncodeSysCall for CgTestprintint {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.string), args.i as isize])
    }
}

impl DecodeSysCallReturn for CgTestprintint {
    fn decode_return(_word: isize) -> Self::Output {}
}
