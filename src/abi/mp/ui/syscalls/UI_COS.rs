use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::abi::pass_float;

/// Arguments for `UI_COS`.
///
/// Raven's engine switch reads one float word with `VMF(1)` and returns
/// `FloatAsInt( cos( VMF(1) ) )`.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:660`
/// Transport source: `oracle/oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:659`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:286`
#[derive(Debug)]
pub struct UiCosArgs {
    value: f32,
}

impl UiCosArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_COS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:134`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:660`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:659`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:286`
pub struct UiCos;

impl OutboundSysCall for UiCos {
    type Import = MpUiImport;
    type Args = UiCosArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_COS;
}

impl EncodeSysCall for UiCos {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiCos {
    // Raven returns `FloatAsInt(cos(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
