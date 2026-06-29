use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::abi::generic::ptr_to_word;

/// Arguments for `UI_LAN_SERVERSTATUS`.
///
/// Raven wrapper: `return syscall( UI_LAN_SERVERSTATUS, serverAddress, serverStatus, maxLen );`
/// Raven transport: `return LAN_GetServerStatus( (char *)VMA(1), (char *)VMA(2), args[3] );`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:234`
/// Args source (SP fallback): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_SERVERSTATUS`.
/// Args source (fallback): `oracle/oracle/codemp/ui/ui_local.h:980`
/// Transport/switch source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:1113-1114`
/// Output source fallback: `oracle/oracle/codemp/ui/ui_local.h:980`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLanServerstatusArgs {
    server_address: *const c_char,
    server_status: *mut c_char,
    max_len: c_int,
}

impl UiLanServerstatusArgs {
    pub const fn new(server_address: *const c_char, server_status: *mut c_char, max_len: c_int) -> Self {
        Self {
            server_address,
            server_status,
            max_len,
        }
    }

    pub const fn server_address(&self) -> *const c_char {
        self.server_address
    }

    pub const fn server_status(&self) -> *mut c_char {
        self.server_status
    }

    pub const fn max_len(&self) -> c_int {
        self.max_len
    }
}

/// `UI_LAN_SERVERSTATUS` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:234`
pub struct UiLanServerstatus;

impl OutboundSysCall for UiLanServerstatus {
    type Import = SpUiImport;
    type Args = UiLanServerstatusArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_SERVERSTATUS;
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
