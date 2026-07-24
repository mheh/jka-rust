use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::qcommon::qtime_t;

/// `UI_REAL_TIME` outbound game-to-engine syscall.
///
/// Reads the engine's wall-clock time into `qtime`, returning the raw
/// seconds-since-epoch value.  Mirrors `syscall!(UiREAL_TIME, qtime as *mut qtime_t)`.
#[derive(Debug)]
pub struct UiRealTimeArgs {
    qtime: *mut qtime_t,
}

impl UiRealTimeArgs {
    pub fn new(qtime: *mut qtime_t) -> Self {
        Self { qtime }
    }

    pub fn qtime(&self) -> *mut qtime_t {
        self.qtime
    }
}

/// `UI_REAL_TIME` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:232`
pub struct UiRealTime;

impl OutboundSysCall for UiRealTime {
    type Import = MpUiImport;
    type Args = UiRealTimeArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_REAL_TIME;
}

impl EncodeSysCall for UiRealTime {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.qtime)])
    }
}

impl DecodeSysCallReturn for UiRealTime {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
