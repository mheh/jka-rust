use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, SysCallTransport};

/// `UI_LAN_SAVECACHEDSERVERS` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:224`
pub struct UiLanSavecachedservers;

impl OutboundSysCall for UiLanSavecachedservers {
    type Import = SpUiImport;
    type Args = ();
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_SAVECACHEDSERVERS;
}

/// Raven wrapper: `trap_LAN_SaveCachedServers();`
/// Raven transport: `LAN_SaveServersToCache(); return 0;`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:224`
/// SP args/output source: `oracle/oracle/code/ui/ui_public.h:224`, `oracle/oracle/codemp/ui/ui_local.h:973`
/// Fallback transport/source evidence: `oracle/oracle/codemp/client/cl_ui.cpp:1059-1061`
impl EncodeSysCall for UiLanSavecachedservers {
    fn encode_syscall(_: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiLanSavecachedservers {
    fn decode_return(_word: isize) -> Self::Output {}
}
