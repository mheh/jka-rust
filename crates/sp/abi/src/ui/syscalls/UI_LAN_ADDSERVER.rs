use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_ADDSERVER`.
///
/// Raven wrapper: `return syscall( UI_LAN_ADDSERVER, source, name, addr );`
/// Raven transport: `return LAN_AddServer( args[1], (const char *)VMA(2), (const char *)VMA(3) );`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:225`
/// Args source (SP): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_ADDSERVER`.
/// Fallback args/source: `oracle/oracle/codemp/ui/ui_syscalls.c:331`
/// Transport/switch source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:1063-1064`
/// Output source fallback: `oracle/oracle/codemp/ui/ui_local.h:977`
pub struct UiLanAddserverArgs {
    source: c_int,
    name: *const c_char,
    addr: *const c_char,
}

impl UiLanAddserverArgs {
    pub const fn new(source: c_int, name: *const c_char, addr: *const c_char) -> Self {
        Self { source, name, addr }
    }

    pub const fn source(&self) -> c_int {
        self.source
    }

    pub const fn name(&self) -> *const c_char {
        self.name
    }

    pub const fn addr(&self) -> *const c_char {
        self.addr
    }
}

/// `UI_LAN_ADDSERVER` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:225`
pub struct UiLanAddserver;

impl OutboundSysCall for UiLanAddserver {
    type Import = SpUiImport;
    type Args = UiLanAddserverArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_ADDSERVER;
}

impl EncodeSysCall for UiLanAddserver {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.source() as isize,
            ptr_to_word(args.name()),
            ptr_to_word(args.addr()),
        ])
    }
}

impl DecodeSysCallReturn for UiLanAddserver {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
