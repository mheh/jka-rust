use core::ffi::{c_int, c_void};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::fileHandle_t;

/// `UI_FS_READ` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:166`
/// Args source: `oracle/oracle/code/ui/ui_public.h:37`
/// Output source:
/// - `oracle/oracle/code/ui/ui_public.h:37` declares `int` return
/// - `oracle/oracle/codemp/client/cl_ui.cpp:917-919` returns `0` (unit)
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:917-919`
/// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` has no `UI_FS_READ` case in this branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFsReadArgs {
    buffer: *mut c_void,
    len: c_int,
    file: fileHandle_t,
}

impl UiFsReadArgs {
    /// # Safety
    /// `buffer` must be valid for writes of up to `len` bytes.
    pub const unsafe fn new(buffer: *mut c_void, len: c_int, file: fileHandle_t) -> Self {
        Self { buffer, len, file }
    }

    pub const fn buffer(&self) -> *mut c_void {
        self.buffer
    }

    pub const fn len(&self) -> c_int {
        self.len
    }

    pub const fn file(&self) -> fileHandle_t {
        self.file
    }
}
pub struct UiFsRead;

impl OutboundSysCall for UiFsRead {
    type Import = SpUiImport;
    type Args = UiFsReadArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_FS_READ;
}

impl EncodeSysCall for UiFsRead {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.buffer()),
            args.len() as isize,
            args.file() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiFsRead {
    fn decode_return(_word: isize) -> Self::Output {}
}
