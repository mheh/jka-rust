use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::fsMode_t;
use crate::ffi::types::fileHandle_t;

/// Arguments for `UI_FS_FOPENFILE`.
///
/// C ABI: `int trap_FS_FOpenFile(const char *qpath, fileHandle_t *f, fsMode_t mode)`.
/// Raven's client switch decodes `qpath` and `f` through `VMA`, then passes the
/// raw mode word as `fsMode_t`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:83-84`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:737-738`
#[derive(Debug)]
pub struct CgFsFopenfileArgs {
    /// Null-terminated virtual-filesystem path, decoded by Raven as `VMA(1)`.
    qpath: *const c_char,
    /// Out pointer to the `fileHandle_t` slot, decoded by Raven as `VMA(2)`.
    f: *mut fileHandle_t,
    /// Open mode, read by Raven as `args[3]`.
    mode: fsMode_t,
}

impl CgFsFopenfileArgs {
    /// Construct raw `trap_FS_FOpenFile` syscall args.
    ///
    /// # Safety
    /// `qpath` must point to a valid NUL-terminated C string, and `f` must point
    /// to a writable `fileHandle_t` slot for the duration of the syscall.
    pub const unsafe fn new(qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> Self {
        Self { qpath, f, mode }
    }

    pub const fn qpath(&self) -> *const c_char {
        self.qpath
    }

    pub const fn f(&self) -> *mut fileHandle_t {
        self.f
    }

    pub const fn mode(&self) -> fsMode_t {
        self.mode
    }
}

/// `UI_FS_FOPENFILE` MP cgame imports syscall boundary token.
///
/// Raven wrapper: `return syscall( UI_FS_FOPENFILE, qpath, f, mode );`
/// Raven transport: `return FS_FOpenFileByMode( (const char *)VMA(1), (int *)VMA(2), (fsMode_t)args[3] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:73`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:83-84`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:737-738`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:737-738`
pub struct CgFsFopenfile;

impl OutboundSysCall for CgFsFopenfile {
    type Import = MpUiImport;
    type Args = CgFsFopenfileArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_FS_FOPENFILE;
}

impl EncodeSysCall for CgFsFopenfile {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.qpath()),
            ptr_to_word(args.f()),
            args.mode() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFsFopenfile {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
