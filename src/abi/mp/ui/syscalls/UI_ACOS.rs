use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::abi::pass_float;

/// Arguments for `UI_ACOS`.
///
/// Raven transports the float through the integer syscall ABI with `PASSFLOAT`
/// on the module side and `VMF(1)` on the engine side.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:683`
/// Transport source: `oracle/oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:682`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:298`
#[derive(Debug)]
pub struct UiAcosArgs {
    value: f32,
}

impl UiAcosArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_ACOS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:146`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:683`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:682`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:298`
pub struct UiAcos;

impl OutboundSysCall for UiAcos {
    type Import = MpUiImport;
    type Args = UiAcosArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_ACOS;
}

impl EncodeSysCall for UiAcos {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiAcos {
    // The engine returns `FloatAsInt(Q_acos(...))`; reinterpret the low 32 bits
    // as the float result, mirroring Raven's `floatint_t` round-trip.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
