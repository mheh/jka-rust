use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ROFF_CLEAN` outbound game-to-engine syscall.
///
/// Flush all cached ROFF (rotation/origin animation) data.
/// C ABI: `qboolean trap_ROFF_Clean(void)`
#[derive(Debug)]
pub struct GRoffCleanArgs;

impl GRoffCleanArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct GRoffClean;

impl OutboundSysCall for GRoffClean {
    type Args = GRoffCleanArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_ROFF_CLEAN;
}

impl EncodeSysCall for GRoffClean {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GRoffClean {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
