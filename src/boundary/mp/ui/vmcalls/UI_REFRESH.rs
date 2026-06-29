use core::ffi::c_int;

use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_REFRESH` MP UI exports vmMain boundary token.
///
/// Raven signature in this enum block is shifted; actual signature is
/// `_UI_Refresh( int time )`.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:231`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:554`
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:601` (return 0)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_scrn.cpp:436`
pub struct UiRefresh;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiRefreshArgs {
    time: c_int,
}

impl UiRefreshArgs {
    pub const fn new(time: c_int) -> Self {
        Self { time }
    }

    pub const fn time(self) -> c_int {
        self.time
    }
}

impl InboundVmCall for UiRefresh {
    type Command = MpUiExport;
    type Args = UiRefreshArgs;
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_REFRESH;
}
