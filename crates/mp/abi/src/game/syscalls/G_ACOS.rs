use super::super::MpGameImport;
use abi_transport::pass_float;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ACOS` outbound game-to-engine syscall.
///
/// C ABI: `float trap_Acos(float value)` — engine handler reads `VMF(1)`,
/// returns `FloatAsInt(Q_acos(value))`.
#[derive(Debug)]
pub struct GAcosArgs {
    value: f32,
}

impl GAcosArgs {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// `G_ACOS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:292`
pub struct GAcos;

impl OutboundSysCall for GAcos {
    type Import = MpGameImport;
    type Args = GAcosArgs;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::G_ACOS;
}

impl EncodeSysCall for GAcos {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(a.value)])
    }
}

impl DecodeSysCallReturn for GAcos {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
