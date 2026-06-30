use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::shared::fileHandle_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for the `G_FS_WRITE` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(G_FS_WRITE, buffer.as_ptr(), buffer.len() as i32, f)`:
/// - `buf`  — pointer to the bytes to write (`const void *buffer`)
/// - `len`  — number of bytes (`int len`)
/// - `f`    — open file handle (`fileHandle_t`)
#[derive(Debug)]
pub struct GFsWriteArgs {
    buf: *const u8,
    len: c_int,
    f: fileHandle_t,
}

impl GFsWriteArgs {
    pub fn new(buf: *const u8, len: c_int, f: fileHandle_t) -> Self {
        Self { buf, len, f }
    }

    pub fn buf(&self) -> *const u8 {
        self.buf
    }

    pub fn len(&self) -> c_int {
        self.len
    }

    pub fn f(&self) -> fileHandle_t {
        self.f
    }
}

/// `G_FS_WRITE` MP game imports syscall ABI token.
///
/// Raven: ( const void *buffer, int len, fileHandle_t f );
/// Source: `oracle/oracle/codemp/game/g_public.h:135`
pub struct GFsWrite;

impl OutboundSysCall for GFsWrite {
    type Import = GameImport;
    type Args = GFsWriteArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_FS_WRITE;
}

impl EncodeSysCall for GFsWrite {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.buf), a.len as isize, a.f as isize])
    }
}

impl DecodeSysCallReturn for GFsWrite {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
