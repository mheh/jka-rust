use core::ffi::c_int;

use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_TASKIDSET` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusTaskidsetArgs {
    ent: *mut gentity_t,
    task_type: c_int,
    task_id: c_int,
}

impl GIcarusTaskidsetArgs {
    pub fn new(ent: *mut gentity_t, task_type: c_int, task_id: c_int) -> Self {
        Self { ent, task_type, task_id }
    }

    pub fn ent(&self) -> *mut gentity_t { self.ent }
    pub fn task_type(&self) -> c_int { self.task_type }
    pub fn task_id(&self) -> c_int { self.task_id }
}

pub struct GIcarusTaskidset;

impl OutboundSysCall for GIcarusTaskidset {
    type Import = GameImport;
    type Args = GIcarusTaskidsetArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ICARUS_TASKIDSET;
}

impl EncodeSysCall for GIcarusTaskidset {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ent),
            a.task_type as isize,
            a.task_id as isize,
        ])
    }
}

impl DecodeSysCallReturn for GIcarusTaskidset {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
