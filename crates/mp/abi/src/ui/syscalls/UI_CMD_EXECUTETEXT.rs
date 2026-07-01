use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CMD_EXECUTETEXT`.
///
/// Raven wrapper: `syscall( UI_CMD_EXECUTETEXT, exec_when, text );`
/// Raven transport: `Cbuf_ExecuteText( args[1], (const char *)VMA(2) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:79-80`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:929`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:910-912`
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

/// `UI_CMD_EXECUTETEXT` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:30`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:79-80`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:79-80`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:910-912`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:910-912`
pub struct UiCmdExecutetext;

impl OutboundSysCall for UiCmdExecutetext {
    type Import = MpUiImport;
    type Args = UiCmdExecutetextArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CMD_EXECUTETEXT;
}

impl EncodeSysCall for UiCmdExecutetext {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.exec_when() as isize, ptr_to_word(args.text())])
    }
}

impl DecodeSysCallReturn for UiCmdExecutetext {
    fn decode_return(_word: isize) -> Self::Output {}
}
