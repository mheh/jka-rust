use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CGAME_COS`.
///
/// Raven's engine switch reads one float word with `VMF(1)` and returns
/// `FloatAsInt( cos( VMF(1) ) )`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:660`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:659`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:286`
#[derive(Debug)]
pub struct CgameCosArgs {
    value: f32,
}

impl CgameCosArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `CGAME_COS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:134`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:660`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:659`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:286`
pub struct CgameCos;

impl OutboundSysCall for CgameCos {
    type Import = MpCgameImport;
    type Args = CgameCosArgs;
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_COS;
}

impl EncodeSysCall for CgameCos {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for CgameCos {
    // Raven returns `FloatAsInt(cos(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
