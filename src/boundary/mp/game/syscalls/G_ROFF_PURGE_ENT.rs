use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::ffi::types::qboolean;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct GRoffPurgeEnt;

impl OutboundSysCall for GRoffPurgeEnt {
    type Import = GameImport;
    type Args = GRoffPurgeEntArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_ROFF_PURGE_ENT;
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
