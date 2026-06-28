use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::codemp::game::q_shared_h::vec3_t;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_ADDRAWPOINT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavAddrawpointArgs {
    point: *const vec3_t,
    flags: c_int,
    radius: c_int,
}

impl GNavAddrawpointArgs {
    pub fn new(point: *const vec3_t, flags: c_int, radius: c_int) -> Self {
        Self { point, flags, radius }
    }

    pub fn point(&self) -> *const vec3_t { self.point }
    pub fn flags(&self) -> c_int { self.flags }
    pub fn radius(&self) -> c_int { self.radius }
}

pub struct GNavAddrawpoint;

impl OutboundSysCall for GNavAddrawpoint {
    type Args = GNavAddrawpointArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_ADDRAWPOINT;
}

impl EncodeSysCall for GNavAddrawpoint {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.point),
            a.flags as isize,
            a.radius as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavAddrawpoint {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
