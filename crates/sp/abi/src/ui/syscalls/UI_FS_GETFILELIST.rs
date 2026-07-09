use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_FS_GETFILELIST` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/code/ui/ui_public.h:169`
/// Args source: `oracle/code/ui/ui_public.h:40`
/// Output source: `oracle/code/ui/ui_public.h:40`
/// Transport/switch source:
/// - SP `oracle/code/client/cl_ui.cpp:408`
/// - MP fallback `oracle/codemp/client/cl_ui.cpp:929-930`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFsGetfilelistArgs {
    path: *const c_char,
    extension: *const c_char,
    listbuf: *mut c_char,
    bufsize: c_int,
}

impl UiFsGetfilelistArgs {
    /// # Safety
    /// `path` and `extension` must be NUL-terminated, and `listbuf` must be writable for `bufsize` bytes.
    pub const unsafe fn new(
        path: *const c_char,
        extension: *const c_char,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> Self {
        Self {
            path,
            extension,
            listbuf,
            bufsize,
        }
    }

    pub const fn path(&self) -> *const c_char {
        self.path
    }

    pub const fn extension(&self) -> *const c_char {
        self.extension
    }

    pub const fn listbuf(&self) -> *mut c_char {
        self.listbuf
    }

    pub const fn bufsize(&self) -> c_int {
        self.bufsize
    }
}

pub struct UiFsGetfilelist;

impl OutboundSysCall for UiFsGetfilelist {
    type Import = SpUiImport;
    type Args = UiFsGetfilelistArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_FS_GETFILELIST;
}

impl EncodeSysCall for UiFsGetfilelist {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.path()),
            ptr_to_word(args.extension()),
            ptr_to_word(args.listbuf()),
            args.bufsize() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiFsGetfilelist {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
