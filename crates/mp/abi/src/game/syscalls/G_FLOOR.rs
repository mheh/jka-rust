use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use abi_transport::pass_float;

/// `G_FLOOR` outbound game-to-engine syscall.
///
/// C ABI: `float floor(float x)` — engine handler: `FloatAsInt(floor(VMF(1)))`.
#[derive(Debug)]
pub struct GFloorArgs {
    x: f32,
}

impl GFloorArgs {
    pub fn new(x: f32) -> Self {
        Self { x }
    }

    pub fn x(&self) -> f32 {
        self.x
    }
}

/// `G_FLOOR` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:286`
pub struct GFloor;

impl OutboundSysCall for GFloor {
    type Import = MpGameImport;
    type Args = GFloorArgs;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::G_FLOOR;
}

impl EncodeSysCall for GFloor {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(a.x)])
    }
}

impl DecodeSysCallReturn for GFloor {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
