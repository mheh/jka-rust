use super::super::SpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// `UI_CEIL` SP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:247`
/// Output source (SP ambiguous): `oracle/oracle/code/ui/ui_public.h:247`
/// Args/output fallback source (MP): `oracle/oracle/codemp/client/cl_ui.cpp:843-846`
/// SP caveat: `oracle/oracle/code/client/cl_ui.cpp` has no SP switch case for this token.
/// TODO: SP transport evidence for float ABI is still missing; this follows MP float syscall pattern.
pub struct UiCeil;

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

impl OutboundSysCall for UiCeil {
    type Import = SpUiImport;
    type Args = UiCeilArgs;
    type Output = f32;

    const IMPORT: SpUiImport = SpUiImport::UI_CEIL;
}

impl EncodeSysCall for UiCeil {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiCeil {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
