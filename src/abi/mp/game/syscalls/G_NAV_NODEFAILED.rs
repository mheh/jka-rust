use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::g_local::gentity_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;
use core::ffi::c_int;

/// `G_NAV_NODEFAILED` outbound game-to-engine syscall.
///
/// Mirrors `trap_Nav_NodeFailed(ent, node_id)` → `qboolean`.
#[derive(Debug)]
pub struct GNavNodefailedArgs {
    ent: *mut gentity_t,
    node_id: c_int,
}

impl GNavNodefailedArgs {
    pub fn new(ent: *mut gentity_t, node_id: c_int) -> Self {
        Self { ent, node_id }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub fn node_id(&self) -> c_int {
        self.node_id
    }
}

/// `G_NAV_NODEFAILED` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:320`
pub struct GNavNodefailed;

impl OutboundSysCall for GNavNodefailed {
    type Import = GameImport;
    type Args = GNavNodefailedArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_NODEFAILED;
}

impl EncodeSysCall for GNavNodefailed {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent), a.node_id as isize])
    }
}

impl DecodeSysCallReturn for GNavNodefailed {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
