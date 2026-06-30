use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_SIN`.
///
/// Raven transports the float through the integer syscall ABI with `PASSFLOAT`
/// semantics on the module side and reads it with `VMF(1)` on the engine side.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:658`
/// Transport source: `oracle/oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:657`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:285`
#[derive(Debug)]
pub struct UiSinArgs {
    value: f32,
}

impl UiSinArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_SIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:133`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:658`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:657`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:285`
pub struct UiSin;

impl OutboundSysCall for UiSin {
    type Import = MpUiImport;
    type Args = UiSinArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_SIN;
}

impl EncodeSysCall for UiSin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiSin {
    // Raven returns `FloatAsInt(sin(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
