use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_FS_FCLOSE_FILE` outbound game-to-engine syscall.
///
/// C ABI: `void trap_FS_FCloseFile( fileHandle_t f )`
/// syscall: `syscall!(G_FS_FCLOSE_FILE, f)`
#[derive(Debug)]
pub struct GFsFcloseFileArgs {
    /// File handle to close (`fileHandle_t`, which is `int` in C).
    pub f: c_int,
}

impl GFsFcloseFileArgs {
    pub fn new(f: c_int) -> Self {
        Self { f }
    }

    pub fn f(&self) -> c_int {
        self.f
    }
}

/// `G_FS_FCLOSE_FILE` MP game imports syscall ABI token.
///
/// Raven: ( fileHandle_t f );
/// Source: `oracle/oracle/codemp/game/g_public.h:136`
pub struct GFsFcloseFile;

impl OutboundSysCall for GFsFcloseFile {
    type Import = MpGameImport;
    type Args = GFsFcloseFileArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_FS_FCLOSE_FILE;
}

impl EncodeSysCall for GFsFcloseFile {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.f as isize])
    }
}

impl DecodeSysCallReturn for GFsFcloseFile {
    fn decode_return(_word: isize) -> Self::Output {}
}
