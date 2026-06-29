use core::ffi::{c_int, c_void};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

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

/// `G_PRECISIONTIMER_END` MP game imports syscall ABI token.
///
/// Raven: console variable interaction
/// Source: `oracle/oracle/codemp/game/g_public.h:118`
pub struct GPrecisiontimerEnd;

impl OutboundSysCall for GPrecisiontimerEnd {
    type Import = GameImport;
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
