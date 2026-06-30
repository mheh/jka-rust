use super::super::MpGameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ATAN2` outbound game-to-engine syscall.
/// C ABI: float atan2(float y, float x) — TRAP_ATAN2, args at VMF(1)/VMF(2), return FloatAsInt.
#[derive(Debug)]
pub struct GAtan2Args {
    y: f32,
    x: f32,
}

impl GAtan2Args {
    pub fn new(y: f32, x: f32) -> Self {
        Self { y, x }
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn x(&self) -> f32 {
        self.x
    }
}

/// `G_ATAN2` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:281`
pub struct GAtan2;

impl OutboundSysCall for GAtan2 {
    type Import = MpGameImport;
    type Args = GAtan2Args;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::G_ATAN2;
}

impl EncodeSysCall for GAtan2 {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([crate::abi::pass_float(a.y), crate::abi::pass_float(a.x)])
    }
}

impl DecodeSysCallReturn for GAtan2 {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
