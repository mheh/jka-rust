use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_GET_USERINFO` outbound game-to-engine syscall.
///
/// Mirrors: `syscall!(G_GET_USERINFO, num, buf.as_mut_ptr(), buf.len() as i32)`
/// The engine writes the userinfo string into the caller-supplied buffer; there
/// is no meaningful integer return value (the C side returns void).
#[derive(Debug)]
pub struct GGetUserinfoArgs {
    /// Client slot number.
    client_num: c_int,
    /// Caller-allocated output buffer the engine writes into.
    buf: *mut c_char,
    /// Capacity of `buf` in bytes.
    buf_size: c_int,
}

impl GGetUserinfoArgs {
    pub fn new(client_num: c_int, buf: *mut c_char, buf_size: c_int) -> Self {
        Self {
            client_num,
            buf,
            buf_size,
        }
    }

    pub fn client_num(&self) -> c_int {
        self.client_num
    }
    pub fn buf(&self) -> *mut c_char {
        self.buf
    }
    pub fn buf_size(&self) -> c_int {
        self.buf_size
    }
}

/// `G_GET_USERINFO` MP game imports syscall ABI token.
///
/// Raven: ( int num, char *buffer, int bufferSize );
/// Raven: userinfo strings are maintained by the server system, so they
/// Raven: are persistant across level loads, while all other game visible
/// Raven: data is completely reset
/// Source: `oracle/codemp/game/g_public.h:166`
pub struct GGetUserinfo;

impl OutboundSysCall for GGetUserinfo {
    type Import = MpGameImport;
    type Args = GGetUserinfoArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_GET_USERINFO;
}

impl EncodeSysCall for GGetUserinfo {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.client_num as isize,
            ptr_to_word(a.buf),
            a.buf_size as isize,
        ])
    }
}

impl DecodeSysCallReturn for GGetUserinfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
