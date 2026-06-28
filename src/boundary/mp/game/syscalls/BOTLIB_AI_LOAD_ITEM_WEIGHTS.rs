use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_LOAD_ITEM_WEIGHTS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiLoadItemWeightsArgs {
    goalstate: c_int,
    filename: CString,
}

impl BotlibAiLoadItemWeightsArgs {
    pub fn new(goalstate: c_int, filename: CString) -> Self {
        Self { goalstate, filename }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }

    pub fn filename(&self) -> &CString {
        &self.filename
    }
}

pub struct BotlibAiLoadItemWeights;

impl OutboundSysCall for BotlibAiLoadItemWeights {
    type Import = GameImport;
    type Args = BotlibAiLoadItemWeightsArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_LOAD_ITEM_WEIGHTS;
}

impl EncodeSysCall for BotlibAiLoadItemWeights {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.goalstate as isize,
            ptr_to_word(a.filename.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiLoadItemWeights {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
