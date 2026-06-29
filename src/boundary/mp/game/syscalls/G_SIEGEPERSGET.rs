use crate::codemp::game::q_shared_h::siegePers_t;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for the `G_SIEGEPERSGET` outbound game-to-engine syscall.
///
/// The engine writes siege persistent data into the caller-supplied `*mut siegePers_t`
/// out-parameter; it is kept as a raw pointer in `Args` per ABI convention.
#[derive(Debug)]
pub struct GSiegepersgetArgs {
    pers: *mut siegePers_t,
}

impl GSiegepersgetArgs {
    pub fn new(pers: *mut siegePers_t) -> Self {
        Self { pers }
    }

    pub fn pers(&self) -> *mut siegePers_t {
        self.pers
    }
}

/// `G_SIEGEPERSGET` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:227`
pub struct GSiegepersget;

impl OutboundSysCall for GSiegepersget {
    type Import = GameImport;
    type Args = GSiegepersgetArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_SIEGEPERSGET;
}

impl EncodeSysCall for GSiegepersget {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.pers)])
    }
}

impl DecodeSysCallReturn for GSiegepersget {
    fn decode_return(_word: isize) -> Self::Output {}
}
