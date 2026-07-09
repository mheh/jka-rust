use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::pc_token_t;

/// Arguments for `UI_PC_READ_TOKEN`.
///
/// Raven wrapper: `syscall( UI_PC_READ_TOKEN, handle, pc_token );`
/// Raven transport: `return botlib_export->PC_ReadTokenHandle( args[1], (struct pc_token_s *)VMA(2) );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:374-375`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1163-1164`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiPcReadTokenArgs {
    handle: c_int,
    pc_token: *mut pc_token_t,
}

impl UiPcReadTokenArgs {
    pub const fn new(handle: c_int, pc_token: *mut pc_token_t) -> Self {
        Self { handle, pc_token }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }

    pub const fn pc_token(&self) -> *mut pc_token_t {
        self.pc_token
    }
}

/// `UI_PC_READ_TOKEN` MP UI imports syscall ABI token.
///
/// Raven wrapper: `int trap_PC_ReadToken( int handle, pc_token_t *pc_token ) { return syscall( UI_PC_READ_TOKEN, handle, pc_token ); }`
/// Raven transport: `return botlib_export->PC_ReadTokenHandle( args[1], (struct pc_token_s *)VMA(2) );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:87`
/// Enum comment source: `oracle/codemp/ui/ui_public.h:82-90`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:374-375`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1163-1164`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1163-1164`
pub struct UiPcReadToken;

impl OutboundSysCall for UiPcReadToken {
    type Import = MpUiImport;
    type Args = UiPcReadTokenArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_PC_READ_TOKEN;
}

impl EncodeSysCall for UiPcReadToken {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize, ptr_to_word(args.pc_token())])
    }
}

impl DecodeSysCallReturn for UiPcReadToken {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
