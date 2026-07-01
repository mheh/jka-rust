use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_GETPINGINFO`.
///
/// Raven wrapper: `syscall( UI_LAN_GETPINGINFO, n, buf, buflen );`
/// Raven transport: `LAN_GetPingInfo( args[1], (char *)VMA(2), args[3] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:314-315`
#[derive(Debug)]
pub struct UiLanGetpinginfoArgs {
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
}

impl UiLanGetpinginfoArgs {
    pub fn new(n: c_int, buf: *mut c_char, buflen: c_int) -> Self {
        Self { n, buf, buflen }
    }

    pub fn n(&self) -> c_int {
        self.n
    }

    pub fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub fn buflen(&self) -> c_int {
        self.buflen
    }
}

/// `UI_LAN_GETPINGINFO` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_GETPINGINFO, n, buf, buflen );`
/// Raven transport: `LAN_GetPingInfo( args[1], (char *)VMA(2), args[3] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:68`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:314-315`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:971`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1081-1083`
pub struct UiLanGetpinginfo;

impl OutboundSysCall for UiLanGetpinginfo {
    type Import = MpUiImport;
    type Args = UiLanGetpinginfoArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_GETPINGINFO;
}

impl EncodeSysCall for UiLanGetpinginfo {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.n() as isize,
            ptr_to_word(args.buf()),
            args.buflen() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiLanGetpinginfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
