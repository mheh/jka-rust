use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CMD_EXECUTETEXT` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:164`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp:401-403`
/// Output source: `oracle/oracle/code/client/cl_ui.cpp:401-403`
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:401-403`
pub struct UiCmdExecutetext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCmdExecutetextArgs {
    exec_when: c_int,
    text: *const c_char,
}

impl UiCmdExecutetextArgs {
    pub const fn new(exec_when: c_int, text: *const c_char) -> Self {
        Self { exec_when, text }
    }

    pub const fn exec_when(&self) -> c_int {
        self.exec_when
    }

    pub const fn text(&self) -> *const c_char {
        self.text
    }
}

impl OutboundSysCall for UiCmdExecutetext {
    type Import = SpUiImport;
    type Args = UiCmdExecutetextArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CMD_EXECUTETEXT;
}

impl EncodeSysCall for UiCmdExecutetext {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.exec_when() as isize, ptr_to_word(args.text())])
    }
}

impl DecodeSysCallReturn for UiCmdExecutetext {
    fn decode_return(_word: isize) -> Self::Output {}
}
