use super::super::MpGameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::c_int;

/// `BOTLIB_AI_FREE_ITEM_WEIGHTS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiFreeItemWeightsArgs {
    goalstate: c_int,
}

impl BotlibAiFreeItemWeightsArgs {
    pub fn new(goalstate: c_int) -> Self {
        Self { goalstate }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
}

/// `BOTLIB_AI_FREE_ITEM_WEIGHTS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:459`
pub struct BotlibAiFreeItemWeights;

impl OutboundSysCall for BotlibAiFreeItemWeights {
    type Import = MpGameImport;
    type Args = BotlibAiFreeItemWeightsArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_FREE_ITEM_WEIGHTS;
}

impl EncodeSysCall for BotlibAiFreeItemWeights {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiFreeItemWeights {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
