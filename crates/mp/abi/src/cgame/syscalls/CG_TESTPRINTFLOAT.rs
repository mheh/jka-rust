use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CG_TESTPRINTFLOAT`.
///
/// Raven shared trap token: `TRAP_TESTPRINTFLOAT`.
/// Raven cgame debug wrapper: `testPrintFloat( char *string, float f );`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgTestprintfloatArgs {
    string: *const c_char,
    f: f32,
}

impl CgTestprintfloatArgs {
    pub const fn new(string: *const c_char, f: f32) -> Self {
        Self { string, f }
    }

    pub const fn f(self) -> f32 {
        self.f
    }
}

/// `CG_TESTPRINTFLOAT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:191-192`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:517-518`
/// Transport enum/source: `oracle/oracle/codemp/qcommon/qcommon.h:295-296`
/// Engine switch/source: `oracle/oracle/codemp/client/cl_cgame.cpp:678-681`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:678-681`
///
/// Raven's searched MP cgame engine switch handles shared `TRAP_TESTPRINTFLOAT`
/// at `oracle/oracle/codemp/client/cl_cgame.cpp:680-681`, which corresponds to
/// `CGAME_TESTPRINTFLOAT`, while this ABI token appears later in
/// `cg_public.h` and keeps the exported ABI index.
pub struct CgTestprintfloat;

impl OutboundSysCall for CgTestprintfloat {
    type Import = MpCgameImport;
    type Args = CgTestprintfloatArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_TESTPRINTFLOAT;
}

impl EncodeSysCall for CgTestprintfloat {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.string), pass_float(args.f)])
    }
}

impl DecodeSysCallReturn for CgTestprintfloat {
    fn decode_return(_word: isize) -> Self::Output {}
}
