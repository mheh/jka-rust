use super::super::SpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::qboolean;

/// Arguments for `UI_KEY_SETOVERSTRIKEMODE`.
///
/// Raven wrapper: `syscall( UI_KEY_SETOVERSTRIKEMODE, state );`
/// SP enum source: `oracle/code/ui/ui_public.h:190`
/// SP transport/source evidence: no direct `UI_KEY_SETOVERSTRIKEMODE` case in `oracle/code/client/cl_ui.cpp`.
/// Fallback transport evidence (MP): `oracle/codemp/ui/ui_syscalls.c:239`
/// Transport/switch source (MP): `oracle/codemp/client/cl_ui.cpp:1025-1026`
/// Output/result type source (SP/Multi): `oracle/code/client/cl_keys.cpp:872`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeySetoverstrikemodeArgs {
    state: qboolean,
}

impl UiKeySetoverstrikemodeArgs {
    pub const fn new(state: qboolean) -> Self {
        Self { state }
    }

    pub const fn state(&self) -> qboolean {
        self.state
    }
}

/// `UI_KEY_SETOVERSTRIKEMODE` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:190`
pub struct UiKeySetoverstrikemode;

impl OutboundSysCall for UiKeySetoverstrikemode {
    type Import = SpUiImport;
    type Args = UiKeySetoverstrikemodeArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_SETOVERSTRIKEMODE;
}

impl EncodeSysCall for UiKeySetoverstrikemode {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.state() as isize])
    }
}

impl DecodeSysCallReturn for UiKeySetoverstrikemode {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
