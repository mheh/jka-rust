use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_ADDSERVER`.
///
/// Raven wrapper: `return syscall( UI_LAN_ADDSERVER, source, name, addr );`
/// Raven transport: `return LAN_AddServer( args[1], (const char *)VMA(2), (const char *)VMA(3) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:330-331`
#[derive(Debug)]
pub struct UiLanAddserverArgs {
    source: c_int,
    name: *const c_char,
    addr: *const c_char,
}

impl UiLanAddserverArgs {
    pub fn new(source: c_int, name: *const c_char, addr: *const c_char) -> Self {
        Self { source, name, addr }
    }

    pub fn source(&self) -> c_int {
        self.source
    }

    pub fn name(&self) -> *const c_char {
        self.name
    }

    pub fn addr(&self) -> *const c_char {
        self.addr
    }
}

/// `UI_LAN_ADDSERVER` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_ADDSERVER, source, name, addr );`
/// Raven transport: `return LAN_AddServer( args[1], (const char *)VMA(2), (const char *)VMA(3) );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:103`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:330-331`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:977`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1063-1064`
pub struct UiLanAddserver;

impl OutboundSysCall for UiLanAddserver {
    type Import = MpUiImport;
    type Args = UiLanAddserverArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_ADDSERVER;
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
