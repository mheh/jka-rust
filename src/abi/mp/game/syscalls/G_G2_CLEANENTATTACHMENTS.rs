use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::GameImport;

/// `G_G2_CLEANENTATTACHMENTS` outbound game-to-engine syscall.
///
/// Mirrors `trap_G2API_CleanEntAttachments()`: no arguments, void return.
/// Clears any engine-side Ghoul2 instance↔entity attachments (used at game init).
#[derive(Debug)]
pub struct GG2CleanentattachmentsArgs;

impl GG2CleanentattachmentsArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_G2_CLEANENTATTACHMENTS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:566`
pub struct GG2Cleanentattachments;

impl OutboundSysCall for GG2Cleanentattachments {
    type Import = GameImport;
    type Args = GG2CleanentattachmentsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_CLEANENTATTACHMENTS;
}

impl EncodeSysCall for GG2Cleanentattachments {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GG2Cleanentattachments {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
