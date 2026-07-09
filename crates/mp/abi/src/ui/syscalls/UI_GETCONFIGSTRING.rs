use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_GETCONFIGSTRING`.
///
/// Raven wrapper: `syscall( UI_GETCONFIGSTRING, index, buff, buffsize );`
/// Raven transport: `return GetConfigString( args[1], (char *)VMA(2), args[3] );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:266-267`
#[derive(Debug)]
pub struct UiGetconfigstringArgs {
    index: c_int,
    buff: *mut c_char,
    buffsize: c_int,
}

impl UiGetconfigstringArgs {
    pub fn new(index: c_int, buff: *mut c_char, buffsize: c_int) -> Self {
        Self {
            index,
            buff,
            buffsize,
        }
    }

    pub fn index(&self) -> c_int {
        self.index
    }

    pub fn buff(&self) -> *mut c_char {
        self.buff
    }

    pub fn buffsize(&self) -> c_int {
        self.buffsize
    }
}

/// `UI_GETCONFIGSTRING` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_GETCONFIGSTRING, index, buff, buffsize );`
/// Raven transport: `return GetConfigString( args[1], (char *)VMA(2), args[3] );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:64`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:266-267`
/// Output source: `oracle/codemp/ui/ui_local.h:963`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1052-1053`
pub struct UiGetconfigstring;

impl OutboundSysCall for UiGetconfigstring {
    type Import = MpUiImport;
    type Args = UiGetconfigstringArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_GETCONFIGSTRING;
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
