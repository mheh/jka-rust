use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_BOT_FREE_CLIENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GBotFreeClientArgs {
    client_num: c_int,
}

impl GBotFreeClientArgs {
    pub fn new(client_num: c_int) -> Self {
        Self { client_num }
    }

    pub fn client_num(&self) -> c_int {
        self.client_num
    }
}

pub struct GBotFreeClient;

impl OutboundSysCall for GBotFreeClient {
    type Import = GameImport;
    type Args = GBotFreeClientArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_BOT_FREE_CLIENT;
}

impl EncodeSysCall for GBotFreeClient {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client_num as isize])
    }
}

impl DecodeSysCallReturn for GBotFreeClient {
    fn decode_return(_word: isize) -> Self::Output {}
}
