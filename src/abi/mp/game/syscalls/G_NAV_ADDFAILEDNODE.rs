use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_ADDFAILEDNODE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavAddfailednodeArgs {
    ent: *mut gentity_t,
    node_id: i32,
}

impl GNavAddfailednodeArgs {
    pub fn new(ent: *mut gentity_t, node_id: i32) -> Self {
        Self { ent, node_id }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub fn node_id(&self) -> i32 {
        self.node_id
    }
}

/// `G_NAV_ADDFAILEDNODE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:319`
pub struct GNavAddfailednode;

impl OutboundSysCall for GNavAddfailednode {
    type Import = GameImport;
    type Args = GNavAddfailednodeArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_ADDFAILEDNODE;
}

impl EncodeSysCall for GNavAddfailednode {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent), a.node_id as isize])
    }
}

impl DecodeSysCallReturn for GNavAddfailednode {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
