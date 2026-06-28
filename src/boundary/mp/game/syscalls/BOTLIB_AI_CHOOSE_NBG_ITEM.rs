use core::ffi::{c_int, c_void};

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_CHOOSE_NBG_ITEM` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotChooseNBGItem(int goalstate, vec3_t origin, int *inventory, int travelflags, void *ltg, float maxtime)`
#[derive(Debug)]
pub struct BotlibAiChooseNbgItemArgs {
    pub goalstate: c_int,
    pub origin: *const vec3_t,
    pub inventory: *mut c_int,
    pub travelflags: c_int,
    pub ltg: *mut c_void,
    pub maxtime: f32,
}

impl BotlibAiChooseNbgItemArgs {
    pub fn new(
        goalstate: c_int,
        origin: *const vec3_t,
        inventory: *mut c_int,
        travelflags: c_int,
        ltg: *mut c_void,
        maxtime: f32,
    ) -> Self {
        Self { goalstate, origin, inventory, travelflags, ltg, maxtime }
    }

    pub fn goalstate(&self) -> c_int { self.goalstate }
    pub fn origin(&self) -> *const vec3_t { self.origin }
    pub fn inventory(&self) -> *mut c_int { self.inventory }
    pub fn travelflags(&self) -> c_int { self.travelflags }
    pub fn ltg(&self) -> *mut c_void { self.ltg }
    pub fn maxtime(&self) -> f32 { self.maxtime }
}

pub struct BotlibAiChooseNbgItem;

impl OutboundSysCall for BotlibAiChooseNbgItem {
    type Import = GameImport;
    type Args = BotlibAiChooseNbgItemArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_CHOOSE_NBG_ITEM;
}

impl EncodeSysCall for BotlibAiChooseNbgItem {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.goalstate as isize,
            ptr_to_word(a.origin as *const u8),
            ptr_to_word(a.inventory as *const u8),
            a.travelflags as isize,
            ptr_to_word(a.ltg as *const u8),
            pass_float(a.maxtime),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiChooseNbgItem {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
