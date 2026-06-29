use crate::ffi::types::vmCvar_t;
use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_CVAR_UPDATE` outbound game-to-engine syscall.
///
/// Refreshes a previously registered cvar mirror (`vmCvar_t`) from the engine's
/// current cvar state.  Mirrors the C ABI: `void trap_Cvar_Update(vmCvar_t *cv)`.
#[derive(Debug)]
pub struct GCvarUpdateArgs {
    /// Pointer to the cvar mirror the engine should refresh in-place.
    cvar: *mut vmCvar_t,
}

impl GCvarUpdateArgs {
    pub fn new(cvar: *mut vmCvar_t) -> Self {
        Self { cvar }
    }

    pub fn cvar(&self) -> *mut vmCvar_t {
        self.cvar
    }
}

/// `G_CVAR_UPDATE` MP game imports syscall ABI token.
///
/// Raven: ( vmCvar_t *vmCvar );
/// Source: `oracle/oracle/codemp/game/g_public.h:122`
pub struct GCvarUpdate;

impl OutboundSysCall for GCvarUpdate {
    type Import = GameImport;
    type Args = GCvarUpdateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_CVAR_UPDATE;
}

impl EncodeSysCall for GCvarUpdate {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.cvar())])
    }
}

impl DecodeSysCallReturn for GCvarUpdate {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
