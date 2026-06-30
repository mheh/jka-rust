use super::super::MpGameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ASIN` outbound game-to-engine syscall.
///
/// ABI: `FloatAsInt( Q_asin( VMF(1) ) )` — one float arg, float return.
#[derive(Debug)]
pub struct GAsinArgs {
    value: f32,
}

impl GAsinArgs {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// `G_ASIN` MP game imports syscall ABI token.
///
/// Raven: END VM STUFF
/// Raven: rww - BEGIN NPC NAV TRAPS
/// Source: `oracle/oracle/codemp/game/g_public.h:293`
pub struct GAsin;

impl OutboundSysCall for GAsin {
    type Import = MpGameImport;
    type Args = GAsinArgs;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::G_ASIN;
}

impl EncodeSysCall for GAsin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([crate::abi::pass_float(a.value())])
    }
}

impl DecodeSysCallReturn for GAsin {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
