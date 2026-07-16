use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::gentity_s;

/// `G_ICARUS_TASKIDCOMPLETE` outbound game-to-engine syscall.
///
/// Signals the engine that `ent`'s `task_type` task has finished.
/// Mirrors the C ABI: `void trap_ICARUS_TaskIDComplete(gentity_t *ent, int task_type)`.
#[derive(Debug)]
pub struct GIcarusTaskidcompleteArgs {
    ent: *mut gentity_s,
    task_type: c_int,
}

impl GIcarusTaskidcompleteArgs {
    pub fn new(ent: *mut gentity_s, task_type: c_int) -> Self {
        Self { ent, task_type }
    }

    pub fn ent(&self) -> *mut gentity_s {
        self.ent
    }

    pub fn task_type(&self) -> c_int {
        self.task_type
    }
}

/// `G_ICARUS_TASKIDCOMPLETE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:266`
pub struct GIcarusTaskidcomplete;

impl OutboundSysCall for GIcarusTaskidcomplete {
    type Import = MpGameImport;
    type Args = GIcarusTaskidcompleteArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_TASKIDCOMPLETE;
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
