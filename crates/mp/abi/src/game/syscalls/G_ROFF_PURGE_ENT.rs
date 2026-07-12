use super::super::MpGameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::c_int;
use mp_qshared::shared::qboolean;

/// `G_ROFF_PURGE_ENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GRoffPurgeEntArgs {
    ent_id: c_int,
}

impl GRoffPurgeEntArgs {
    pub fn new(ent_id: c_int) -> Self {
        Self { ent_id }
    }

    pub fn ent_id(&self) -> c_int {
        self.ent_id
    }
}

/// `G_ROFF_PURGE_ENT` MP game imports syscall ABI token.
///
/// Raven: qboolean ROFF_PurgeEnt( int entID )
/// Raven: rww - dynamic vm memory allocation!
/// Source: `oracle/codemp/game/g_public.h:245`
pub struct GRoffPurgeEnt;

impl OutboundSysCall for GRoffPurgeEnt {
    type Import = MpGameImport;
    type Args = GRoffPurgeEntArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ROFF_PURGE_ENT;
}

impl EncodeSysCall for GRoffPurgeEnt {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.ent_id as isize])
    }
}

impl DecodeSysCallReturn for GRoffPurgeEnt {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
