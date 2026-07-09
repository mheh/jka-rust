use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `UI_LAN_LOADCACHEDSERVERS` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_LOADCACHEDSERVERS );`
/// Raven transport: `LAN_LoadCachedServers();`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:101`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:298-299`
/// Output source: `oracle/codemp/ui/ui_local.h:972`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1055-1057`
pub struct UiLanLoadcachedservers;

impl OutboundSysCall for UiLanLoadcachedservers {
    type Import = MpUiImport;
    type Args = ();
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_LOADCACHEDSERVERS;
}

impl EncodeSysCall for UiLanLoadcachedservers {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiLanLoadcachedservers {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
