use core::ffi::c_void;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qhandle_t;

/// Arguments for `UI_R_ADDPOLYTOSCENE`.
///
/// C ABI: `void trap_R_AddPolyToScene(qhandle_t hShader, int numVerts, const polyVert_t *verts)`.
/// Raven's client switch forwards the shader handle and vertex count as raw
/// words and the vertex array through `VMA(3)`.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:178-179`
/// Output source: `oracle/codemp/ui/ui_syscalls.c:178-179`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:964-965`
#[derive(Debug, Clone, Copy)]
pub struct UiRAddpolytosceneArgs {
    pub shader: qhandle_t,
    pub num_verts: i32,
    pub verts: *const c_void,
}

impl UiRAddpolytosceneArgs {
    pub const fn new(shader: qhandle_t, num_verts: i32, verts: *const c_void) -> Self {
        Self {
            shader,
            num_verts,
            verts,
        }
    }
}

/// `UI_R_ADDPOLYTOSCENE` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:42`
pub struct UiRAddpolytoscene;

impl OutboundSysCall for UiRAddpolytoscene {
    type Import = MpUiImport;
    type Args = UiRAddpolytosceneArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_ADDPOLYTOSCENE;
}

impl EncodeSysCall for UiRAddpolytoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.shader as isize,
            args.num_verts as isize,
            ptr_to_word(args.verts),
        ])
    }
}

impl DecodeSysCallReturn for UiRAddpolytoscene {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
