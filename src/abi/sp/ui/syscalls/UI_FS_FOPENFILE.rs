use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::fsMode_t;
use crate::shared::fileHandle_t;

/// `UI_FS_FOPENFILE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:165`
/// Args source: `oracle/oracle/code/ui/ui_public.h:36`
/// Output source: `oracle/oracle/code/ui/ui_public.h:36` (engine contract), `oracle/oracle/codemp/client/cl_ui.cpp:914-915`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:914-915`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFsFopenfileArgs {
    qpath: *const c_char,
    file: *mut fileHandle_t,
    mode: fsMode_t,
}

impl UiFsFopenfileArgs {
    /// # Safety
    /// `qpath` must be a valid NUL-terminated C string and `file` must point to writable
    /// storage for a `fileHandle_t` during the syscall.
    pub const unsafe fn new(qpath: *const c_char, file: *mut fileHandle_t, mode: fsMode_t) -> Self {
        Self { qpath, file, mode }
    }

    pub const fn qpath(&self) -> *const c_char {
        self.qpath
    }

    pub const fn file(&self) -> *mut fileHandle_t {
        self.file
    }

    pub const fn mode(&self) -> fsMode_t {
        self.mode
    }
}
pub struct UiFsFopenfile;

impl OutboundSysCall for UiFsFopenfile {
    type Import = SpUiImport;
    type Args = UiFsFopenfileArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_FS_FOPENFILE;
}

impl EncodeSysCall for UiFsFopenfile {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.qpath()),
            ptr_to_word(args.file()),
            args.mode() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiFsFopenfile {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
