use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::vmCvar_t;

/// `CG_CVAR_UPDATE` outbound cgame-to-engine syscall.
///
/// Refreshes a previously registered cvar mirror (`vmCvar_t`) from the engine's
/// current cvar state. Mirrors the C ABI: `void trap_Cvar_Update(vmCvar_t *vmCvar)`.
#[derive(Debug)]
pub struct CgCvarUpdateArgs {
    /// Pointer to the cvar mirror the engine should refresh in-place.
    cvar: *mut vmCvar_t,
}

impl CgCvarUpdateArgs {
    pub fn new(cvar: *mut vmCvar_t) -> Self {
        Self { cvar }
    }

    pub fn cvar(&self) -> *mut vmCvar_t {
        self.cvar
    }
}

/// `CG_CVAR_UPDATE` MP cgame imports syscall boundary token.
///
/// Raven: ( vmCvar_t *vmCvar );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:66`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:54-55`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:54`
/// Transport source: `oracle/oracle/codemp/client/cl_cgame.cpp:717-719`
pub struct CgCvarUpdate;

impl OutboundSysCall for CgCvarUpdate {
    type Import = MpCgameImport;
    type Args = CgCvarUpdateArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_UPDATE;
}

impl EncodeSysCall for CgCvarUpdate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.cvar())])
    }
}

impl DecodeSysCallReturn for CgCvarUpdate {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
