use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `G_BOT_FREE_CLIENT` MP game imports syscall ABI token.
///
/// Raven: ( int clientNum );
/// Source: `oracle/codemp/game/g_public.h:217`
pub struct GBotFreeClient;

impl OutboundSysCall for GBotFreeClient {
    type Import = MpGameImport;
    type Args = GBotFreeClientArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_BOT_FREE_CLIENT;
}

impl EncodeSysCall for GBotFreeClient {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client_num as isize])
    }
}

impl DecodeSysCallReturn for GBotFreeClient {
    fn decode_return(_word: isize) -> Self::Output {}
}
