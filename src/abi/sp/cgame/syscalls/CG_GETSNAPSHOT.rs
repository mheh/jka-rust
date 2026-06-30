use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_GETSNAPSHOT`.
///
/// Raven wrapper: `qboolean trap_GetSnapshot(int snapshotNumber, snapshot_t *snapshot)`.
/// SP client switch calls `CL_GetSnapshot(args[1], (snapshot_t *)VMA(2))`.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:454-455`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:758-759`
#[derive(Debug)]
pub struct CgGetsnapshotArgs {
    snapshot_number: c_int,
    snapshot: *mut c_void,
}

impl CgGetsnapshotArgs {
    /// Construct raw `trap_GetSnapshot` syscall args.
    ///
    /// # Safety
    /// `snapshot` must point to a writable `snapshot_t` for the duration of the
    /// syscall.
    pub const unsafe fn new(snapshot_number: c_int, snapshot: *mut c_void) -> Self {
        Self {
            snapshot_number,
            snapshot,
        }
    }

    pub const fn snapshot_number(&self) -> c_int {
        self.snapshot_number
    }

    pub const fn snapshot(&self) -> *mut c_void {
        self.snapshot
    }
}

/// `CG_GETSNAPSHOT` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:153`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:454-455`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:758-759`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:758-759`
pub struct CgGetsnapshot;

impl OutboundSysCall for CgGetsnapshot {
    type Import = SpCgameImport;
    type Args = CgGetsnapshotArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETSNAPSHOT;
}

impl EncodeSysCall for CgGetsnapshot {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.snapshot_number() as isize,
            ptr_to_word(args.snapshot()),
        ])
    }
}

impl DecodeSysCallReturn for CgGetsnapshot {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
