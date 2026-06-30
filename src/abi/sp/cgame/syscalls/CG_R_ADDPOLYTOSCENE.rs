use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qhandle_t;

/// Arguments for `CG_R_ADDPOLYTOSCENE`.
///
/// Raven wrapper: `cgi_R_AddPolyToScene( qhandle_t hShader , int numVerts, const polyVert_t *verts )`
/// Raven transport: `re.AddPolyToScene( args[1], args[2], (const polyVert_t *) VMA(3) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:380-381`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:698-700`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:698-700`
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

/// `CG_R_ADDPOLYTOSCENE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:137`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:380-381`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:698-700`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:698-700`
pub struct CgRAddpolytoscene;

impl OutboundSysCall for CgRAddpolytoscene {
    type Import = SpCgameImport;
    type Args = CgRAddpolytosceneArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_ADDPOLYTOSCENE;
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
