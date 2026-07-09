use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FS_GETFILELIST`.
///
/// C ABI: `int trap_FS_GetFileList(const char *path, const char *extension, char *listbuf, int bufsize)`.
/// Raven's client switch decodes the three pointer-shaped args through `VMA`
/// and reads the buffer size as the raw `args[4]` word. The returned `int` is
/// the file count; `listbuf` is an out buffer filled by the engine.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:99-100`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:748-749`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:748-749`
#[derive(Debug)]
pub struct CgFsGetfilelistArgs {
    /// Null-terminated virtual-filesystem path, decoded by Raven as `VMA(1)`.
    path: *const c_char,
    /// Null-terminated extension filter, decoded by Raven as `VMA(2)`.
    extension: *const c_char,
    /// Writable output buffer, decoded by Raven as `VMA(3)`.
    listbuf: *mut c_char,
    /// Output buffer size in bytes, read by Raven as `args[4]`.
    bufsize: c_int,
}

impl CgFsGetfilelistArgs {
    /// Construct raw `trap_FS_GetFileList` syscall args.
    ///
    /// # Safety
    /// `path` and `extension` must point to valid NUL-terminated C strings, and
    /// `listbuf` must point to a writable buffer of at least `bufsize` bytes for
    /// the duration of the syscall.
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

/// `CG_FS_GETFILELIST` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall( CG_FS_GETFILELIST, path, extension, listbuf, bufsize );`
/// Raven transport: `return FS_GetFileList( (const char *)VMA(1), (const char *)VMA(2), (char *)VMA(3), args[4] );`
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:77`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:99-100`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:748-749`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:748-749`
pub struct CgFsGetfilelist;

impl OutboundSysCall for CgFsGetfilelist {
    type Import = MpCgameImport;
    type Args = CgFsGetfilelistArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_GETFILELIST;
}

impl EncodeSysCall for CgFsGetfilelist {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.path()),
            ptr_to_word(args.extension()),
            ptr_to_word(args.listbuf()),
            args.bufsize() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFsGetfilelist {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
