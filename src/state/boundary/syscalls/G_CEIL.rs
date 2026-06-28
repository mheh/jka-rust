use crate::ffi::GameImport;
use crate::ffi::syscalls::pass_float;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct GCeil;

impl OutboundSysCall for GCeil {
    type Args = GCeilArgs;
    type Output = f32;

    const IMPORT: GameImport = GameImport::G_CEIL;
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
