use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `G_NAV_SHOWPATH` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:307`
pub struct GNavShowpath;

impl OutboundSysCall for GNavShowpath {
    type Import = MpGameImport;
    type Args = GNavShowpathArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_SHOWPATH;
}

impl EncodeSysCall for GNavShowpath {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.start as isize, a.end as isize])
    }
}

impl DecodeSysCallReturn for GNavShowpath {
    fn decode_return(_word: isize) -> Self::Output {}
}
