use std::ffi::CString;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

// Flow:
//
//   executable --dllEntry(syscallptr)--> jampgame stores engine syscall pointer
//   executable --vmMain(command, args)--> jampgame runs game code
//   jampgame   --G_PRINT(message)-----> executable/engine via stored syscall pointer
//
// `G_PRINT` is an outbound game-to-engine syscall raised while jampgame is
// processing a `vmMain` request.
// `G_PRINT` asks the engine to write server-console text.
//
// C ABI: void trap_Printf(const char *fmt)
//   syscall!(G_PRINT, c.as_ptr())  — one pointer arg, void return.

/// Args for the `G_PRINT` outbound syscall.
///
/// Holds the NUL-terminated message string that will be passed to the engine.
#[derive(Debug)]
pub struct GPrintArgs {
    message: CString,
}

impl GPrintArgs {
    pub fn new(message: CString) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &CString {
        &self.message
    }
}

/// `G_PRINT` MP game imports syscall ABI token.
///
/// Raven: ============== general Quake services ==================
/// Raven: ( const char *string );
/// Raven: print message on the local console
/// Source: `oracle/oracle/codemp/game/g_public.h:105`
pub struct GPrint;

impl OutboundSysCall for GPrint {
    type Import = GameImport;
    type Args = GPrintArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_PRINT;
}

impl EncodeSysCall for GPrint {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.message.as_ptr())])
    }
}

impl DecodeSysCallReturn for GPrint {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
