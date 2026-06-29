use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::abi::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_KEY_EVENT`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:208-210`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_keys.cpp:1564-1568`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_keys.cpp:1577-1584`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgKeyEventArgs {
    key: c_int,
    down: qboolean,
}

impl CgKeyEventArgs {
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

/// `CG_KEY_EVENT` MP cgame exports vmMain ABI token.
///
/// Raven: void (*CG_KeyEvent)( int key, qboolean down );
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:384-385`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:208-210`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:208-210`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_keys.cpp:1564-1568`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_keys.cpp:1577-1584`
pub struct CgKeyEvent;

impl InboundVmCall for CgKeyEvent {
    type Command = MpCgameExport;
    type Args = CgKeyEventArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_KEY_EVENT;
}

impl DecodeVmMain for CgKeyEvent {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgKeyEventArgs::new(
            word_to_c_int(transport.arg(0)),
            word_to_c_int(transport.arg(1)) as qboolean,
        )
    }
}

impl EncodeVmMainReturn for CgKeyEvent {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
