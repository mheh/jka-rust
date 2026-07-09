use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use abi_transport::pass_float;

/// `G_CEIL` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GCeilArgs {
    value: f32,
}

impl GCeilArgs {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// `G_CEIL` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:287`
pub struct GCeil;

impl OutboundSysCall for GCeil {
    type Import = MpGameImport;
    type Args = GCeilArgs;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::G_CEIL;
}

impl EncodeSysCall for GCeil {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(a.value)])
    }
}

impl DecodeSysCallReturn for GCeil {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
