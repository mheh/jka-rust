use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_INIT_LEVEL_ITEMS` outbound game-to-engine syscall.
///
/// C signature: `void trap_BotInitLevelItems(void);`
#[derive(Debug)]
pub struct BotlibAiInitLevelItemsArgs;

impl BotlibAiInitLevelItemsArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_AI_INIT_LEVEL_ITEMS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:456`
pub struct BotlibAiInitLevelItems;

impl OutboundSysCall for BotlibAiInitLevelItems {
    type Import = GameImport;
    type Args = BotlibAiInitLevelItemsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_INIT_LEVEL_ITEMS;
}

impl EncodeSysCall for BotlibAiInitLevelItems {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibAiInitLevelItems {
    fn decode_return(_word: isize) -> Self::Output {}
}
