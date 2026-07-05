use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

// Flow:
//
//   executable --vmMain(BOTAI_START_FRAME, time, ...)--> jampgame
//   jampgame   --BotAIStartFrame(time)-----------------> advance bot AI state
//   jampgame   --return int----------------------------> executable
//
// `BOTAI_START_FRAME` is an inbound executable-to-game call raised once per bot
// frame to advance bot library and AI state.

/// Arguments for `BOTAI_START_FRAME`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BotAiStartFrameArgs {
    time: c_int,
}

impl BotAiStartFrameArgs {
    pub const fn new(time: c_int) -> Self {
        Self { time }
    }

    pub const fn time(self) -> c_int {
        self.time
    }
}

/// `BOTAI_START_FRAME` MP game exports vmMain ABI token.
///
/// Raven: ( int time );
/// Source: `oracle/oracle/codemp/game/g_public.h:764`
pub struct BotAiStartFrame;

impl InboundVmCall for BotAiStartFrame {
    type Command = MpGameExport;
    type Args = BotAiStartFrameArgs;
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::BOTAI_START_FRAME;
}

impl DecodeVmMain for BotAiStartFrame {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `BotAIStartFrame( arg0 )` — g_main.c:546.
        BotAiStartFrameArgs::new(word_to_c_int(t.arg(0)))
    }
}

impl EncodeVmMainReturn for BotAiStartFrame {
    fn encode_return(output: Self::Output) -> isize {
        // `return BotAIStartFrame( arg0 );` — g_main.c:546.
        output as isize
    }
}
