use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CGAME_SIN`.
///
/// Raven transports the float through the integer syscall ABI with `PASSFLOAT`
/// semantics on the module side and reads it with `VMF(1)` on the engine side.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:658`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:657`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:285`
#[derive(Debug)]
pub struct CgameSinArgs {
    value: f32,
}

impl CgameSinArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `CGAME_SIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:133`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:658`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:657`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:285`
pub struct CgameSin;

impl OutboundSysCall for CgameSin {
    type Import = MpCgameImport;
    type Args = CgameSinArgs;
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_SIN;
}

impl EncodeSysCall for CgameSin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for CgameSin {
    // Raven returns `FloatAsInt(sin(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
