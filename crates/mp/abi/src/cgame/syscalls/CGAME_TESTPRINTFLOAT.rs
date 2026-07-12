use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CGAME_TESTPRINTFLOAT`.
///
/// Raven shared trap token: `TRAP_TESTPRINTFLOAT`.
/// Raven cgame debug wrapper: `syscall( CG_TESTPRINTFLOAT, string, PASSFLOAT(f) );`
/// Raven MP cgame switch: `case TRAP_TESTPRINTFLOAT: return 0;`
///
/// The MP cgame import enum also contains later `CG_TESTPRINTFLOAT`, but the
/// searched MP engine switch handles the shared `TRAP_TESTPRINTFLOAT` slot that
/// corresponds to `CGAME_TESTPRINTFLOAT`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:517-518`
/// Transport source: `oracle/codemp/qcommon/qcommon.h:296`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:680-681`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgameTestprintfloatArgs {
    string: *const c_char,
    f: f32,
}

impl CgameTestprintfloatArgs {
    pub const fn new(string: *const c_char, f: f32) -> Self {
        Self { string, f }
    }

    pub const fn f(&self) -> f32 {
        self.f
    }
}

/// `CGAME_TESTPRINTFLOAT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:144`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:517-518`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:680-681`
/// Transport source: `oracle/codemp/qcommon/qcommon.h:296`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:680-681`
pub struct CgameTestprintfloat;

impl OutboundSysCall for CgameTestprintfloat {
    type Import = MpCgameImport;
    type Args = CgameTestprintfloatArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_TESTPRINTFLOAT;
}

impl EncodeSysCall for CgameTestprintfloat {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.string), pass_float(args.f())])
    }
}

impl DecodeSysCallReturn for CgameTestprintfloat {
    fn decode_return(_word: isize) -> Self::Output {}
}
