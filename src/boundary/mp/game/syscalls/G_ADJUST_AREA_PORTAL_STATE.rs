use crate::codemp::game::g_local::gentity_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ADJUST_AREA_PORTAL_STATE` outbound game-to-engine syscall.
///
/// Opens or closes the area portal that `ent` (a door) straddles, updating
/// PVS and area connectivity. `open` is a `qboolean`.
#[derive(Debug)]
pub struct GAdjustAreaPortalStateArgs {
    ent: *mut gentity_t,
    open: qboolean,
}

impl GAdjustAreaPortalStateArgs {
    pub const fn new(ent: *mut gentity_t, open: qboolean) -> Self {
        Self { ent, open }
    }

    pub const fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub const fn open(&self) -> qboolean {
        self.open
    }
}

/// `G_ADJUST_AREA_PORTAL_STATE` MP game imports syscall boundary token.
///
/// Raven: ( gentity_t *ent, qboolean open );
/// Source: `oracle/oracle/codemp/game/g_public.h:195`
pub struct GAdjustAreaPortalState;

impl OutboundSysCall for GAdjustAreaPortalState {
    type Import = GameImport;
    type Args = GAdjustAreaPortalStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ADJUST_AREA_PORTAL_STATE;
}

impl EncodeSysCall for GAdjustAreaPortalState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent), a.open as isize])
    }
}

impl DecodeSysCallReturn for GAdjustAreaPortalState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
