use core::ffi::c_int;

use super::super::MpCgameExport;
use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// `CG_INIT` MP cgame exports vmMain ABI token.
///
/// Raven: void CG_Init( int serverMessageNum, int serverCommandSequence, int clientNum )
/// Raven: called when the level loads or when the renderer is restarted
/// Raven: reliableCommandSequence will be 0 on fresh loads, but higher for
/// Raven: demos, tourney restarts, or vid_restarts
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:353-360`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:193-195`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:193-195`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_cgame.cpp:1777-1780`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgInitArgs {
    server_message_num: c_int,
    server_command_sequence: c_int,
    client_num: c_int,
}

impl CgInitArgs {
    pub const fn new(
        server_message_num: c_int,
        server_command_sequence: c_int,
        client_num: c_int,
    ) -> Self {
        Self {
            server_message_num,
            server_command_sequence,
            client_num,
        }
    }

    pub const fn server_message_num(self) -> c_int {
        self.server_message_num
    }

    pub const fn server_command_sequence(self) -> c_int {
        self.server_command_sequence
    }

    pub const fn client_num(self) -> c_int {
        self.client_num
    }
}

/// Arguments for `CG_INIT`.
pub struct CgInit;

impl InboundVmCall for CgInit {
    type Command = MpCgameExport;
    type Args = CgInitArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_INIT;
}

impl DecodeVmMain for CgInit {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgInitArgs::new(
            word_to_c_int(transport.arg(0)),
            word_to_c_int(transport.arg(1)),
            word_to_c_int(transport.arg(2)),
        )
    }
}

impl EncodeVmMainReturn for CgInit {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
