use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_GOAL_NAME` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotGoalName(int number, char *name, int size)`
/// The engine writes the goal name into the `name` buffer (out-param).
#[derive(Debug)]
pub struct BotlibAiGoalNameArgs {
    number: c_int,
    name: *mut c_char,
    size: c_int,
}

impl BotlibAiGoalNameArgs {
    pub fn new(number: c_int, name: *mut c_char, size: c_int) -> Self {
        Self { number, name, size }
    }

    pub fn number(&self) -> c_int {
        self.number
    }

    pub fn name(&self) -> *mut c_char {
        self.name
    }

    pub fn size(&self) -> c_int {
        self.size
    }
}

/// `BOTLIB_AI_GOAL_NAME` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:447`
pub struct BotlibAiGoalName;

impl OutboundSysCall for BotlibAiGoalName {
    type Import = MpGameImport;
    type Args = BotlibAiGoalNameArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_GOAL_NAME;
}

impl EncodeSysCall for BotlibAiGoalName {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.number as isize, ptr_to_word(a.name), a.size as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiGoalName {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
