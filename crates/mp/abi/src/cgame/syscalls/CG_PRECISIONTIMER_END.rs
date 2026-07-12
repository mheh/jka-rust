use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PRECISIONTIMER_END`.
///
/// Raven: If you're using the start example, the appropriate call for this is
/// `int result = trap_PrecisionTimer_End(blah);`
/// Raven wrapper: `return syscall(CG_PRECISIONTIMER_END, theTimer);`
/// Raven transport: casts `args[1]` back to `timing_c *`, returns
/// `timer->End()`, then deletes the timer.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:44-47`
/// Args source: `oracle/codemp/cgame/cg_local.h:2158`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:706-713`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPrecisiontimerEndArgs {
    the_timer: *mut c_void,
}

impl CgPrecisiontimerEndArgs {
    pub const fn new(the_timer: *mut c_void) -> Self {
        Self { the_timer }
    }
}

/// `CG_PRECISIONTIMER_END` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:63`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:44-47`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:44-47`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:706-713`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:706-713`
pub struct CgPrecisiontimerEnd;

impl OutboundSysCall for CgPrecisiontimerEnd {
    type Import = MpCgameImport;
    type Args = CgPrecisiontimerEndArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_PRECISIONTIMER_END;
}

impl EncodeSysCall for CgPrecisiontimerEnd {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.the_timer)])
    }
}

impl DecodeSysCallReturn for CgPrecisiontimerEnd {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
