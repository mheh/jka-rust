use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_SIN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GSinArgs {
    angle: f32,
}

impl GSinArgs {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }
}

/// `G_SIN` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:279`
pub struct GSin;

impl OutboundSysCall for GSin {
    type Import = GameImport;
    type Args = GSinArgs;
    type Output = f32;

    const IMPORT: GameImport = GameImport::G_SIN;
}

impl EncodeSysCall for GSin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(a.angle)])
    }
}

impl DecodeSysCallReturn for GSin {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
