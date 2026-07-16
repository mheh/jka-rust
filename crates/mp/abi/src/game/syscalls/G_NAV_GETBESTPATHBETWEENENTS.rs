use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::gentity_s;

/// `G_NAV_GETBESTPATHBETWEENENTS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetbestpathbetweenentsArgs {
    ent: *mut gentity_s,
    goal: *mut gentity_s,
    flags: c_int,
}

impl GNavGetbestpathbetweenentsArgs {
    pub fn new(ent: *mut gentity_s, goal: *mut gentity_s, flags: c_int) -> Self {
        Self { ent, goal, flags }
    }

    pub fn ent(&self) -> *mut gentity_s {
        self.ent
    }

    pub fn goal(&self) -> *mut gentity_s {
        self.goal
    }

    pub fn flags(&self) -> c_int {
        self.flags
    }
}

/// `G_NAV_GETBESTPATHBETWEENENTS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:331`
pub struct GNavGetbestpathbetweenents;

impl OutboundSysCall for GNavGetbestpathbetweenents {
    type Import = MpGameImport;
    type Args = GNavGetbestpathbetweenentsArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETBESTPATHBETWEENENTS;
}

impl EncodeSysCall for GNavGetbestpathbetweenents {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent), ptr_to_word(a.goal), a.flags as isize])
    }
}

impl DecodeSysCallReturn for GNavGetbestpathbetweenents {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
