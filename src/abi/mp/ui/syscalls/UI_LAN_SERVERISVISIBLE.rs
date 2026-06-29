use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_SERVERISVISIBLE`.
///
/// Raven wrapper: `return syscall( UI_LAN_SERVERISVISIBLE, source, n );`
/// Raven transport: `return LAN_ServerIsVisible( args[1], args[2] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:322-323`
#[derive(Debug)]
pub struct UiLanServerisvisibleArgs {
    source: c_int,
    n: c_int,
}

impl UiLanServerisvisibleArgs {
    pub fn new(source: c_int, n: c_int) -> Self {
        Self { source, n }
    }

    pub fn source(&self) -> c_int {
        self.source
    }

    pub fn n(&self) -> c_int {
        self.n
    }
}

/// `UI_LAN_SERVERISVISIBLE` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_SERVERISVISIBLE, source, n );`
/// Raven transport: `return LAN_ServerIsVisible( args[1], args[2] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:113`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:322-323`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:975`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1103-1104`
pub struct UiLanServerisvisible;

impl OutboundSysCall for UiLanServerisvisible {
    type Import = MpUiImport;
    type Args = UiLanServerisvisibleArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_SERVERISVISIBLE;
}

impl EncodeSysCall for UiLanServerisvisible {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize, args.n() as isize])
    }
}

impl DecodeSysCallReturn for UiLanServerisvisible {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
