use core::ffi::c_int;

use super::super::MpUiExport;
use abi_transport::generic::InboundVmCall;

/// `UI_MOUSE_EVENT` MP UI exports vmMain ABI token.
///
/// Raven signature in this enum block is shifted; actual signature is
/// `_UI_MouseEvent( int dx, int dy )`.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:228`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:553`
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:597` (return 0)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_input.cpp:1006`
pub struct UiMouseEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiMouseEventArgs {
    dx: c_int,
    dy: c_int,
}

impl UiMouseEventArgs {
    pub const fn new(dx: c_int, dy: c_int) -> Self {
        Self { dx, dy }
    }

    pub const fn dx(self) -> c_int {
        self.dx
    }

    pub const fn dy(self) -> c_int {
        self.dy
    }
}

impl InboundVmCall for UiMouseEvent {
    type Command = MpUiExport;
    type Args = UiMouseEventArgs;
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_MOUSE_EVENT;
}
