use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_SAY_TEAM` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaSayTeamArgs {
    client: c_int,
    str: CString,
}

impl BotlibEaSayTeamArgs {
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

/// `BOTLIB_EA_SAY_TEAM` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:383`
pub struct BotlibEaSayTeam;

impl OutboundSysCall for BotlibEaSayTeam {
    type Import = GameImport;
    type Args = BotlibEaSayTeamArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_SAY_TEAM;
}

impl EncodeSysCall for BotlibEaSayTeam {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, ptr_to_word(a.str.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibEaSayTeam {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
