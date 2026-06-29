use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_FLOOR`.
///
/// Raven's engine switch reads one float word with `VMF(1)`.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:675`
/// Transport source: `oracle/oracle/codemp/ui/ui_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:674`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:292`
#[derive(Debug)]
pub struct UiFloorArgs {
    value: f32,
}

impl UiFloorArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_FLOOR` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:140`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:675`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:674`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:292`
pub struct UiFloor;

impl OutboundSysCall for UiFloor {
    type Import = MpUiImport;
    type Args = UiFloorArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_FLOOR;
}

impl EncodeSysCall for UiFloor {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiFloor {
    // Raven returns `FloatAsInt(floor(...))`; reinterpret the low 32 bits as f32.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
