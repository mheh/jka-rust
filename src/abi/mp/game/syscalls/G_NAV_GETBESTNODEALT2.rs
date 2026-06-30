use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETBESTNODEALT2` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetbestnodealt2Args {
    start_id: c_int,
    end_id: c_int,
    reject_id: c_int,
}

impl GNavGetbestnodealt2Args {
    pub fn new(start_id: c_int, end_id: c_int, reject_id: c_int) -> Self {
        Self {
            start_id,
            end_id,
            reject_id,
        }
    }

    pub fn start_id(&self) -> c_int {
        self.start_id
    }
    pub fn end_id(&self) -> c_int {
        self.end_id
    }
    pub fn reject_id(&self) -> c_int {
        self.reject_id
    }
}

/// `G_NAV_GETBESTNODEALT2` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:330`
pub struct GNavGetbestnodealt2;

impl OutboundSysCall for GNavGetbestnodealt2 {
    type Import = MpGameImport;
    type Args = GNavGetbestnodealt2Args;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETBESTNODEALT2;
}

impl EncodeSysCall for GNavGetbestnodealt2 {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.start_id as isize, a.end_id as isize, a.reject_id as isize])
    }
}

impl DecodeSysCallReturn for GNavGetbestnodealt2 {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
