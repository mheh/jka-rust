use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use super::super::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_IN_PVS`.
///
/// Both points are read-only `vec3_t` inputs; the engine never writes through
/// them, so they are held as `*const vec3_t` and forwarded by address.
#[derive(Debug)]
pub struct GInPvsArgs {
    p1: *const vec3_t,
    p2: *const vec3_t,
}

impl GInPvsArgs {
    pub const fn new(p1: *const vec3_t, p2: *const vec3_t) -> Self {
        Self { p1, p2 }
    }

    pub const fn p1(&self) -> *const vec3_t {
        self.p1
    }

    pub const fn p2(&self) -> *const vec3_t {
        self.p2
    }
}

/// `G_IN_PVS` outbound game-to-engine syscall.
pub struct GInPvs;

impl OutboundSysCall for GInPvs {
    type Args = GInPvsArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_IN_PVS;
}

impl EncodeSysCall for GInPvs {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.p1()), ptr_to_word(args.p2())])
    }
}

impl DecodeSysCallReturn for GInPvs {
    // `trap_InPVS` returns `qboolean`; the engine's return word carries the flag.
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
