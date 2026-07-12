#![allow(non_camel_case_types, non_snake_case)]

/// Raven `PACKET_BACKUP` — number of old messages that must be kept on client
/// and server for delta compression and ping estimation.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:98`
pub const PACKET_BACKUP: usize = 32;

/// Raven `PACKET_MASK`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:100`
pub const PACKET_MASK: usize = PACKET_BACKUP - 1;

/// Raven `MAX_PACKET_USERCMDS` — max number of `usercmd_t` in a packet.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:102`
pub const MAX_PACKET_USERCMDS: usize = 32;

/// Raven `PORT_ANY`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:104`
pub const PORT_ANY: i32 = -1;

/// Raven `MAX_RELIABLE_COMMANDS` — max string commands buffered for
/// retransmit.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:106`
pub const MAX_RELIABLE_COMMANDS: usize = 128;

/// Raven `MAX_MSGLEN` — max length of a message, which may be fragmented into
/// multiple transport packets.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:150`
pub const MAX_MSGLEN: usize = 49152;

/// Raven `MAX_DOWNLOAD_WINDOW` — max of eight download frames.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:155`
pub const MAX_DOWNLOAD_WINDOW: usize = 8;

/// Raven `MAX_DOWNLOAD_BLKSIZE` — 2048 byte block chunks.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:156`
pub const MAX_DOWNLOAD_BLKSIZE: usize = 2048;
