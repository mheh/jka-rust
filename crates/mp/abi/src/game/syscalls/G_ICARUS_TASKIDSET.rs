use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::gentity_s;

/// `G_ICARUS_TASKIDSET` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusTaskidsetArgs {
    ent: *mut gentity_s,
    task_type: c_int,
    task_id: c_int,
}

impl GIcarusTaskidsetArgs {
    pub fn new(ent: *mut gentity_s, task_type: c_int, task_id: c_int) -> Self {
        Self {
            ent,
            task_type,
            task_id,
        }
    }

    pub fn ent(&self) -> *mut gentity_s {
        self.ent
    }
    pub fn task_type(&self) -> c_int {
        self.task_type
    }
    pub fn task_id(&self) -> c_int {
        self.task_id
    }
}

/// `G_ICARUS_TASKIDSET` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:265`
pub struct GIcarusTaskidset;

impl OutboundSysCall for GIcarusTaskidset {
    type Import = MpGameImport;
    type Args = GIcarusTaskidsetArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_TASKIDSET;
}

impl EncodeSysCall for GIcarusTaskidset {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent), a.task_type as isize, a.task_id as isize])
    }
}

impl DecodeSysCallReturn for GIcarusTaskidset {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
