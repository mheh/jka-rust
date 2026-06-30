use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::GameImport;
use crate::shared::qboolean;

/// `G_NAV_GETPATHSCALCULATED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetpathscalculatedArgs;

impl GNavGetpathscalculatedArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_GETPATHSCALCULATED` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:338`
pub struct GNavGetpathscalculated;

impl OutboundSysCall for GNavGetpathscalculated {
    type Import = GameImport;
    type Args = GNavGetpathscalculatedArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_GETPATHSCALCULATED;
}

impl EncodeSysCall for GNavGetpathscalculated {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavGetpathscalculated {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
