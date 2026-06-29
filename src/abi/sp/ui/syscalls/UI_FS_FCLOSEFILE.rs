use super::super::SpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::fileHandle_t;

/// `UI_FS_FCLOSEFILE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:168`
/// Args source: `oracle/oracle/code/ui/ui_public.h:39`
/// Output source: `oracle/oracle/code/ui/ui_public.h:39`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:925-927`
/// SP caveat: `oracle/oracle/code/client/cl_ui.cpp` does not emit a `UI_FS_FCLOSEFILE` case in this branch.
/// TODO: using MP transport parity until SP table entry is confirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFsFclosefileArgs {
    file: fileHandle_t,
}

impl UiFsFclosefileArgs {
    pub const fn new(file: fileHandle_t) -> Self {
        Self { file }
    }

    pub const fn file(&self) -> fileHandle_t {
        self.file
    }
}

pub struct UiFsFclosefile;

impl OutboundSysCall for UiFsFclosefile {
    type Import = SpUiImport;
    type Args = UiFsFclosefileArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_FS_FCLOSEFILE;
}

impl EncodeSysCall for UiFsFclosefile {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.file() as isize])
    }
}

impl DecodeSysCallReturn for UiFsFclosefile {
    fn decode_return(_word: isize) -> Self::Output {}
}
