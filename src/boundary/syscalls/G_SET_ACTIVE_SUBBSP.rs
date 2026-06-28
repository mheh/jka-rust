use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_SET_ACTIVE_SUBBSP` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GSetActiveSubbspArgs {
    /// Sub-BSP index to activate; negative value clears it.
    index: c_int,
}

impl GSetActiveSubbspArgs {
    pub fn new(index: c_int) -> Self {
        Self { index }
    }

    pub fn index(&self) -> c_int {
        self.index
    }
}

pub struct GSetActiveSubbsp;

impl OutboundSysCall for GSetActiveSubbsp {
    type Args = GSetActiveSubbspArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_SET_ACTIVE_SUBBSP;
}

impl EncodeSysCall for GSetActiveSubbsp {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.index as isize])
    }
}

impl DecodeSysCallReturn for GSetActiveSubbsp {
    fn decode_return(_word: isize) -> Self::Output {}
}
