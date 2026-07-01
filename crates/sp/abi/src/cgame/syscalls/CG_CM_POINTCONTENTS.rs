use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::clipHandle_t;
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_CM_POINTCONTENTS`.
///
/// Raven wrapper: `syscall( CG_CM_POINTCONTENTS, p, model )`
/// Raven transport: `CM_PointContents((float *)VMA(1), args[2])`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:147-149`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:535-536`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmPointcontentsArgs {
    point: *const vec3_t,
    model: clipHandle_t,
}

impl CgCmPointcontentsArgs {
    pub const fn new(point: *const vec3_t, model: clipHandle_t) -> Self {
        Self { point, model }
    }

    pub const fn point(&self) -> *const vec3_t {
        self.point
    }

    pub const fn model(&self) -> clipHandle_t {
        self.model
    }
}

/// `CG_CM_POINTCONTENTS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:85`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:147-149`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:535-536`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:535-536`
pub struct CgCmPointcontents;

impl OutboundSysCall for CgCmPointcontents {
    type Import = SpCgameImport;
    type Args = CgCmPointcontentsArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_POINTCONTENTS;
}

impl EncodeSysCall for CgCmPointcontents {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.point()), args.model() as isize])
    }
}

impl DecodeSysCallReturn for CgCmPointcontents {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
