use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_GETSERVERADDRESSSTRING`.
///
/// Raven wrapper: `syscall( UI_LAN_GETSERVERADDRESSSTRING, source, n, buf, buflen );`
/// Raven transport: `LAN_GetServerAddressString( args[1], args[2], (char *)VMA(3), args[4] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:274-275`
#[derive(Debug)]
pub struct UiLanGetserveraddressstringArgs {
    source: c_int,
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
}

impl UiLanGetserveraddressstringArgs {
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

/// `UI_LAN_GETSERVERADDRESSSTRING` MP UI imports syscall boundary token.
///
/// Raven wrapper: `syscall( UI_LAN_GETSERVERADDRESSSTRING, source, n, buf, buflen );`
/// Raven transport: `LAN_GetServerAddressString( args[1], args[2], (char *)VMA(3), args[4] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:96`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:274-275`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:965`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1088-1090`
pub struct UiLanGetserveraddressstring;

impl OutboundSysCall for UiLanGetserveraddressstring {
    type Import = MpUiImport;
    type Args = UiLanGetserveraddressstringArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_GETSERVERADDRESSSTRING;
}

impl EncodeSysCall for UiLanGetserveraddressstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.source() as isize,
            args.n() as isize,
            ptr_to_word(args.buf()),
            args.buflen() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiLanGetserveraddressstring {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
