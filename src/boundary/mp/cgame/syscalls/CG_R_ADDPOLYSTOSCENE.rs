use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;

/// Arguments for `CG_R_ADDPOLYSTOSCENE`.
///
/// Raven wrapper:
/// `syscall( CG_R_ADDPOLYSTOSCENE, hShader, numVerts, verts, numPolys );`
/// Raven transport forwards `verts` as an opaque `polyVert_t` block through
/// `VMA(3)` and repeats the poly batch `args[4]` times.
/// The Raven comment says these polys are intended for simple wall marks, not
/// significant construction.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:339-340`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2269-2272`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:900-902`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRAddpolystosceneArgs {
    h_shader: qhandle_t,
    num_verts: c_int,
    verts: *const c_void,
    num_polys: c_int,
}

impl CgRAddpolystosceneArgs {
    pub const fn new(
        h_shader: qhandle_t,
        num_verts: c_int,
        verts: *const c_void,
        num_polys: c_int,
    ) -> Self {
        Self {
            h_shader,
            num_verts,
            verts,
            num_polys,
        }
    }
}

/// `CG_R_ADDPOLYSTOSCENE` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:153`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:339-340`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:900-902`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:900-902`
pub struct CgRAddpolystoscene;

impl OutboundSysCall for CgRAddpolystoscene {
    type Import = MpCgameImport;
    type Args = CgRAddpolystosceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDPOLYSTOSCENE;
}

impl EncodeSysCall for CgRAddpolystoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.h_shader as isize,
            args.num_verts as isize,
            ptr_to_word(args.verts),
            args.num_polys as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRAddpolystoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
