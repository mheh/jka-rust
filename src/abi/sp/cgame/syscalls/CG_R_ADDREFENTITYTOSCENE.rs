use core::ffi::c_void;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_ADDREFENTITYTOSCENE`.
///
/// Raven wrapper: `cgi_R_AddRefEntityToScene( const refEntity_t *re )`
/// Raven transport: `re.AddRefEntityToScene( (const refEntity_t *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:366-367`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:689-691`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:689-691`
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

/// `CG_R_ADDREFENTITYTOSCENE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:132`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:366-367`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:689-691`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:689-691`
pub struct CgRAddrefentitytoscene;

impl OutboundSysCall for CgRAddrefentitytoscene {
    type Import = SpCgameImport;
    type Args = CgRAddrefentitytosceneArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_ADDREFENTITYTOSCENE;
}

impl EncodeSysCall for CgRAddrefentitytoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ref_entity())])
    }
}

impl DecodeSysCallReturn for CgRAddrefentitytoscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
