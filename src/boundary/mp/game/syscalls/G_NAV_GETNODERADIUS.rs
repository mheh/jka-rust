use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETNODERADIUS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetnoderadiusArgs {
    node_id: c_int,
}

impl GNavGetnoderadiusArgs {
    pub fn new(node_id: c_int) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> c_int {
        self.node_id
    }
}

pub struct GNavGetnoderadius;

impl OutboundSysCall for GNavGetnoderadius {
    type Import = GameImport;
    type Args = GNavGetnoderadiusArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETNODERADIUS;
}

impl EncodeSysCall for GNavGetnoderadius {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.node_id as isize])
    }
}

impl DecodeSysCallReturn for GNavGetnoderadius {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
