use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::GameImport;
use core::ffi::c_int;

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

/// `G_NAV_HARDCONNECT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:304`
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
