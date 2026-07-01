use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETCURRENTSNAPSHOTNUMBER`.
///
/// Raven wrapper: `void trap_GetCurrentSnapshotNumber(int *snapshotNumber, int *serverTime)`.
/// The MP client switch decodes both words with `VMA` and passes them to
/// `CL_GetCurrentSnapshotNumber`, which writes the current snapshot number and
/// server time through the caller-provided out pointers. The switch returns `0`,
/// so those pointer writes are the result channel.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:469-470`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:147-150`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:960-962`
#[derive(Debug)]
pub struct CgGetcurrentsnapshotnumberArgs {
    /// Out pointer for `cl.snap.messageNum`, decoded by Raven as `VMA(1)`.
    snapshot_number: *mut c_int,
    /// Out pointer for `cl.snap.serverTime`, decoded by Raven as `VMA(2)`.
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

/// `CG_GETCURRENTSNAPSHOTNUMBER` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_GETCURRENTSNAPSHOTNUMBER, snapshotNumber, serverTime );`
/// Raven transport: `CL_GetCurrentSnapshotNumber((int *)VMA(1), (int *)VMA(2)); return 0;`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:181`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:469-470`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:147-150`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:960-962`
pub struct CgGetcurrentsnapshotnumber;

impl OutboundSysCall for CgGetcurrentsnapshotnumber {
    type Import = MpCgameImport;
    type Args = CgGetcurrentsnapshotnumberArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER;
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
    // Raven returns 0; `snapshotNumber` and `serverTime` are written through out pointers.
    fn decode_return(_word: isize) -> Self::Output {}
}
