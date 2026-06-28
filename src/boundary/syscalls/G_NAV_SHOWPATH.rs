use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_SHOWPATH` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavShowpathArgs {
    start: c_int,
    end: c_int,
}

impl GNavShowpathArgs {
    pub fn new(start: c_int, end: c_int) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> c_int {
        self.start
    }

    pub fn end(&self) -> c_int {
        self.end
    }
}

pub struct GNavShowpath;

impl OutboundSysCall for GNavShowpath {
    type Args = GNavShowpathArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_SHOWPATH;
}

impl EncodeSysCall for GNavShowpath {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.start as isize, a.end as isize])
    }
}

impl DecodeSysCallReturn for GNavShowpath {
    fn decode_return(_word: isize) -> Self::Output {}
}
