use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;

/// Arguments for `CG_R_ADDPOLYTOSCENE`.
///
/// Raven wrapper: `syscall( CG_R_ADDPOLYTOSCENE, hShader, numVerts, verts );`
/// Raven transport:
/// `re.AddPolyToScene( args[1], args[2], (const polyVert_t *)VMA(3), 1 );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:335-336`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2271`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:897-899`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRAddpolytosceneArgs {
    h_shader: qhandle_t,
    num_verts: c_int,
    verts: *const c_void,
}

impl CgRAddpolytosceneArgs {
    pub const fn new(h_shader: qhandle_t, num_verts: c_int, verts: *const c_void) -> Self {
        Self {
            h_shader,
            num_verts,
            verts,
        }
    }
}

/// `CG_R_ADDPOLYTOSCENE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:152`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:335-336`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:897-899`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:897-899`
pub struct CgRAddpolytoscene;

impl OutboundSysCall for CgRAddpolytoscene {
    type Import = MpCgameImport;
    type Args = CgRAddpolytosceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDPOLYTOSCENE;
}

impl EncodeSysCall for CgRAddpolytoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.h_shader as isize,
            args.num_verts as isize,
            ptr_to_word(args.verts),
        ])
    }
}

impl DecodeSysCallReturn for CgRAddpolytoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
