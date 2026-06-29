use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CGAME_ACOS`.
///
/// Raven transports the float through the integer syscall ABI with `PASSFLOAT`
/// on the module side and `VMF(1)` on the engine side.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:683`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:682`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:298`
#[derive(Debug)]
pub struct CgameAcosArgs {
    value: f32,
}

impl CgameAcosArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `CGAME_ACOS` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:146`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:683`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:682`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:298`
pub struct CgameAcos;

impl OutboundSysCall for CgameAcos {
    type Import = MpCgameImport;
    type Args = CgameAcosArgs;
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_ACOS;
}

impl EncodeSysCall for CgameAcos {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for CgameAcos {
    // The engine returns `FloatAsInt(Q_acos(...))`; reinterpret the low 32 bits
    // as the float result, mirroring Raven's `floatint_t` round-trip.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
