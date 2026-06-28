use core::ffi::c_int;

use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETBESTPATHBETWEENENTS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetbestpathbetweenentsArgs {
    ent: *mut gentity_t,
    goal: *mut gentity_t,
    flags: c_int,
}

impl GNavGetbestpathbetweenentsArgs {
    pub fn new(ent: *mut gentity_t, goal: *mut gentity_t, flags: c_int) -> Self {
        Self { ent, goal, flags }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub fn goal(&self) -> *mut gentity_t {
        self.goal
    }

    pub fn flags(&self) -> c_int {
        self.flags
    }
}

pub struct GNavGetbestpathbetweenents;

impl OutboundSysCall for GNavGetbestpathbetweenents {
    type Args = GNavGetbestpathbetweenentsArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETBESTPATHBETWEENENTS;
}

impl EncodeSysCall for GNavGetbestpathbetweenents {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ent),
            ptr_to_word(a.goal),
            a.flags as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavGetbestpathbetweenents {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
