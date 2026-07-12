use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vmCvar_t;

/// `UI_CVAR_UPDATE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/code/ui/ui_public.h:203`
/// Args source: `oracle/code/client/cl_ui.cpp:387-389`.
/// Output source: `oracle/code/client/cl_ui.cpp:387-389`.
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:387-389`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarUpdateArgs {
    cvar: *mut vmCvar_t,
}

impl UiCvarUpdateArgs {
    /// Construct `Cvar_Update( vmCvar )` payload.
    pub const unsafe fn new(cvar: *mut vmCvar_t) -> Self {
        Self { cvar }
    }

    pub const fn cvar(&self) -> *mut vmCvar_t {
        self.cvar
    }
}

pub struct UiCvarUpdate;

impl OutboundSysCall for UiCvarUpdate {
    type Import = SpUiImport;
    type Args = UiCvarUpdateArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_UPDATE;
}

impl EncodeSysCall for UiCvarUpdate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.cvar())])
    }
}

impl DecodeSysCallReturn for UiCvarUpdate {
    fn decode_return(_word: isize) -> Self::Output {}
}
