use core::ffi::{c_int, CStr};
use std::ffi::CString;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::fsMode_t;
use crate::ffi::{types::fileHandle_t, GameImport};

/// `G_FS_FOPEN_FILE` outbound game-to-engine syscall.
///
/// C ABI: `int trap_FS_FOpenFile( const char *qpath, fileHandle_t *f, fsMode_t mode )`
///
/// The engine writes the opened file handle into `*f` and returns the file
/// length (or -1 on failure).  `f` is an out-param raw pointer and is kept as
/// such in `Args`; callers are responsible for providing a valid `*mut
/// fileHandle_t`.
#[derive(Debug)]
pub struct GFsFopenFileArgs {
    /// Null-terminated virtual-filesystem path.
    qpath: CString,
    /// Pointer to the `fileHandle_t` the engine will fill in.
    f: *mut fileHandle_t,
    /// Open mode (`FS_READ`, `FS_WRITE`, …).
    mode: fsMode_t,
}

impl GFsFopenFileArgs {
    /// Construct the args.
    ///
    /// # Safety
    /// `f` must point to a valid, writable `fileHandle_t` that outlives the
    /// syscall.
    pub unsafe fn new(qpath: CString, f: *mut fileHandle_t, mode: fsMode_t) -> Self {
        Self { qpath, f, mode }
    }

    pub fn qpath(&self) -> &CStr {
        self.qpath.as_c_str()
    }

    pub fn f(&self) -> *mut fileHandle_t {
        self.f
    }

    pub fn mode(&self) -> fsMode_t {
        self.mode
    }
}

/// `G_FS_FOPEN_FILE` MP game imports syscall ABI token.
///
/// Raven: ( const char *qpath, fileHandle_t *file, fsMode_t mode );
/// Source: `oracle/oracle/codemp/game/g_public.h:133`
pub struct GFsFopenFile;

impl OutboundSysCall for GFsFopenFile {
    type Import = GameImport;
    type Args = GFsFopenFileArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_FS_FOPEN_FILE;
}

impl EncodeSysCall for GFsFopenFile {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.qpath.as_ptr()),
            ptr_to_word(a.f),
            a.mode as isize,
        ])
    }
}

impl DecodeSysCallReturn for GFsFopenFile {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
