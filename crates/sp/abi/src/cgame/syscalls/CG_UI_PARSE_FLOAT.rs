use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_PARSE_FLOAT`.
///
/// Raven wrapper: `cgi_UI_Parse_Float(value);`
/// Raven transport: `PC_ParseFloat((float *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:588-590`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:865-867`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiParseFloatArgs {
    value: *mut f32,
}

impl CgUiParseFloatArgs {
    /// # Safety
    /// `value` must be valid for writes of a `float`.
    pub const unsafe fn new(value: *mut f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> *mut f32 {
        self.value
    }
}

/// `CG_UI_PARSE_FLOAT` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:198`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:588-590`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:865-867`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:865-867`
pub struct CgUiParseFloat;

impl OutboundSysCall for CgUiParseFloat {
    type Import = SpCgameImport;
    type Args = CgUiParseFloatArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_PARSE_FLOAT;
}

impl EncodeSysCall for CgUiParseFloat {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.value())])
    }
}

impl DecodeSysCallReturn for CgUiParseFloat {
    fn decode_return(_word: isize) -> Self::Output {}
}
