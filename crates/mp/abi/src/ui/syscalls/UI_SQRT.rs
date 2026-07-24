use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `UI_SQRT`.
///
/// Raven's engine switch reads one packed float word with `VMF(1)` and returns
/// `FloatAsInt( sqrt( VMF(1) ) )`.
///
/// Args source: `oracle/codemp/client/cl_ui.cpp:664`
/// Transport source: `oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:663`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:288`
#[derive(Debug)]
pub struct UiSqrtArgs {
    value: f32,
}

impl UiSqrtArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_SQRT` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:136`
/// Output source: `oracle/codemp/client/cl_ui.cpp:664`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:663`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:288`
pub struct UiSqrt;

impl OutboundSysCall for UiSqrt {
    type Import = MpUiImport;
    type Args = UiSqrtArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_SQRT;
}

impl EncodeSysCall for UiSqrt {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiSqrt {
    // Raven returns `FloatAsInt(sqrt(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
