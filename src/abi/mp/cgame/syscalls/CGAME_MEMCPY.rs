use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CGAME_MEMCPY`.
///
/// Raven transport: `Com_Memcpy(VMA(1), VMA(2), args[3])`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:653`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:652`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:283`
#[derive(Debug)]
pub struct CgameMemcpyArgs {
    /// Destination buffer read through `VMA(1)`.
    dest: *mut u8,
    /// Source buffer read through `VMA(2)`.
    src: *const u8,
    /// Number of bytes copied from `args[3]`.
    count: c_int,
}

impl CgameMemcpyArgs {
    pub const fn new(dest: *mut u8, src: *const u8, count: c_int) -> Self {
        Self { dest, src, count }
    }

    pub const fn dest(&self) -> *mut u8 {
        self.dest
    }

    pub const fn src(&self) -> *const u8 {
        self.src
    }

    pub const fn count(&self) -> c_int {
        self.count
    }
}

/// `CGAME_MEMCPY` MP cgame imports syscall ABI token.
///
/// Raven: "DO NOT EVER add a GAME/CGAME/UI generic call without adding a trap
/// to match"; generic traps are shared and ordered from 100.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:131`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:654`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:652`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:283`
pub struct CgameMemcpy;

impl OutboundSysCall for CgameMemcpy {
    type Import = MpCgameImport;
    type Args = CgameMemcpyArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_MEMCPY;
}

impl EncodeSysCall for CgameMemcpy {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.dest() as *const u8),
            ptr_to_word(args.src()),
            args.count() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgameMemcpy {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
