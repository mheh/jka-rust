use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `UI_SP_GETSTRINGTEXTSTRING`.
///
/// Raven wrapper: `return syscall( UI_SP_GETSTRINGTEXTSTRING, text, buffer, bufferLength );`
/// Raven transport: `Q_strncpyz( (char *) VMA(2), text, args[3] ); return qtrue;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:448-450`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1221-1228`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSpGetstringtextstringArgs {
    text: *const c_char,
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl UiSpGetstringtextstringArgs {
    pub const fn new(text: *const c_char, buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            text,
            buffer,
            buffer_length,
        }
    }

    pub const fn text(&self) -> *const c_char {
        self.text
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn buffer_length(&self) -> c_int {
        self.buffer_length
    }
}

/// `UI_SP_GETSTRINGTEXTSTRING` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:137`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:448-450`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1221-1228`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1221-1228`
pub struct UiSpGetstringtextstring;

impl OutboundSysCall for UiSpGetstringtextstring {
    type Import = MpUiImport;
    type Args = UiSpGetstringtextstringArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_SP_GETSTRINGTEXTSTRING;
}

impl EncodeSysCall for UiSpGetstringtextstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.text()),
            ptr_to_word(args.buffer()),
            args.buffer_length() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiSpGetstringtextstring {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
