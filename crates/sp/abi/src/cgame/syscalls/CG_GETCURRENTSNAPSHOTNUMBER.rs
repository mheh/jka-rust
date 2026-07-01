use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETCURRENTSNAPSHOTNUMBER`.
///
/// Raven wrapper: `void trap_GetCurrentSnapshotNumber(int *snapshotNumber, int *serverTime)`.
/// The SP client switch decodes both pointers as `VMA(1)` and `VMA(2)` and writes
/// snapshot data through `CL_GetCurrentSnapshotNumber`, then returns `0`.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:450-451`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:755-757`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:755-757`
#[derive(Debug)]
pub struct CgGetcurrentsnapshotnumberArgs {
    snapshot_number: *mut c_int,
    server_time: *mut c_int,
}

impl CgGetcurrentsnapshotnumberArgs {
    /// Construct raw `trap_GetCurrentSnapshotNumber` syscall args.
    ///
    /// # Safety
    /// Both pointers must be valid writable `int` slots for the duration of the
    /// syscall.
    pub const unsafe fn new(snapshot_number: *mut c_int, server_time: *mut c_int) -> Self {
        Self {
            snapshot_number,
            server_time,
        }
    }

    pub const fn snapshot_number(&self) -> *mut c_int {
        self.snapshot_number
    }

    pub const fn server_time(&self) -> *mut c_int {
        self.server_time
    }
}

/// `CG_GETCURRENTSNAPSHOTNUMBER` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:152`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:450-451`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:755-757`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:755-757`
pub struct CgGetcurrentsnapshotnumber;

impl OutboundSysCall for CgGetcurrentsnapshotnumber {
    type Import = SpCgameImport;
    type Args = CgGetcurrentsnapshotnumberArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER;
}

impl EncodeSysCall for CgGetcurrentsnapshotnumber {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.snapshot_number()),
            ptr_to_word(args.server_time()),
        ])
    }
}

impl DecodeSysCallReturn for CgGetcurrentsnapshotnumber {
    // Raven returns 0; results are written through out pointers.
    fn decode_return(_word: isize) -> Self::Output {}
}
