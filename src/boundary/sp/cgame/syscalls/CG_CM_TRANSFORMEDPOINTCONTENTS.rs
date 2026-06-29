use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::{clipHandle_t, vec3_t};

/// Arguments for `CG_CM_TRANSFORMEDPOINTCONTENTS`.
///
/// Raven wrapper: `syscall( CG_CM_TRANSFORMEDPOINTCONTENTS, p, model, origin, angles )`
/// Raven transport: `CM_TransformedPointContents((const float *)VMA(1), args[2], (const float *)VMA(3), (const float *)VMA(4))`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:151-153`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:537-538`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTransformedpointcontentsArgs {
    point: *const vec3_t,
    model: clipHandle_t,
    origin: *const vec3_t,
    angles: *const vec3_t,
}

impl CgCmTransformedpointcontentsArgs {
    pub const fn new(
        point: *const vec3_t,
        model: clipHandle_t,
        origin: *const vec3_t,
        angles: *const vec3_t,
    ) -> Self {
        Self {
            point,
            model,
            origin,
            angles,
        }
    }

    pub const fn point(&self) -> *const vec3_t {
        self.point
    }

    pub const fn model(&self) -> clipHandle_t {
        self.model
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn angles(&self) -> *const vec3_t {
        self.angles
    }
}

/// `CG_CM_TRANSFORMEDPOINTCONTENTS` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:86`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:151-153`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:537-538`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:537-538`
pub struct CgCmTransformedpointcontents;

impl OutboundSysCall for CgCmTransformedpointcontents {
    type Import = SpCgameImport;
    type Args = CgCmTransformedpointcontentsArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_TRANSFORMEDPOINTCONTENTS;
}

impl EncodeSysCall for CgCmTransformedpointcontents {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.point()),
            args.model() as isize,
            ptr_to_word(args.origin()),
            ptr_to_word(args.angles()),
        ])
    }
}

impl DecodeSysCallReturn for CgCmTransformedpointcontents {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
