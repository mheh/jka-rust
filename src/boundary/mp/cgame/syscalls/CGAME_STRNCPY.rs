use super::super::MpCgameImport;
use core::ffi::{c_char, c_int};

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CGAME_STRNCPY`.
///
/// Raven's MP client switch reads `dest` with `VMA(1)`, `src` with `VMA(2)`,
/// and `count` from `args[3]`, then calls C `strncpy`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:656`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:655`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:284`
#[derive(Debug)]
pub struct CgameStrncpyArgs {
    dest: *mut c_char,
    src: *const c_char,
    count: c_int,
}

impl CgameStrncpyArgs {
    /// Construct the raw `strncpy` syscall args.
    ///
    /// # Safety
    /// `dest` must be valid for writes of up to `count` bytes, `src` must be a
    /// valid C string readable for the same operation, and the buffers must obey
    /// C `strncpy` aliasing requirements.
    pub const unsafe fn new(dest: *mut c_char, src: *const c_char, count: c_int) -> Self {
        Self { dest, src, count }
    }

    pub const fn dest(&self) -> *mut c_char {
        self.dest
    }

    pub const fn src(&self) -> *const c_char {
        self.src
    }

    pub const fn count(&self) -> c_int {
        self.count
    }
}

/// `CGAME_STRNCPY` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:132`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:656`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:655`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:284`
pub struct CgameStrncpy;

impl OutboundSysCall for CgameStrncpy {
    type Import = MpCgameImport;
    type Args = CgameStrncpyArgs;
    type Output = *mut c_char;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_STRNCPY;
}

impl EncodeSysCall for CgameStrncpy {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.dest()),
            ptr_to_word(args.src()),
            args.count() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgameStrncpy {
    fn decode_return(word: isize) -> Self::Output {
        word as *mut c_char
    }
}
