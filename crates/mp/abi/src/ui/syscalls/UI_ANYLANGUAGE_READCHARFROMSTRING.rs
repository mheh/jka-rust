use core::ffi::{c_char, c_int, c_uint};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `UI_ANYLANGUAGE_READCHARFROMSTRING`.
///
/// Raven wrapper: `return syscall( UI_ANYLANGUAGE_READCHARFROMSTRING, psText, piAdvanceCount, pbIsTrailingPunctuation);`
/// Raven transport: `return re.AnyLanguage_ReadCharFromString( (const char *)VMA(1), (int *) VMA(2), (qboolean *) VMA(3) );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:146-148`
/// Args source: `oracle/codemp/ui/ui_local.h:999`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1154-1155`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiAnylanguageReadcharfromstringArgs {
    ps_text: *const c_char,
    pi_advance_count: *mut c_int,
    pb_is_trailing_punctuation: *mut qboolean,
}

impl UiAnylanguageReadcharfromstringArgs {
    pub const fn new(
        ps_text: *const c_char,
        pi_advance_count: *mut c_int,
        pb_is_trailing_punctuation: *mut qboolean,
    ) -> Self {
        Self {
            ps_text,
            pi_advance_count,
            pb_is_trailing_punctuation,
        }
    }

    pub const fn ps_text(&self) -> *const c_char {
        self.ps_text
    }

    pub const fn pi_advance_count(&self) -> *mut c_int {
        self.pi_advance_count
    }

    pub const fn pb_is_trailing_punctuation(&self) -> *mut qboolean {
        self.pb_is_trailing_punctuation
    }
}

/// `UI_ANYLANGUAGE_READCHARFROMSTRING` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:82`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:146-148`
/// Output source: `oracle/codemp/ui/ui_local.h:999`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1154-1155`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1154-1155`
pub struct UiAnylanguageReadcharfromstring;

impl OutboundSysCall for UiAnylanguageReadcharfromstring {
    type Import = MpUiImport;
    type Args = UiAnylanguageReadcharfromstringArgs;
    type Output = c_uint;

    const IMPORT: MpUiImport = MpUiImport::UI_ANYLANGUAGE_READCHARFROMSTRING;
}

impl EncodeSysCall for UiAnylanguageReadcharfromstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ps_text()),
            ptr_to_word(args.pi_advance_count()),
            ptr_to_word(args.pb_is_trailing_punctuation()),
        ])
    }
}

impl DecodeSysCallReturn for UiAnylanguageReadcharfromstring {
    fn decode_return(word: isize) -> Self::Output {
        word as c_uint
    }
}
