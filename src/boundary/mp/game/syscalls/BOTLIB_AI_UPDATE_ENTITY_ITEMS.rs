use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_UPDATE_ENTITY_ITEMS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiUpdateEntityItemsArgs;

impl BotlibAiUpdateEntityItemsArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_AI_UPDATE_ENTITY_ITEMS` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:457`
pub struct BotlibAiUpdateEntityItems;

impl OutboundSysCall for BotlibAiUpdateEntityItems {
    type Import = GameImport;
    type Args = BotlibAiUpdateEntityItemsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_UPDATE_ENTITY_ITEMS;
}

impl EncodeSysCall for BotlibAiUpdateEntityItems {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibAiUpdateEntityItems {
    fn decode_return(_word: isize) -> Self::Output {}
}
