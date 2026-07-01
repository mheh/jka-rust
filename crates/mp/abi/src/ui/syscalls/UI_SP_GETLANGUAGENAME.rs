use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_SP_GETLANGUAGENAME`.
///
/// Raven wrapper: `syscall( UI_SP_GETLANGUAGENAME, languageIndex, buffer);`
/// Raven transport: `Q_strncpyz( holdName, languageName,128 ); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:443-445`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1213-1219`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSpGetlanguagenameArgs {
    language_index: c_int,
    buffer: *mut c_char,
}

impl UiSpGetlanguagenameArgs {
    pub const fn new(language_index: c_int, buffer: *mut c_char) -> Self {
        Self {
            language_index,
            buffer,
        }
    }

    pub const fn language_index(&self) -> c_int {
        self.language_index
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }
}

/// `UI_SP_GETLANGUAGENAME` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:136`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:443-445`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1213-1219`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1213-1219`
pub struct UiSpGetlanguagename;

impl OutboundSysCall for UiSpGetlanguagename {
    type Import = MpUiImport;
    type Args = UiSpGetlanguagenameArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_SP_GETLANGUAGENAME;
}

impl EncodeSysCall for UiSpGetlanguagename {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.language_index() as isize, ptr_to_word(args.buffer())])
    }
}

impl DecodeSysCallReturn for UiSpGetlanguagename {
    fn decode_return(_word: isize) -> Self::Output {}
}
