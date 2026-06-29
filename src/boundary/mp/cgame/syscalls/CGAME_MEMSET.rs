use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CGAME_MEMSET`.
///
/// Raven's cgame switch reads `dest` through `VMA(1)` and passes the remaining
/// words directly to `Com_Memset(VMA(1), args[2], args[3])`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:650`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:624`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:649`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:282`
#[derive(Debug)]
pub struct CgameMemsetArgs {
    /// Destination buffer pointer, decoded by Raven as `VMA(1)`.
    dest: *mut c_void,
    /// Fill byte value, read by Raven as `args[2]`.
    val: c_int,
    /// Number of bytes to fill, read by Raven as `args[3]`.
    count: c_int,
}

impl CgameMemsetArgs {
    pub const fn new(dest: *mut c_void, val: c_int, count: c_int) -> Self {
        Self { dest, val, count }
    }

    pub const fn dest(&self) -> *mut c_void {
        self.dest
    }

    pub const fn val(&self) -> c_int {
        self.val
    }

    pub const fn count(&self) -> c_int {
        self.count
    }
}

/// `CGAME_MEMSET` MP cgame imports syscall boundary token.
///
/// Raven: `Com_Memset(VMA(1), args[2], args[3])`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:130`
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:650`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:651`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:649`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:282`
pub struct CgameMemset;

impl OutboundSysCall for CgameMemset {
    type Import = MpCgameImport;
    type Args = CgameMemsetArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_MEMSET;
}

impl EncodeSysCall for CgameMemset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.dest()),
            args.val() as isize,
            args.count() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgameMemset {
    // Raven calls `Com_Memset`, then returns 0; the helper has no semantic output.
    fn decode_return(_word: isize) -> Self::Output {}
}
