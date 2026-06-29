use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_PARSE_INT`.
///
/// Raven wrapper: `cgi_UI_Parse_Int(value);`
/// Raven transport: `PC_ParseInt((int *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:578-580`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:857-859`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiParseIntArgs {
    value: *mut c_int,
}

impl CgUiParseIntArgs {
    /// # Safety
    /// `value` must be valid for writes of an `int`.
    pub const unsafe fn new(value: *mut c_int) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> *mut c_int {
        self.value
    }
}

/// `CG_UI_PARSE_INT` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:196`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:578-580`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:857-859`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:857-859`
pub struct CgUiParseInt;

impl OutboundSysCall for CgUiParseInt {
    type Import = SpCgameImport;
    type Args = CgUiParseIntArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_PARSE_INT;
}

impl EncodeSysCall for CgUiParseInt {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.value())])
    }
}

impl DecodeSysCallReturn for CgUiParseInt {
    fn decode_return(_word: isize) -> Self::Output {}
}
