use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_GETPING`.
///
/// Raven wrapper: `syscall( UI_LAN_GETPING, n, buf, buflen, pingtime );`
/// Raven transport: `LAN_GetPing( args[1], (char *)VMA(2), args[3], (int *)VMA(4) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:310-311`
#[derive(Debug)]
pub struct UiLanGetpingArgs {
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
    pingtime: *mut c_int,
}

impl UiLanGetpingArgs {
    pub fn new(n: c_int, buf: *mut c_char, buflen: c_int, pingtime: *mut c_int) -> Self {
        Self {
            n,
            buf,
            buflen,
            pingtime,
        }
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

    pub fn pingtime(&self) -> *mut c_int {
        self.pingtime
    }
}

/// `UI_LAN_GETPING` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_GETPING, n, buf, buflen, pingtime );`
/// Raven transport: `LAN_GetPing( args[1], (char *)VMA(2), args[3], (int *)VMA(4) );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:67`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:310-311`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:970`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1077-1079`
pub struct UiLanGetping;

impl OutboundSysCall for UiLanGetping {
    type Import = MpUiImport;
    type Args = UiLanGetpingArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_GETPING;
}

impl EncodeSysCall for UiLanGetping {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.n() as isize,
            ptr_to_word(args.buf()),
            args.buflen() as isize,
            ptr_to_word(args.pingtime()),
        ])
    }
}

impl DecodeSysCallReturn for UiLanGetping {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
