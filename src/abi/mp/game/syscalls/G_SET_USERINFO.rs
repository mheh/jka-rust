use core::ffi::{c_char, c_int};
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_SET_USERINFO` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GSetUserinfoArgs {
    /// Client number.
    num: c_int,
    /// Userinfo string (NUL-terminated, kept alive for the call duration).
    info: CString,
}

impl GSetUserinfoArgs {
    pub fn new(num: c_int, info: CString) -> Self {
        Self { num, info }
    }

    pub fn num(&self) -> c_int {
        self.num
    }

    pub fn info(&self) -> *const c_char {
        self.info.as_ptr()
    }
}

/// `G_SET_USERINFO` MP game imports syscall ABI token.
///
/// Raven: ( int num, const char *buffer );
/// Source: `oracle/oracle/codemp/game/g_public.h:171`
pub struct GSetUserinfo;

impl OutboundSysCall for GSetUserinfo {
    type Import = GameImport;
    type Args = GSetUserinfoArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_SET_USERINFO;
}

impl EncodeSysCall for GSetUserinfo {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.num() as isize, ptr_to_word(a.info())])
    }
}

impl DecodeSysCallReturn for GSetUserinfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
