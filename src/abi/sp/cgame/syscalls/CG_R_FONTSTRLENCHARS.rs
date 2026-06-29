use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_FONTSTRLENCHARS`.
///
/// Raven wrapper: `return syscall( CG_R_FONTSTRLENCHARS, text );`
/// Raven transport: `return re.Font_StrLenChars((const char *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:329-330`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:667-668`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRFontstrlencharsArgs {
    text: *const c_char,
}

impl CgRFontstrlencharsArgs {
    pub const fn new(text: *const c_char) -> Self {
        Self { text }
    }

    pub const fn text(&self) -> *const c_char {
        self.text
    }
}

/// `CG_R_FONTSTRLENCHARS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:124`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:329-330`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:667-668`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:667-668`
pub struct CgRFontstrlenchars;

impl OutboundSysCall for CgRFontstrlenchars {
    type Import = SpCgameImport;
    type Args = CgRFontstrlencharsArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_FONTSTRLENCHARS;
}

impl EncodeSysCall for CgRFontstrlenchars {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.text())])
    }
}

impl DecodeSysCallReturn for CgRFontstrlenchars {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
