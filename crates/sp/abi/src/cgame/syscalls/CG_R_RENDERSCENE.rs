use core::ffi::c_void;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_RENDERSCENE`.
///
/// Raven wrapper: `cgi_R_RenderScene( const refdef_t *fd )`
/// Raven transport: `re.RenderScene( (const refdef_t *) VMA(1) );`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:388-389`
/// Output source: `oracle/code/client/cl_cgame.cpp:708-710`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:708-710`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRendersceneArgs {
    refdef: *const c_void,
}

impl CgRRendersceneArgs {
    pub const fn new(refdef: *const c_void) -> Self {
        Self { refdef }
    }

    pub const fn refdef(&self) -> *const c_void {
        self.refdef
    }
}

/// `CG_R_RENDERSCENE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:139`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:388-389`
/// Output source: `oracle/code/client/cl_cgame.cpp:708-710`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:708-710`
pub struct CgRRenderscene;

impl OutboundSysCall for CgRRenderscene {
    type Import = SpCgameImport;
    type Args = CgRRendersceneArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_RENDERSCENE;
}

impl EncodeSysCall for CgRRenderscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.refdef())])
    }
}

impl DecodeSysCallReturn for CgRRenderscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
