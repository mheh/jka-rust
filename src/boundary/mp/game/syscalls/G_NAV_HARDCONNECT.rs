use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_HARDCONNECT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavHardconnectArgs {
    first: c_int,
    second: c_int,
}

impl GNavHardconnectArgs {
    pub fn new(first: c_int, second: c_int) -> Self {
        Self { first, second }
    }

    pub fn first(&self) -> c_int {
        self.first
    }

    pub fn second(&self) -> c_int {
        self.second
    }
}

pub struct GNavHardconnect;

impl OutboundSysCall for GNavHardconnect {
    type Import = GameImport;
    type Args = GNavHardconnectArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_HARDCONNECT;
}

impl EncodeSysCall for GNavHardconnect {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.first as isize, a.second as isize])
    }
}

impl DecodeSysCallReturn for GNavHardconnect {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
