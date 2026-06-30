use core::ffi::c_int;

use super::super::MpUiExport;
use crate::abi::generic::InboundVmCall;
use crate::shared::qboolean;

/// `UI_KEY_EVENT` MP UI exports vmMain ABI token.
///
/// Raven signature in this enum block is shifted; actual signature is
/// `_UI_KeyEvent( int key, qboolean down )`.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:225`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:552`
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:593` (return 0)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_keys.cpp:1549`
pub struct UiKeyEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiKeyEventArgs {
    key: c_int,
    down: qboolean,
}

impl UiKeyEventArgs {
    pub const fn new(key: c_int, down: qboolean) -> Self {
        Self { key, down }
    }

    pub const fn key(self) -> c_int {
        self.key
    }

    pub const fn down(self) -> qboolean {
        self.down
    }
}

impl InboundVmCall for UiKeyEvent {
    type Command = MpUiExport;
    type Args = UiKeyEventArgs;
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_KEY_EVENT;
}
