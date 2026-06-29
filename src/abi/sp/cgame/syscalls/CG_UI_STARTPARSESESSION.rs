use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_STARTPARSESESSION`.
///
/// Raven wrapper: `cgi_UI_StartParseSession(menuFile, buf)`
/// Raven transport: `return(PC_StartParseSession((char *) VMA(1),(char **) VMA(2)));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:593-595`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:869-871`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiStartparsesessionArgs {
    menu_file: *const c_char,
    buf: *mut *mut c_char,
}

impl CgUiStartparsesessionArgs {
    pub const fn new(menu_file: *const c_char, buf: *mut *mut c_char) -> Self {
        Self { menu_file, buf }
    }

    pub const fn menu_file(&self) -> *const c_char {
        self.menu_file
    }

    pub const fn buf(&self) -> *mut *mut c_char {
        self.buf
    }
}

/// `CG_UI_STARTPARSESESSION` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:199`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:593-595`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:869-871`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:869-871`
pub struct CgUiStartparsesession;

impl OutboundSysCall for CgUiStartparsesession {
    type Import = SpCgameImport;
    type Args = CgUiStartparsesessionArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_STARTPARSESESSION;
}

impl EncodeSysCall for CgUiStartparsesession {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.menu_file()), ptr_to_word(args.buf())])
    }
}

impl DecodeSysCallReturn for CgUiStartparsesession {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
