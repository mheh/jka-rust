use super::super::MpGameImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `G_ROFF_CLEAN` MP game imports syscall ABI token.
///
/// Raven: qboolean	ROFF_Clean(void);
/// Source: `oracle/codemp/game/g_public.h:241`
pub struct GRoffClean;

impl OutboundSysCall for GRoffClean {
    type Import = MpGameImport;
    type Args = GRoffCleanArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ROFF_CLEAN;
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
