use core::ffi::c_int;
use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibAiFreeItemWeights;

impl OutboundSysCall for BotlibAiFreeItemWeights {
    type Args = BotlibAiFreeItemWeightsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_FREE_ITEM_WEIGHTS;
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
