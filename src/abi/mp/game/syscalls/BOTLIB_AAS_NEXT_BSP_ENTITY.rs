use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `BOTLIB_AAS_NEXT_BSP_ENTITY` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:369`
pub struct BotlibAasNextBspEntity;

impl OutboundSysCall for BotlibAasNextBspEntity {
    type Import = MpGameImport;
    type Args = BotlibAasNextBspEntityArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AAS_NEXT_BSP_ENTITY;
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
