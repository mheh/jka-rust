use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PRECISIONTIMER_START`.
///
/// Raven: precision timer funcs... ALWAYS call end after start with supplied
/// ptr, or you'll get a nasty memory leak. Not that you should be using these
/// outside of debug anyway.. because you shouldn't be. So don't.
/// Raven: Start should be supplied with a pointer to an empty pointer; the
/// empty pointer will be filled with an exe address to our timer. You must pass
/// this pointer back unmodified to the timer end func.
/// Raven wrapper: `syscall(CG_PRECISIONTIMER_START, theNewTimer);`
/// Raven transport: writes a new `timing_c` pointer into `(void **)VMA(1)`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:36-41`
/// Args source: `oracle/codemp/cgame/cg_local.h:2155-2157`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:696-705`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPrecisiontimerStartArgs {
    the_new_timer: *mut *mut c_void,
}

impl CgPrecisiontimerStartArgs {
    pub const fn new(the_new_timer: *mut *mut c_void) -> Self {
        Self { the_new_timer }
    }
}

/// `CG_PRECISIONTIMER_START` MP cgame imports syscall ABI token.
///
/// Raven: Also for profiling.. do not use for game related tasks.
/// Enum value source: `oracle/codemp/cgame/cg_public.h:62`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:36-41`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:696-705`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:696-705`
pub struct CgPrecisiontimerStart;

impl OutboundSysCall for CgPrecisiontimerStart {
    type Import = MpCgameImport;
    type Args = CgPrecisiontimerStartArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_PRECISIONTIMER_START;
}

impl EncodeSysCall for CgPrecisiontimerStart {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.the_new_timer)])
    }
}

impl DecodeSysCallReturn for CgPrecisiontimerStart {
    fn decode_return(_word: isize) -> Self::Output {}
}
