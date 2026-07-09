use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_ADDREFENTITYTOSCENE`.
///
/// Raven wrapper: `syscall( CG_R_ADDREFENTITYTOSCENE, re );`
/// Raven transport forwards the raw `refEntity_t` block through `VMA(1)`.
/// Nothing is drawn until `R_RenderScene` is called.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:331-332`
/// Args source: `oracle/codemp/cgame/cg_local.h:2267`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:894-896`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRAddrefentitytosceneArgs {
    ref_entity: *const c_void,
}

impl CgRAddrefentitytosceneArgs {
    pub const fn new(ref_entity: *const c_void) -> Self {
        Self { ref_entity }
    }

    pub const fn ref_entity(&self) -> *const c_void {
        self.ref_entity
    }
}

/// `CG_R_ADDREFENTITYTOSCENE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:151`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:331-332`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:894-896`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:894-896`
pub struct CgRAddrefentitytoscene;

impl OutboundSysCall for CgRAddrefentitytoscene {
    type Import = MpCgameImport;
    type Args = CgRAddrefentitytosceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDREFENTITYTOSCENE;
}

impl EncodeSysCall for CgRAddrefentitytoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ref_entity())])
    }
}

impl DecodeSysCallReturn for CgRAddrefentitytoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
