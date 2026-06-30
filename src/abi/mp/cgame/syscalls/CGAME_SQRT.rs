use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::abi::pass_float;

/// Arguments for `CGAME_SQRT`.
///
/// Raven's engine switch reads one packed float word with `VMF(1)` and returns
/// `FloatAsInt( sqrt( VMF(1) ) )`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:664`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:663`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:288`
#[derive(Debug)]
pub struct CgameSqrtArgs {
    value: f32,
}

impl CgameSqrtArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `CGAME_SQRT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:136`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:664`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:663`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:288`
pub struct CgameSqrt;

impl OutboundSysCall for CgameSqrt {
    type Import = MpCgameImport;
    type Args = CgameSqrtArgs;
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_SQRT;
}

impl EncodeSysCall for CgameSqrt {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for CgameSqrt {
    // Raven returns `FloatAsInt(sqrt(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
