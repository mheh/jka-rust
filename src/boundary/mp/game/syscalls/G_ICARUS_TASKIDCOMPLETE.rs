use core::ffi::c_int;

use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_TASKIDCOMPLETE` outbound game-to-engine syscall.
///
/// Signals the engine that `ent`'s `task_type` task has finished.
/// Mirrors the C ABI: `void trap_ICARUS_TaskIDComplete(gentity_t *ent, int task_type)`.
#[derive(Debug)]
pub struct GIcarusTaskidcompleteArgs {
    ent: *mut gentity_t,
    task_type: c_int,
}

impl GIcarusTaskidcompleteArgs {
    pub fn new(ent: *mut gentity_t, task_type: c_int) -> Self {
        Self { ent, task_type }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub fn task_type(&self) -> c_int {
        self.task_type
    }
}

pub struct GIcarusTaskidcomplete;

impl OutboundSysCall for GIcarusTaskidcomplete {
    type Import = GameImport;
    type Args = GIcarusTaskidcompleteArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ICARUS_TASKIDCOMPLETE;
}

impl EncodeSysCall for GIcarusTaskidcomplete {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent), a.task_type as isize])
    }
}

impl DecodeSysCallReturn for GIcarusTaskidcomplete {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
