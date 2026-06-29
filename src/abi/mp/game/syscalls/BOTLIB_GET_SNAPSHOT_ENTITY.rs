use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use core::ffi::c_int;

/// `BOTLIB_GET_SNAPSHOT_ENTITY` outbound game-to-engine syscall.
///
/// Returns the entity number at snapshot slot `sequence` for `client_num`.
#[derive(Debug)]
pub struct BotlibGetSnapshotEntityArgs {
    client_num: c_int,
    sequence: c_int,
}

impl BotlibGetSnapshotEntityArgs {
    pub fn new(client_num: c_int, sequence: c_int) -> Self {
        Self {
            client_num,
            sequence,
        }
    }

    pub fn client_num(&self) -> c_int {
        self.client_num
    }

    pub fn sequence(&self) -> c_int {
        self.sequence
    }
}

/// `BOTLIB_GET_SNAPSHOT_ENTITY` MP game imports syscall ABI token.
///
/// Raven: ( int client, int ent );
/// Source: `oracle/oracle/codemp/game/g_public.h:352`
pub struct BotlibGetSnapshotEntity;

impl OutboundSysCall for BotlibGetSnapshotEntity {
    type Import = GameImport;
    type Args = BotlibGetSnapshotEntityArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_GET_SNAPSHOT_ENTITY;
}

impl EncodeSysCall for BotlibGetSnapshotEntity {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client_num as isize, a.sequence as isize])
    }
}

impl DecodeSysCallReturn for BotlibGetSnapshotEntity {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
