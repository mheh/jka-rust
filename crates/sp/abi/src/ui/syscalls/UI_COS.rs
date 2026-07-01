use super::super::SpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use abi_transport::pass_float;

/// `UI_COS` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:243`
/// Args/output source: SP-side transport implementation absent from
/// `oracle/oracle/code/ui/ui_syscalls.cpp` / `oracle/oracle/code/client/cl_ui.cpp`;
/// fallback source (same Raven profile): `oracle/oracle/codemp/ui/ui_syscalls.c` and
/// `oracle/oracle/codemp/client/cl_ui.cpp:659-660`.
/// Transport/switch source: fallback reference `oracle/oracle/codemp/client/cl_ui.cpp:659-660`.
/// TODO: SP transport evidence for float ABI is still missing; this mirrors MP float syscall pattern.
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

pub struct UiCos;

impl OutboundSysCall for UiCos {
    type Import = SpUiImport;
    type Args = UiCosArgs;
    type Output = f32;

    const IMPORT: SpUiImport = SpUiImport::UI_COS;
}

impl EncodeSysCall for UiCos {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiCos {
    // `trap` returns cos result via FloatAsInt convention.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
