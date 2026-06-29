use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_R_GET_BMODEL_VERTS`.
///
/// Raven wrapper: `syscall( CG_R_GET_BMODEL_VERTS, bmodelIndex, verts, normal );`
/// Raven transport writes the chosen brush-model face into `VMA(2)` and reads
/// the view normal from `VMA(3)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:421-423`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2299`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1054-1056`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRGetBmodelVertsArgs {
    bmodel_index: c_int,
    verts: *mut vec3_t,
    normal: *const vec3_t,
}

impl CgRGetBmodelVertsArgs {
    pub const fn new(bmodel_index: c_int, verts: *mut vec3_t, normal: *const vec3_t) -> Self {
        Self {
            bmodel_index,
            verts,
            normal,
        }
    }
}

/// `CG_R_GET_BMODEL_VERTS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:170`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:421-423`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1054-1056`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1054-1056`
pub struct CgRGetBmodelVerts;

impl OutboundSysCall for CgRGetBmodelVerts {
    type Import = MpCgameImport;
    type Args = CgRGetBmodelVertsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_GET_BMODEL_VERTS;
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
