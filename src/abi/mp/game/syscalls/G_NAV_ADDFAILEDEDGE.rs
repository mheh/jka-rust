use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_ADDFAILEDEDGE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavAddfailededgeArgs {
    ent_id: c_int,
    start_id: c_int,
    end_id: c_int,
}

impl GNavAddfailededgeArgs {
    pub fn new(ent_id: c_int, start_id: c_int, end_id: c_int) -> Self {
        Self {
            ent_id,
            start_id,
            end_id,
        }
    }

    pub fn ent_id(&self) -> c_int {
        self.ent_id
    }
    pub fn start_id(&self) -> c_int {
        self.start_id
    }
    pub fn end_id(&self) -> c_int {
        self.end_id
    }
}

/// `G_NAV_ADDFAILEDEDGE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:325`
pub struct GNavAddfailededge;

impl OutboundSysCall for GNavAddfailededge {
    type Import = GameImport;
    type Args = GNavAddfailededgeArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_ADDFAILEDEDGE;
}

impl EncodeSysCall for GNavAddfailededge {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.ent_id as isize, a.start_id as isize, a.end_id as isize])
    }
}

impl DecodeSysCallReturn for GNavAddfailededge {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
