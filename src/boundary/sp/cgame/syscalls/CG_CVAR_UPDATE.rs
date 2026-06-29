use crate::ffi::types::vmCvar_t;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CVAR_UPDATE`.
///
/// Raven wrapper: `syscall( CG_CVAR_UPDATE, vmCvar );`
/// Raven transport: `Cvar_Update( (vmCvar_t *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:62-64`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:448-450`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCvarUpdateArgs {
    vm_cvar: *mut vmCvar_t,
}

impl CgCvarUpdateArgs {
    /// # Safety
    /// `vm_cvar` must be valid for writes.
    pub const unsafe fn new(vm_cvar: *mut vmCvar_t) -> Self {
        Self { vm_cvar }
    }
}

/// `CG_CVAR_UPDATE` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:65`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:62-64`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:448-450`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:448-450`
pub struct CgCvarUpdate;

impl OutboundSysCall for CgCvarUpdate {
    type Import = SpCgameImport;
    type Args = CgCvarUpdateArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CVAR_UPDATE;
}

impl EncodeSysCall for CgCvarUpdate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.vm_cvar)])
    }
}

impl DecodeSysCallReturn for CgCvarUpdate {
    fn decode_return(_word: isize) -> Self::Output {}
}
