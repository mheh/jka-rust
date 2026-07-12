use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_CM_POINTCONTENTS`.
///
/// C ABI: `int trap_CM_PointContents(const vec3_t p, clipHandle_t model)`.
/// Raven's wrapper forwards the raw `vec3_t` pointer plus an int-compatible
/// `clipHandle_t`, and the client switch reads them as `VMA(1)` and `args[2]`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:143-144`
/// Args source: `oracle/codemp/cgame/cg_local.h:2201`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:789-790`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmPointcontentsArgs {
    point: *const vec3_t,
    model: c_int,
}

impl CgCmPointcontentsArgs {
    pub const fn new(point: *const vec3_t, model: c_int) -> Self {
        Self { point, model }
    }

    pub const fn point(&self) -> *const vec3_t {
        self.point
    }

    pub const fn model(&self) -> c_int {
        self.model
    }
}

/// `CG_CM_POINTCONTENTS` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall( CG_CM_POINTCONTENTS, p, model );`
/// Raven transport: `return CM_PointContents((const float *)VMA(1), args[2]);`
/// Raven collision API: `CM_PointContents` returns an ORed contents mask.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:88`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:143-144`
/// Args source: `oracle/codemp/cgame/cg_local.h:2201`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:143-144`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:789-790`
/// Output source: `oracle/codemp/qcommon/cm_public.h:21`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:789-790`
pub struct CgCmPointcontents;

impl OutboundSysCall for CgCmPointcontents {
    type Import = MpCgameImport;
    type Args = CgCmPointcontentsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_POINTCONTENTS;
}

impl EncodeSysCall for CgCmPointcontents {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.point()), args.model() as isize])
    }
}

impl DecodeSysCallReturn for CgCmPointcontents {
    // `CM_PointContents` returns an int contents mask in the syscall word.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
