use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_LAN_COMPARESERVERS`.
///
/// Raven wrapper: `return syscall( UI_LAN_COMPARESERVERS, source, sortKey, sortDir, s1, s2 );`
/// Raven transport: `return LAN_CompareServers( args[1], args[2], args[3], args[4], args[5] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:338-339`
#[derive(Debug)]
pub struct UiLanCompareserversArgs {
    source: c_int,
    sort_key: c_int,
    sort_dir: c_int,
    s1: c_int,
    s2: c_int,
}

impl UiLanCompareserversArgs {
    pub fn new(source: c_int, sort_key: c_int, sort_dir: c_int, s1: c_int, s2: c_int) -> Self {
        Self {
            source,
            sort_key,
            sort_dir,
            s1,
            s2,
        }
    }

    pub fn source(&self) -> c_int {
        self.source
    }

    pub fn sort_key(&self) -> c_int {
        self.sort_key
    }

    pub fn sort_dir(&self) -> c_int {
        self.sort_dir
    }

    pub fn s1(&self) -> c_int {
        self.s1
    }

    pub fn s2(&self) -> c_int {
        self.s2
    }
}

/// `UI_LAN_COMPARESERVERS` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_COMPARESERVERS, source, sortKey, sortDir, s1, s2 );`
/// Raven transport: `return LAN_CompareServers( args[1], args[2], args[3], args[4], args[5] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:114`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:338-339`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:981`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1116-1117`
pub struct UiLanCompareservers;

impl OutboundSysCall for UiLanCompareservers {
    type Import = MpUiImport;
    type Args = UiLanCompareserversArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_COMPARESERVERS;
}

impl EncodeSysCall for UiLanCompareservers {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.source() as isize,
            args.sort_key() as isize,
            args.sort_dir() as isize,
            args.s1() as isize,
            args.s2() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiLanCompareservers {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
