use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qtime_t;

/// Arguments for `CG_REAL_TIME`.
///
/// Raven wrapper: `return syscall( CG_REAL_TIME, qtime );`
/// Raven transport: `return Com_RealTime( (struct qtime_s *)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:575-576`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:575-576`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1019-1020`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRealTimeArgs {
    qtime: *mut qtime_t,
}

impl CgRealTimeArgs {
    pub const fn new(qtime: *mut qtime_t) -> Self {
        Self { qtime }
    }
}

/// `CG_REAL_TIME` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:208`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:575-576`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:575-576`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1019-1020`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1019-1020`
pub struct CgRealTime;

impl OutboundSysCall for CgRealTime {
    type Import = MpCgameImport;
    type Args = CgRealTimeArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_REAL_TIME;
}

impl EncodeSysCall for CgRealTime {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.qtime)])
    }
}

impl DecodeSysCallReturn for CgRealTime {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
