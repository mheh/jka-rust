use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_SERVERSTATUS`.
///
/// Raven wrapper: `return syscall( UI_LAN_SERVERSTATUS, serverAddress, serverStatus, maxLen );`
/// Raven transport: `return LAN_GetServerStatus( (char *)VMA(1), (char *)VMA(2), args[3] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:290-291`
#[derive(Debug)]
pub struct UiLanServerstatusArgs {
    server_address: *const c_char,
    server_status: *mut c_char,
    max_len: c_int,
}

impl UiLanServerstatusArgs {
    pub fn new(server_address: *const c_char, server_status: *mut c_char, max_len: c_int) -> Self {
        Self {
            server_address,
            server_status,
            max_len,
        }
    }

    pub fn server_address(&self) -> *const c_char {
        self.server_address
    }

    pub fn server_status(&self) -> *mut c_char {
        self.server_status
    }

    pub fn max_len(&self) -> c_int {
        self.max_len
    }
}

/// `UI_LAN_SERVERSTATUS` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_SERVERSTATUS, serverAddress, serverStatus, maxLen );`
/// Raven transport: `return LAN_GetServerStatus( (char *)VMA(1), (char *)VMA(2), args[3] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:111`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:290-291`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:980`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1113-1114`
pub struct UiLanServerstatus;

impl OutboundSysCall for UiLanServerstatus {
    type Import = MpUiImport;
    type Args = UiLanServerstatusArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_SERVERSTATUS;
}

impl EncodeSysCall for UiLanServerstatus {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.server_address()),
            ptr_to_word(args.server_status()),
            args.max_len() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiLanServerstatus {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
