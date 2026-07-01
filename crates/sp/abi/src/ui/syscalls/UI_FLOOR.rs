use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use abi_transport::pass_float;

use super::super::SpUiImport;

/// `UI_FLOOR` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:246`
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:843`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:844`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:843-844`
///
/// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` does not include a `UI_FLOOR` case in its visible switch
/// (function-table ABI for this token is intentionally out-of-scope). The transport shape is inferred from
/// MP-side `oracle/oracle/codemp/client/cl_ui.cpp` behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
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

pub struct UiFloor;

impl OutboundSysCall for UiFloor {
    type Import = SpUiImport;
    type Args = UiFloorArgs;
    type Output = f32;

    const IMPORT: SpUiImport = SpUiImport::UI_FLOOR;
}

impl EncodeSysCall for UiFloor {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiFloor {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
