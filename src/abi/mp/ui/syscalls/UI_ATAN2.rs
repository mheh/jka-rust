use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_ATAN2`.
///
/// Raven transport: two packed float words read as `VMF(1)` and `VMF(2)`.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:662`
/// Transport source: `oracle/oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:661`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:287`
#[derive(Debug)]
pub struct UiAtan2Args {
    y: f32,
    x: f32,
}

impl UiAtan2Args {
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

/// `UI_ATAN2` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:135`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:662`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:661`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:287`
pub struct UiAtan2;

impl OutboundSysCall for UiAtan2 {
    type Import = MpUiImport;
    type Args = UiAtan2Args;
    /// Float return transported as an integer word by Raven `FloatAsInt`.
    ///
    /// Output sources: `oracle/oracle/codemp/client/cl_ui.cpp:609`,
    /// `oracle/oracle/codemp/client/cl_ui.cpp:662`
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_ATAN2;
}

impl EncodeSysCall for UiAtan2 {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.y()), pass_float(args.x())])
    }
}

impl DecodeSysCallReturn for UiAtan2 {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
