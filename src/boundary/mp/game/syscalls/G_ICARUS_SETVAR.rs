use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_SETVAR` outbound game-to-engine syscall.
///
/// Sets ICARUS variable `type_name` to `data` (for task `task_id`, entity `ent_id`).
#[derive(Debug)]
pub struct GIcarusSetvarArgs {
    task_id: c_int,
    ent_id: c_int,
    type_name: *const c_char,
    data: *const c_char,
}

impl GIcarusSetvarArgs {
    pub fn new(task_id: c_int, ent_id: c_int, type_name: *const c_char, data: *const c_char) -> Self {
        Self { task_id, ent_id, type_name, data }
    }

    pub fn task_id(&self) -> c_int { self.task_id }
    pub fn ent_id(&self) -> c_int { self.ent_id }
    pub fn type_name(&self) -> *const c_char { self.type_name }
    pub fn data(&self) -> *const c_char { self.data }
}

pub struct GIcarusSetvar;

impl OutboundSysCall for GIcarusSetvar {
    type Import = GameImport;
    type Args = GIcarusSetvarArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ICARUS_SETVAR;
}

impl EncodeSysCall for GIcarusSetvar {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.task_id as isize,
            a.ent_id as isize,
            ptr_to_word(a.type_name),
            ptr_to_word(a.data),
        ])
    }
}

impl DecodeSysCallReturn for GIcarusSetvar {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
