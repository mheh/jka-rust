use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_SQRT`.
///
/// The engine handler is `FloatAsInt( sqrt( VMF(1) ) )`: a single float input,
/// passed across the integer-only syscall ABI via `PASSFLOAT`.
#[derive(Debug)]
pub struct GSqrtArgs {
    value: f32,
}

impl GSqrtArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `G_SQRT` outbound game-to-engine syscall.
pub struct GSqrt;

impl OutboundSysCall for GSqrt {
    type Import = GameImport;
    type Args = GSqrtArgs;
    type Output = f32;

    const IMPORT: GameImport = GameImport::G_SQRT;
}

impl EncodeSysCall for GSqrt {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for GSqrt {
    // The engine returns `FloatAsInt(...)`; reinterpret the word's low 32 bits as
    // the float result, mirroring the C `floatint_t` round-trip.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
