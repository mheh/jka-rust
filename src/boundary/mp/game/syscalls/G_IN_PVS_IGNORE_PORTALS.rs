use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_IN_PVS_IGNORE_PORTALS` outbound game-to-engine syscall.
///
/// Returns whether `p2` is in `p1`'s PVS, ignoring area portals.
#[derive(Debug)]
pub struct GInPvsIgnorePortalsArgs {
    p1: *const vec3_t,
    p2: *const vec3_t,
}

impl GInPvsIgnorePortalsArgs {
    pub fn new(p1: *const vec3_t, p2: *const vec3_t) -> Self {
        Self { p1, p2 }
    }

    pub fn p1(&self) -> *const vec3_t {
        self.p1
    }

    pub fn p2(&self) -> *const vec3_t {
        self.p2
    }
}

/// `G_IN_PVS_IGNORE_PORTALS` MP game imports syscall boundary token.
///
/// Raven: ( const vec3_t p1, const vec3_t p2 );
/// Source: `oracle/oracle/codemp/game/g_public.h:193`
pub struct GInPvsIgnorePortals;

impl OutboundSysCall for GInPvsIgnorePortals {
    type Import = GameImport;
    type Args = GInPvsIgnorePortalsArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_IN_PVS_IGNORE_PORTALS;
}

impl EncodeSysCall for GInPvsIgnorePortals {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.p1), ptr_to_word(a.p2)])
    }
}

impl DecodeSysCallReturn for GInPvsIgnorePortals {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
