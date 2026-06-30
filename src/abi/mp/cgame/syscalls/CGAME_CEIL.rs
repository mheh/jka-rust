use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CGAME_CEIL`.
///
/// Raven's engine switch reads one float word with `VMF(1)`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:677`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:676`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:293`
#[derive(Debug)]
pub struct CgameCeilArgs {
    value: f32,
}

impl CgameCeilArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `CGAME_CEIL` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:141`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:677`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:676`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:293`
pub struct CgameCeil;

impl OutboundSysCall for CgameCeil {
    type Import = MpCgameImport;
    type Args = CgameCeilArgs;
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_CEIL;
}

impl EncodeSysCall for CgameCeil {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for CgameCeil {
    // Raven returns `FloatAsInt(ceil(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
