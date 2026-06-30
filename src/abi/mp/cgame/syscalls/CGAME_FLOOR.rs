use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CGAME_FLOOR`.
///
/// Raven's engine switch reads one float word with `VMF(1)`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:675`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:674`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:292`
#[derive(Debug)]
pub struct CgameFloorArgs {
    value: f32,
}

impl CgameFloorArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `CGAME_FLOOR` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:140`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:675`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:674`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:292`
pub struct CgameFloor;

impl OutboundSysCall for CgameFloor {
    type Import = MpCgameImport;
    type Args = CgameFloorArgs;
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_FLOOR;
}

impl EncodeSysCall for CgameFloor {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for CgameFloor {
    // Raven returns `FloatAsInt(floor(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
