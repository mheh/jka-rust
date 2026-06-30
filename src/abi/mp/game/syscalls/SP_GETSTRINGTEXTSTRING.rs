use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `SP_GETSTRINGTEXTSTRING` outbound game-to-engine syscall.
///
/// C: `int trap_SP_GetStringTextString(const char *text, char *buffer, int bufferLength)`
#[derive(Debug)]
pub struct SpGetstringtextstringArgs {
    text: *const c_char,
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl SpGetstringtextstringArgs {
    pub fn new(text: *const c_char, buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            text,
            buffer,
            buffer_length,
        }
    }

    pub fn text(&self) -> *const c_char {
        self.text
    }

    pub fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub fn buffer_length(&self) -> c_int {
        self.buffer_length
    }
}

/// `SP_GETSTRINGTEXTSTRING` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:239`
pub struct SpGetstringtextstring;

impl OutboundSysCall for SpGetstringtextstring {
    type Import = MpGameImport;
    type Args = SpGetstringtextstringArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::SP_GETSTRINGTEXTSTRING;
}

impl EncodeSysCall for SpGetstringtextstring {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.text),
            ptr_to_word(a.buffer),
            a.buffer_length as isize,
        ])
    }
}

impl DecodeSysCallReturn for SpGetstringtextstring {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
