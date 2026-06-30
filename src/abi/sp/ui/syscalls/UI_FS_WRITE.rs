use core::ffi::{c_int, c_void};

use super::super::SpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::fileHandle_t;

/// `UI_FS_WRITE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:167`
/// Args source: `oracle/oracle/code/ui/ui_public.h:38`
/// Output source:
/// - `oracle/oracle/code/ui/ui_public.h:38` declares `int` return
/// - `oracle/oracle/codemp/client/cl_ui.cpp:921-923` returns `0`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:921-923`
/// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` has no `UI_FS_WRITE` case in this branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFsWriteArgs {
    buffer: *const c_void,
    len: c_int,
    file: fileHandle_t,
}

impl UiFsWriteArgs {
    pub const fn new(buffer: *const c_void, len: c_int, file: fileHandle_t) -> Self {
        Self { buffer, len, file }
    }

    pub const fn buffer(&self) -> *const c_void {
        self.buffer
    }

    pub const fn len(&self) -> c_int {
        self.len
    }

    pub const fn file(&self) -> fileHandle_t {
        self.file
    }
}
pub struct UiFsWrite;

impl OutboundSysCall for UiFsWrite {
    type Import = SpUiImport;
    type Args = UiFsWriteArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_FS_WRITE;
}

impl EncodeSysCall for UiFsWrite {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.buffer()),
            args.len() as isize,
            args.file() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiFsWrite {
    fn decode_return(_word: isize) -> Self::Output {}
}
