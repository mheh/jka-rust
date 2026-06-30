use super::super::MpGameImport;
use crate::codemp::game::q_shared_h::siegePers_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_SIEGEPERSSET` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GSiegeperssetArgs {
    pers: *const siegePers_t,
}

impl GSiegeperssetArgs {
    pub fn new(pers: *const siegePers_t) -> Self {
        Self { pers }
    }

    pub fn pers(&self) -> *const siegePers_t {
        self.pers
    }
}

/// `G_SIEGEPERSSET` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:226`
pub struct GSiegepersset;

impl OutboundSysCall for GSiegepersset {
    type Import = MpGameImport;
    type Args = GSiegeperssetArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_SIEGEPERSSET;
}

impl EncodeSysCall for GSiegepersset {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.pers)])
    }
}

impl DecodeSysCallReturn for GSiegepersset {
    fn decode_return(_word: isize) -> Self::Output {}
}
