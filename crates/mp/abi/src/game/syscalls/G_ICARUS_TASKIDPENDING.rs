use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::common::mp::gentity_t;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_TASKIDPENDING` outbound game-to-engine syscall.
///
/// Queries whether task `task_id` is still pending on entity `ent`.
#[derive(Debug)]
pub struct GIcarusTaskidpendingArgs {
    ent: *mut gentity_t,
    task_id: c_int,
}

impl GIcarusTaskidpendingArgs {
    pub fn new(ent: *mut gentity_t, task_id: c_int) -> Self {
        Self { ent, task_id }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub fn task_id(&self) -> c_int {
        self.task_id
    }
}

/// `G_ICARUS_TASKIDPENDING` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:260`
pub struct GIcarusTaskidpending;

impl OutboundSysCall for GIcarusTaskidpending {
    type Import = MpGameImport;
    type Args = GIcarusTaskidpendingArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_TASKIDPENDING;
}

impl EncodeSysCall for GIcarusTaskidpending {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent), a.task_id as isize])
    }
}

impl DecodeSysCallReturn for GIcarusTaskidpending {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
