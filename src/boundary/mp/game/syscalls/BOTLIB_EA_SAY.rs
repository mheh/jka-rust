use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_SAY` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaSayArgs {
    client: c_int,
    str: CString,
}

impl BotlibEaSayArgs {
    pub fn new(client: c_int, str: CString) -> Self {
        Self { client, str }
    }

    pub fn client(&self) -> c_int {
        self.client
    }

    pub fn str(&self) -> &CString {
        &self.str
    }
}

/// `BOTLIB_EA_SAY` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:382`
pub struct BotlibEaSay;

impl OutboundSysCall for BotlibEaSay {
    type Import = GameImport;
    type Args = BotlibEaSayArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_SAY;
}

impl EncodeSysCall for BotlibEaSay {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, ptr_to_word(a.str.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibEaSay {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
