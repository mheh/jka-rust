use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_ENDPARSESESSION`.
///
/// Raven wrapper: `cgi_UI_EndParseSession(buf);`
/// Raven transport: `PC_EndParseSession((char *) VMA(1)); return 0;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:598-600`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:872-874`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiEndparsesessionArgs {
    buf: *mut c_char,
}

impl CgUiEndparsesessionArgs {
    /// # Safety
    /// `buf` must be valid for the parser API this call receives.
    pub const unsafe fn new(buf: *mut c_char) -> Self {
        Self { buf }
    }

    pub const fn buf(&self) -> *mut c_char {
        self.buf
    }
}

/// `CG_UI_ENDPARSESESSION` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:200`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:598-600`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:872-874`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:872-874`
pub struct CgUiEndparsesession;

impl OutboundSysCall for CgUiEndparsesession {
    type Import = SpCgameImport;
    type Args = CgUiEndparsesessionArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_ENDPARSESESSION;
}

impl EncodeSysCall for CgUiEndparsesession {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buf())])
    }
}

impl DecodeSysCallReturn for CgUiEndparsesession {
    fn decode_return(_word: isize) -> Self::Output {}
}
