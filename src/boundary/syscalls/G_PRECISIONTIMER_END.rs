use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_PRECISIONTIMER_END` outbound game-to-engine syscall.
///
/// Stops the high-precision timer `the_timer` (allocated by
/// `G_PRECISIONTIMER_START`) and returns its elapsed measurement in
/// microseconds.
#[derive(Debug)]
pub struct GPrecisiontimerEndArgs {
    /// The timer handle previously written by `G_PRECISIONTIMER_START`.
    the_timer: *mut c_void,
}

impl GPrecisiontimerEndArgs {
    pub fn new(the_timer: *mut c_void) -> Self {
        Self { the_timer }
    }

    pub fn the_timer(&self) -> *mut c_void {
        self.the_timer
    }
}

pub struct GPrecisiontimerEnd;

impl OutboundSysCall for GPrecisiontimerEnd {
    type Args = GPrecisiontimerEndArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_PRECISIONTIMER_END;
}

impl EncodeSysCall for GPrecisiontimerEnd {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.the_timer())])
    }
}

impl DecodeSysCallReturn for GPrecisiontimerEnd {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
