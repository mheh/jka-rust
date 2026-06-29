use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `G_NAV_CLEARALLFAILEDEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavClearallfailededgesArgs;

impl GNavClearallfailededgesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_CLEARALLFAILEDEDGES` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:323`
pub struct GNavClearallfailededges;

impl OutboundSysCall for GNavClearallfailededges {
    type Import = GameImport;
    type Args = GNavClearallfailededgesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_CLEARALLFAILEDEDGES;
}

impl EncodeSysCall for GNavClearallfailededges {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavClearallfailededges {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
