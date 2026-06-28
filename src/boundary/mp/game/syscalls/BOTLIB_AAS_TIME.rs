use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_TIME` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasTimeArgs;

impl BotlibAasTimeArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct BotlibAasTime;

impl OutboundSysCall for BotlibAasTime {
    type Import = GameImport;
    type Args = BotlibAasTimeArgs;
    type Output = f32;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_TIME;
}

impl EncodeSysCall for BotlibAasTime {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibAasTime {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
