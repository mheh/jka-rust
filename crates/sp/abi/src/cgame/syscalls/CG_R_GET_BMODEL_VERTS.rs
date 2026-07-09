use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_R_GET_BMODEL_VERTS`.
///
/// Raven wrapper: `syscall( CG_R_GET_BMODEL_VERTS, bmodelIndex, verts, normal );`
/// Raven transport writes selected model vertices to `VMA(2)` and normal to `VMA(3)`.
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:509-511`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:809-811`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRGetBmodelVertsArgs {
    bmodel_index: c_int,
    verts: *mut vec3_t,
    normal: *mut vec3_t,
}

impl CgRGetBmodelVertsArgs {
    pub const fn new(bmodel_index: c_int, verts: *mut vec3_t, normal: *mut vec3_t) -> Self {
        Self {
            bmodel_index,
            verts,
            normal,
        }
    }
}

/// `CG_R_GET_BMODEL_VERTS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:182`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:509-511`
/// Output source: `oracle/code/client/cl_cgame.cpp:809-811`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:809-811`
pub struct CgRGetBmodelVerts;

impl OutboundSysCall for CgRGetBmodelVerts {
    type Import = SpCgameImport;
    type Args = CgRGetBmodelVertsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_GET_BMODEL_VERTS;
}

impl EncodeSysCall for CgRGetBmodelVerts {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.bmodel_index as isize,
            ptr_to_word(args.verts),
            ptr_to_word(args.normal),
        ])
    }
}

impl DecodeSysCallReturn for CgRGetBmodelVerts {
    fn decode_return(_word: isize) -> Self::Output {}
}
