use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETEDGECOST` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetedgecostArgs {
    start_id: c_int,
    end_id: c_int,
}

impl GNavGetedgecostArgs {
    pub fn new(start_id: c_int, end_id: c_int) -> Self {
        Self { start_id, end_id }
    }

    pub fn start_id(&self) -> c_int {
        self.start_id
    }

    pub fn end_id(&self) -> c_int {
        self.end_id
    }
}

/// `G_NAV_GETEDGECOST` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:316`
pub struct GNavGetedgecost;

impl OutboundSysCall for GNavGetedgecost {
    type Import = MpGameImport;
    type Args = GNavGetedgecostArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETEDGECOST;
}

impl EncodeSysCall for GNavGetedgecost {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.start_id as isize, a.end_id as isize])
    }
}

impl DecodeSysCallReturn for GNavGetedgecost {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
