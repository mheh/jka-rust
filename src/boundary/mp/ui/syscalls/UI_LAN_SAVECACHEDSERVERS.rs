use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_LAN_SAVECACHEDSERVERS` MP UI imports syscall boundary token.
///
/// Raven wrapper: `syscall( UI_LAN_SAVECACHEDSERVERS );`
/// Raven transport: `LAN_SaveServersToCache();`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:102`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:294-295`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:973`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1059-1061`
pub struct UiLanSavecachedservers;

impl OutboundSysCall for UiLanSavecachedservers {
    type Import = MpUiImport;
    type Args = ();
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_SAVECACHEDSERVERS;
}

impl EncodeSysCall for UiLanSavecachedservers {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiLanSavecachedservers {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
