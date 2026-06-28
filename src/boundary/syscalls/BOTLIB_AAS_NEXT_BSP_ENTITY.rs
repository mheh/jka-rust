use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_NEXT_BSP_ENTITY` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasNextBspEntityArgs {
    ent: c_int,
}

impl BotlibAasNextBspEntityArgs {
    pub fn new(ent: c_int) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> c_int {
        self.ent
    }
}

pub struct BotlibAasNextBspEntity;

impl OutboundSysCall for BotlibAasNextBspEntity {
    type Args = BotlibAasNextBspEntityArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_NEXT_BSP_ENTITY;
}

impl EncodeSysCall for BotlibAasNextBspEntity {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.ent as isize])
    }
}

impl DecodeSysCallReturn for BotlibAasNextBspEntity {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
