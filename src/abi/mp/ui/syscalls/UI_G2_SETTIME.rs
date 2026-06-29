use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_G2_SETTIME`.
///
/// Raven wrapper: `void trap_G2API_SetTime(int time, int clock)`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:629-631`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1381-1383`
#[derive(Debug)]
pub struct UiG2SettimeArgs {
    /// Time read directly from `args[1]`.
    time: c_int,
    /// Clock selector read directly from `args[2]`.
    clock: c_int,
}

impl UiG2SettimeArgs {
    pub fn new(time: c_int, clock: c_int) -> Self {
        Self { time, clock }
    }

    pub fn time(&self) -> c_int {
        self.time
    }
    pub fn clock(&self) -> c_int {
        self.clock
    }
}

/// `UI_G2_SETTIME` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:170`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:170`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:629-631`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1381-1383`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1381-1383`
pub struct UiG2Settime;

impl OutboundSysCall for UiG2Settime {
    type Import = MpUiImport;
    type Args = UiG2SettimeArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETTIME;
}

impl EncodeSysCall for UiG2Settime {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.time as isize, a.clock as isize])
    }
}

impl DecodeSysCallReturn for UiG2Settime {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
