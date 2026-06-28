use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETPATHCOST` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetpathcostArgs {
    start_id: c_int,
    end_id: c_int,
}

impl GNavGetpathcostArgs {
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

pub struct GNavGetpathcost;

impl OutboundSysCall for GNavGetpathcost {
    type Import = GameImport;
    type Args = GNavGetpathcostArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETPATHCOST;
}

impl EncodeSysCall for GNavGetpathcost {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.start_id as isize, a.end_id as isize])
    }
}

impl DecodeSysCallReturn for GNavGetpathcost {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
