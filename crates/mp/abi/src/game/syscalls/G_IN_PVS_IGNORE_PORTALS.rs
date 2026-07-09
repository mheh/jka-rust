use super::super::MpGameImport;
use mp_qshared::shared::qboolean;
use mp_qshared::shared::vec3_t;

use abi_transport::generic::{
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

/// `G_IN_PVS_IGNORE_PORTALS` MP game imports syscall ABI token.
///
/// Raven: ( const vec3_t p1, const vec3_t p2 );
/// Source: `oracle/codemp/game/g_public.h:193`
pub struct GInPvsIgnorePortals;

impl OutboundSysCall for GInPvsIgnorePortals {
    type Import = MpGameImport;
    type Args = GInPvsIgnorePortalsArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_IN_PVS_IGNORE_PORTALS;
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
