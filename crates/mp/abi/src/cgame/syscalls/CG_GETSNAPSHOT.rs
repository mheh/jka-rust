use super::super::MpCgameImport;
use core::ffi::{c_int, c_void};

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_GETSNAPSHOT`.
///
/// Raven wrapper: `qboolean trap_GetSnapshot(int snapshotNumber, snapshot_t *snapshot)`.
/// Raven transport: `return CL_GetSnapshot(args[1], (snapshot_t *)VMA(2));`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:473-474`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:963-964`
#[derive(Debug)]
pub struct CgGetsnapshotArgs {
    /// Snapshot sequence number, read by Raven as `args[1]`.
    snapshot_number: c_int,
    /// Caller-owned `snapshot_t` output buffer, decoded by Raven as `VMA(2)`.
    snapshot: *mut c_void,
}

impl CgGetsnapshotArgs {
    /// Construct raw `trap_GetSnapshot` syscall args.
    ///
    /// # Safety
    /// `snapshot` must point to a writable `snapshot_t` slot for the duration of
    /// the syscall.
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

/// `CG_GETSNAPSHOT` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall(CG_GETSNAPSHOT, snapshotNumber, snapshot);`
/// Raven transport: `return CL_GetSnapshot(args[1], (snapshot_t *)VMA(2));`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:182`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:473-474`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:473-474`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:963-964`
pub struct CgGetsnapshot;

impl OutboundSysCall for CgGetsnapshot {
    type Import = MpCgameImport;
    type Args = CgGetsnapshotArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETSNAPSHOT;
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
