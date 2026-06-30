use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CGAME_ATAN2`.
///
/// Raven transport: two packed float words read as `VMF(1)` and `VMF(2)`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:662`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:661`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:287`
#[derive(Debug)]
pub struct CgameAtan2Args {
    y: f32,
    x: f32,
}

impl CgameAtan2Args {
    pub const fn new(y: f32, x: f32) -> Self {
        Self { y, x }
    }

    pub const fn y(&self) -> f32 {
        self.y
    }

    pub const fn x(&self) -> f32 {
        self.x
    }
}

/// `CGAME_ATAN2` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:135`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:662`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:661`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:287`
pub struct CgameAtan2;

impl OutboundSysCall for CgameAtan2 {
    type Import = MpCgameImport;
    type Args = CgameAtan2Args;
    /// Float return transported as an integer word by Raven `FloatAsInt`.
    ///
    /// Output sources: `oracle/oracle/codemp/client/cl_cgame.cpp:609`,
    /// `oracle/oracle/codemp/client/cl_cgame.cpp:662`
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_ATAN2;
}

impl EncodeSysCall for CgameAtan2 {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.y()), pass_float(args.x())])
    }
}

impl DecodeSysCallReturn for CgameAtan2 {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
