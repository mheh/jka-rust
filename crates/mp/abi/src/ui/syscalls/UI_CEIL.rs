use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `UI_CEIL`.
///
/// Raven's engine switch reads one float word with `VMF(1)`.
///
/// Args source: `oracle/codemp/client/cl_ui.cpp:677`
/// Transport source: `oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:676`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:293`
#[derive(Debug)]
pub struct UiCeilArgs {
    value: f32,
}

impl UiCeilArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_CEIL` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:141`
/// Output source: `oracle/codemp/client/cl_ui.cpp:677`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:676`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:293`
pub struct UiCeil;

impl OutboundSysCall for UiCeil {
    type Import = MpUiImport;
    type Args = UiCeilArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_CEIL;
}

impl EncodeSysCall for UiCeil {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiCeil {
    // Raven returns `FloatAsInt(ceil(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
