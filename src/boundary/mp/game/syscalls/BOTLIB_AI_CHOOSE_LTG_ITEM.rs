use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_CHOOSE_LTG_ITEM` outbound game-to-engine syscall.
///
/// C signature: `int trap_BotChooseLTGItem(int goalstate, vec3_t origin, int *inventory, int travelflags)`
#[derive(Debug)]
pub struct BotlibAiChooseLtgItemArgs {
    pub goalstate: c_int,
    pub origin: *const vec3_t,
    pub inventory: *mut c_int,
    pub travelflags: c_int,
}

impl BotlibAiChooseLtgItemArgs {
    pub fn new(goalstate: c_int, origin: *const vec3_t, inventory: *mut c_int, travelflags: c_int) -> Self {
        Self { goalstate, origin, inventory, travelflags }
    }

    pub fn goalstate(&self) -> c_int { self.goalstate }
    pub fn origin(&self) -> *const vec3_t { self.origin }
    pub fn inventory(&self) -> *mut c_int { self.inventory }
    pub fn travelflags(&self) -> c_int { self.travelflags }
}

pub struct BotlibAiChooseLtgItem;

impl OutboundSysCall for BotlibAiChooseLtgItem {
    type Import = GameImport;
    type Args = BotlibAiChooseLtgItemArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_CHOOSE_LTG_ITEM;
}

impl EncodeSysCall for BotlibAiChooseLtgItem {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.goalstate as isize,
            ptr_to_word(a.origin),
            ptr_to_word(a.inventory),
            a.travelflags as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiChooseLtgItem {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
