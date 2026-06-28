use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_BOT_CALCULATEPATHS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GBotCalculatepathsArgs {
    /// RMG flag: non-zero when the map was procedurally generated.
    rmg: c_int,
}

impl GBotCalculatepathsArgs {
    pub fn new(rmg: c_int) -> Self {
        Self { rmg }
    }

    pub fn rmg(&self) -> c_int {
        self.rmg
    }
}

pub struct GBotCalculatepaths;

impl OutboundSysCall for GBotCalculatepaths {
    type Import = GameImport;
    type Args = GBotCalculatepathsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_BOT_CALCULATEPATHS;
}

impl EncodeSysCall for GBotCalculatepaths {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.rmg as isize])
    }
}

impl DecodeSysCallReturn for GBotCalculatepaths {
    fn decode_return(_word: isize) -> Self::Output {}
}
