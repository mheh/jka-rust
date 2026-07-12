use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `UI_ASIN`.
///
/// The engine handler reads a single `VMF(1)` float and returns
/// `FloatAsInt( Q_asin( VMF(1) ) )`.
///
/// Args source: `oracle/codemp/client/cl_ui.cpp:685`
/// Output source: `oracle/codemp/client/cl_ui.cpp:685`
/// Transport source: `oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:684`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:299`
#[derive(Debug)]
pub struct UiAsinArgs {
    value: f32,
}

impl UiAsinArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_ASIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:147`
/// Output source: `oracle/codemp/client/cl_ui.cpp:685`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:684`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:299`
pub struct UiAsin;

impl OutboundSysCall for UiAsin {
    type Import = MpUiImport;
    type Args = UiAsinArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_ASIN;
}

impl EncodeSysCall for UiAsin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiAsin {
    // The engine returns `FloatAsInt(...)`; reinterpret the word's low 32 bits as
    // the float result, mirroring the Raven `floatint_t` round-trip.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
