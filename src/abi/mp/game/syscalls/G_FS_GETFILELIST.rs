use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_FS_GETFILELIST` outbound game-to-engine syscall.
///
/// C ABI: `int trap_FS_GetFileList(const char *path, const char *extension, char *listbuf, int bufsize)`
#[derive(Debug)]
pub struct GFsGetfilelistArgs {
    path: CString,
    extension: CString,
    listbuf: *mut u8,
    bufsize: c_int,
}

impl GFsGetfilelistArgs {
    pub fn new(path: CString, extension: CString, listbuf: *mut u8, bufsize: c_int) -> Self {
        Self {
            path,
            extension,
            listbuf,
            bufsize,
        }
    }

    pub fn path(&self) -> &CString {
        &self.path
    }

    pub fn extension(&self) -> &CString {
        &self.extension
    }

    pub fn listbuf(&self) -> *mut u8 {
        self.listbuf
    }

    pub fn bufsize(&self) -> c_int {
        self.bufsize
    }
}

/// `G_FS_GETFILELIST` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:229`
pub struct GFsGetfilelist;

impl OutboundSysCall for GFsGetfilelist {
    type Import = MpGameImport;
    type Args = GFsGetfilelistArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_FS_GETFILELIST;
}

impl EncodeSysCall for GFsGetfilelist {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.path.as_ptr()),
            ptr_to_word(a.extension.as_ptr()),
            ptr_to_word(a.listbuf),
            a.bufsize as isize,
        ])
    }
}

impl DecodeSysCallReturn for GFsGetfilelist {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
