use super::super::MpGameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ROFF_UPDATE_ENTITIES` outbound game-to-engine syscall.
///
/// Advances every entity currently playing a ROFF (rotation/origin animation).
/// Takes no arguments and returns nothing.
#[derive(Debug)]
pub struct GRoffUpdateEntitiesArgs;

impl GRoffUpdateEntitiesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_ROFF_UPDATE_ENTITIES` MP game imports syscall ABI token.
///
/// Raven: void		ROFF_UpdateEntities(void);
/// Source: `oracle/codemp/game/g_public.h:242`
pub struct GRoffUpdateEntities;

impl OutboundSysCall for GRoffUpdateEntities {
    type Import = MpGameImport;
    type Args = GRoffUpdateEntitiesArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_ROFF_UPDATE_ENTITIES;
}

impl EncodeSysCall for GRoffUpdateEntities {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GRoffUpdateEntities {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
