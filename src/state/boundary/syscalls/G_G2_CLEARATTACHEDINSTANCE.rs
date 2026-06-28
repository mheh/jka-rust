use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_CLEARATTACHEDINSTANCE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2ClearattachedinstanceArgs {
    entity_num: c_int,
}

impl GG2ClearattachedinstanceArgs {
    pub fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub fn entity_num(&self) -> c_int {
        self.entity_num
    }
}

pub struct GG2Clearattachedinstance;

impl OutboundSysCall for GG2Clearattachedinstance {
    type Args = GG2ClearattachedinstanceArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_CLEARATTACHEDINSTANCE;
}

impl EncodeSysCall for GG2Clearattachedinstance {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.entity_num as isize])
    }
}

impl DecodeSysCallReturn for GG2Clearattachedinstance {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
