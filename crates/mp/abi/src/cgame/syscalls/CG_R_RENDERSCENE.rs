use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_RENDERSCENE`.
///
/// Raven wrapper: `syscall( CG_R_RENDERSCENE, fd );`
/// Raven transport: `re.RenderScene( (const refdef_t *)VMA(1) ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:358-361`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2278`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:922-924`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRendersceneArgs {
    refdef: *const c_void,
}

impl CgRRendersceneArgs {
    pub const fn new(refdef: *const c_void) -> Self {
        Self { refdef }
    }
}

/// `CG_R_RENDERSCENE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:158`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:358-361`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:922-924`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:922-924`
pub struct CgRRenderscene;

impl OutboundSysCall for CgRRenderscene {
    type Import = MpCgameImport;
    type Args = CgRRendersceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_RENDERSCENE;
}

impl EncodeSysCall for CgRRenderscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.refdef)])
    }
}

impl DecodeSysCallReturn for CgRRenderscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
