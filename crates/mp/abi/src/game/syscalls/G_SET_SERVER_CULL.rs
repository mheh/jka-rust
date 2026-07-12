use super::super::MpGameImport;
use abi_transport::pass_float;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_SET_SERVER_CULL` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GSetServerCullArgs {
    cull_distance: f32,
}

impl GSetServerCullArgs {
    pub fn new(cull_distance: f32) -> Self {
        Self { cull_distance }
    }

    pub fn cull_distance(&self) -> f32 {
        self.cull_distance
    }
}

/// `G_SET_SERVER_CULL` MP game imports syscall ABI token.
///
/// Raven: server culling to reduce traffic on open maps -rww
/// Source: `oracle/codemp/game/g_public.h:176`
pub struct GSetServerCull;

impl OutboundSysCall for GSetServerCull {
    type Import = MpGameImport;
    type Args = GSetServerCullArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_SET_SERVER_CULL;
}

impl EncodeSysCall for GSetServerCull {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(a.cull_distance)])
    }
}

impl DecodeSysCallReturn for GSetServerCull {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
