use core::ffi::c_void;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_PRECISIONTIMER_START` outbound game-to-engine syscall.
///
/// Allocates a high-precision timer, writing its opaque handle through
/// `the_new_timer` (a `void **` out-param). Pair with `G_PRECISIONTIMER_END`.
#[derive(Debug)]
pub struct GPrecisiontimerStartArgs {
    /// Out-param: engine writes the allocated timer handle here (`void **`).
    the_new_timer: *mut *mut c_void,
}

impl GPrecisiontimerStartArgs {
    pub fn new(the_new_timer: *mut *mut c_void) -> Self {
        Self { the_new_timer }
    }

    pub fn the_new_timer(&self) -> *mut *mut c_void {
        self.the_new_timer
    }
}

/// `G_PRECISIONTIMER_START` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:117`
pub struct GPrecisiontimerStart;

impl OutboundSysCall for GPrecisiontimerStart {
    type Import = GameImport;
    type Args = GPrecisiontimerStartArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_PRECISIONTIMER_START;
}

impl EncodeSysCall for GPrecisiontimerStart {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.the_new_timer as *mut c_void)])
    }
}

impl DecodeSysCallReturn for GPrecisiontimerStart {
    fn decode_return(_word: isize) -> Self::Output {}
}
