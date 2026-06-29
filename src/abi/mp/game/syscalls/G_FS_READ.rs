use core::ffi::c_int;

use crate::ffi::{types::fileHandle_t, GameImport};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_FS_READ` outbound game-to-engine syscall.
///
/// C ABI: `void trap_FS_Read( void *buffer, int len, fileHandle_t f )`
#[derive(Debug)]
pub struct GFsReadArgs {
    buffer: *mut u8,
    len: c_int,
    f: fileHandle_t,
}

impl GFsReadArgs {
    pub fn new(buffer: *mut u8, len: c_int, f: fileHandle_t) -> Self {
        Self { buffer, len, f }
    }

    pub fn buffer(&self) -> *mut u8 {
        self.buffer
    }

    pub fn len(&self) -> c_int {
        self.len
    }

    pub fn f(&self) -> fileHandle_t {
        self.f
    }
}

/// `G_FS_READ` MP game imports syscall ABI token.
///
/// Raven: ( void *buffer, int len, fileHandle_t f );
/// Source: `oracle/oracle/codemp/game/g_public.h:134`
pub struct GFsRead;

impl OutboundSysCall for GFsRead {
    type Import = GameImport;
    type Args = GFsReadArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_FS_READ;
}

impl EncodeSysCall for GFsRead {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.buffer), a.len as isize, a.f as isize])
    }
}

impl DecodeSysCallReturn for GFsRead {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
