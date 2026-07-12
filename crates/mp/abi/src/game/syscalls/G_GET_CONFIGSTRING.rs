use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_GET_CONFIGSTRING` outbound game-to-engine syscall.
///
/// ABI: `trap_GetConfigstring(int num, char *buf, int buflen)` — returns void;
/// the engine writes the configstring into the caller-supplied buffer.
#[derive(Debug)]
pub struct GGetConfigstringArgs {
    /// Configstring index.
    num: c_int,
    /// Caller-allocated output buffer; engine writes into it.
    buf: *mut c_char,
    /// Size of `buf` in bytes.
    buf_len: c_int,
}

impl GGetConfigstringArgs {
    pub fn new(num: c_int, buf: *mut c_char, buf_len: c_int) -> Self {
        Self { num, buf, buf_len }
    }

    pub fn num(&self) -> c_int {
        self.num
    }

    pub fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub fn buf_len(&self) -> c_int {
        self.buf_len
    }
}

/// `G_GET_CONFIGSTRING` MP game imports syscall ABI token.
///
/// Raven: ( int num, char *buffer, int bufferSize );
/// Source: `oracle/codemp/game/g_public.h:164`
pub struct GGetConfigstring;

impl OutboundSysCall for GGetConfigstring {
    type Import = MpGameImport;
    type Args = GGetConfigstringArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_GET_CONFIGSTRING;
}

impl EncodeSysCall for GGetConfigstring {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.num as isize, ptr_to_word(a.buf), a.buf_len as isize])
    }
}

impl DecodeSysCallReturn for GGetConfigstring {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
