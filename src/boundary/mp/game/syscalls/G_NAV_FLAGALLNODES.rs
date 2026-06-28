use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_FLAGALLNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavFlagallnodesArgs {
    new_flag: c_int,
}

impl GNavFlagallnodesArgs {
    pub fn new(new_flag: c_int) -> Self {
        Self { new_flag }
    }

    pub fn new_flag(&self) -> c_int {
        self.new_flag
    }
}

pub struct GNavFlagallnodes;

impl OutboundSysCall for GNavFlagallnodes {
    type Import = GameImport;
    type Args = GNavFlagallnodesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_FLAGALLNODES;
}

impl EncodeSysCall for GNavFlagallnodes {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.new_flag as isize])
    }
}

impl DecodeSysCallReturn for GNavFlagallnodes {
    fn decode_return(_word: isize) -> Self::Output {}
}
