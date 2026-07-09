use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use abi_transport::pass_float;

/// `G_COS` outbound game-to-engine syscall.
/// C ABI: `FloatAsInt cos(VMF(1))` — one float arg, returns float bits.
#[derive(Debug)]
pub struct GCosArgs {
    angle: f32,
}

impl GCosArgs {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }
}

/// `G_COS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:280`
pub struct GCos;

impl OutboundSysCall for GCos {
    type Import = MpGameImport;
    type Args = GCosArgs;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::G_COS;
}

impl EncodeSysCall for GCos {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(a.angle)])
    }
}

impl DecodeSysCallReturn for GCos {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
