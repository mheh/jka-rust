use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

// Flow:
//
//   executable --dllEntry(syscallptr)--> jampgame stores engine syscall pointer
//   executable --vmMain(command, args)--> jampgame runs game code
//   jampgame   --G_ERROR(message)-----> executable/engine via stored syscall pointer
//
// `G_ERROR` is an outbound game-to-engine syscall raised while jampgame is
// processing a `vmMain` request.
// `G_ERROR` asks the engine to abort with a fatal message.

/// Arguments for the `G_ERROR` syscall.
///
/// ABI: `void trap_Error(const char *fmt)`
#[derive(Debug)]
pub struct GErrorArgs {
    /// NUL-terminated message string passed to the engine.
    message: CString,
}

impl GErrorArgs {
    pub fn new(message: CString) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &CString {
        &self.message
    }
}

/// `G_ERROR` aborts the game through the engine and is not expected to return.
pub struct GError;

impl OutboundSysCall for GError {
    type Import = GameImport;
    type Args = GErrorArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ERROR;
}

impl EncodeSysCall for GError {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.message.as_ptr())])
    }
}

impl DecodeSysCallReturn for GError {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
