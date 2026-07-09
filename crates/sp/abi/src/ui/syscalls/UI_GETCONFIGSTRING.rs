use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::{c_char, c_int};

/// `UI_GETCONFIGSTRING` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:197`
pub struct UiGetconfigstring;

#[derive(Debug)]
pub struct UiGetconfigstringArgs {
    index: c_int,
    buff: *mut c_char,
    buffsize: c_int,
}

impl UiGetconfigstringArgs {
    pub const fn new(index: c_int, buff: *mut c_char, buffsize: c_int) -> Self {
        Self {
            index,
            buff,
            buffsize,
        }
    }

    pub const fn index(&self) -> c_int {
        self.index
    }

    pub const fn buff(&self) -> *mut c_char {
        self.buff
    }

    pub const fn buffsize(&self) -> c_int {
        self.buffsize
    }
}

impl OutboundSysCall for UiGetconfigstring {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_GETCONFIGSTRING, index, buff, buffsize );`
    ///
    /// Args source: `oracle/code/client/cl_ui.cpp:141`
    /// Output source: `oracle/code/client/cl_ui.cpp:141`
    ///
    /// Transport/import source: `oracle/code/client/cl_ui.cpp:293`
    type Args = UiGetconfigstringArgs;
    /// `GetConfigString` fills the provided buffer and returns `qboolean` in C helper.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_GETCONFIGSTRING;
}

impl EncodeSysCall for UiGetconfigstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.index() as isize,
            ptr_to_word(args.buff()),
            args.buffsize() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiGetconfigstring {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
