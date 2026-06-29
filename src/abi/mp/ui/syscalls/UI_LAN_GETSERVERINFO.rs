use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_GETSERVERINFO`.
///
/// Raven wrapper: `syscall( UI_LAN_GETSERVERINFO, source, n, buf, buflen );`
/// Raven transport: `LAN_GetServerInfo( args[1], args[2], (char *)VMA(3), args[4] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:278-279`
#[derive(Debug)]
pub struct UiLanGetserverinfoArgs {
    source: c_int,
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
}

impl UiLanGetserverinfoArgs {
    pub fn new(source: c_int, n: c_int, buf: *mut c_char, buflen: c_int) -> Self {
        Self {
            source,
            n,
            buf,
            buflen,
        }
    }

    pub fn source(&self) -> c_int {
        self.source
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

/// `UI_LAN_GETSERVERINFO` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_GETSERVERINFO, source, n, buf, buflen );`
/// Raven transport: `LAN_GetServerInfo( args[1], args[2], (char *)VMA(3), args[4] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:97`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:278-279`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:966`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1092-1094`
pub struct UiLanGetserverinfo;

impl OutboundSysCall for UiLanGetserverinfo {
    type Import = MpUiImport;
    type Args = UiLanGetserverinfoArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_GETSERVERINFO;
}

impl EncodeSysCall for UiLanGetserverinfo {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.source() as isize,
            args.n() as isize,
            ptr_to_word(args.buf()),
            args.buflen() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiLanGetserverinfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
