use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_GET_SERVERINFO` outbound game-to-engine syscall.
///
/// C ABI: `void trap_GetServerinfo(char *buffer, int bufferSize)`
/// The engine writes the serverinfo string into the caller-provided buffer.
#[derive(Debug)]
pub struct GGetServerinfoArgs {
    /// Caller-owned output buffer; engine writes the serverinfo string here.
    buf: *mut c_char,
    /// Capacity of `buf` in bytes.
    size: c_int,
}

impl GGetServerinfoArgs {
    pub fn new(buf: *mut c_char, size: c_int) -> Self {
        Self { buf, size }
    }

    pub fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub fn size(&self) -> c_int {
        self.size
    }
}

/// `G_GET_SERVERINFO` MP game imports syscall ABI token.
///
/// Raven: ( char *buffer, int bufferSize );
/// Raven: the serverinfo info string has all the cvars visible to server browsers
/// Source: `oracle/oracle/codemp/game/g_public.h:173`
pub struct GGetServerinfo;

impl OutboundSysCall for GGetServerinfo {
    type Import = MpGameImport;
    type Args = GGetServerinfoArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_GET_SERVERINFO;
}

impl EncodeSysCall for GGetServerinfo {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.buf), (a.size as isize)])
    }
}

impl DecodeSysCallReturn for GGetServerinfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
