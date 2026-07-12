use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    c_int_to_word, ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall,
    SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_CM_TRANSFORMEDPOINTCONTENTS`.
///
/// C ABI: `int trap_CM_TransformedPointContents(const vec3_t p, clipHandle_t model,
/// const vec3_t origin, const vec3_t angles)`.
/// Raven's wrapper forwards the raw vector pointers and model handle; the client
/// switch reads `p`, `origin`, and `angles` with `VMA` and `model` from `args[2]`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:145-147`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:791-792`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTransformedpointcontentsArgs {
    point: *const vec3_t,
    model: c_int,
    origin: *const vec3_t,
    angles: *const vec3_t,
}

impl CgCmTransformedpointcontentsArgs {
    pub const fn new(
        point: *const vec3_t,
        model: c_int,
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

    pub const fn model(&self) -> c_int {
        self.model
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn angles(&self) -> *const vec3_t {
        self.angles
    }
}

/// `CG_CM_TRANSFORMEDPOINTCONTENTS` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall( CG_CM_TRANSFORMEDPOINTCONTENTS, p, model, origin, angles );`
/// Raven transport: `return CM_TransformedPointContents((const float *)VMA(1), args[2], (const float *)VMA(3), (const float *)VMA(4));`
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:89`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:145-147`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:145-147`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:791-792`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:791-792`
pub struct CgCmTransformedpointcontents;

impl OutboundSysCall for CgCmTransformedpointcontents {
    type Import = MpCgameImport;
    type Args = CgCmTransformedpointcontentsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_TRANSFORMEDPOINTCONTENTS;
}

impl EncodeSysCall for CgCmTransformedpointcontents {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.point()),
            c_int_to_word(args.model()),
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
