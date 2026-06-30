use super::super::SpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::abi::pass_float;

/// `UI_ATAN2` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:244`
/// Args/output source (SP ambiguous): SP `cl_ui.cpp` has no implementation case.
/// Transport/source fallback (MP): `oracle/oracle/codemp/client/cl_ui.cpp:830-831`
/// TODO: SP transport evidence for float ABI is still missing; this follows MP float syscall pattern.
pub struct UiAtan2;

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

impl OutboundSysCall for UiAtan2 {
    type Import = SpUiImport;
    type Args = UiAtan2Args;
    /// Float return transported as an integer word by Raven `FloatAsInt`/`PASSFLOAT` conventions.
    type Output = f32;

    const IMPORT: SpUiImport = SpUiImport::UI_ATAN2;
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
