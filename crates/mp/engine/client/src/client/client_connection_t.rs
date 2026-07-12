#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS;
use mp_engine_qcommon::qcommon::netchan_t::netchan_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::limits::MAX_STRING_TOKENS;
use mp_qshared::shared::{fileHandle_t, qboolean, MAX_INFO_STRING, MAX_QPATH, MAX_STRING_CHARS};

use mp_qshared::RmAutomapSymbol;

// Raven `#define MAX_OSPATH PATH_MAX` (1024 here, matching other ports of this const).
// Source: oracle/codemp/game/q_shared.h:395
const MAX_OSPATH: usize = 1024;

// `MAX_INFO_STRING` (`q_shared.h:384`) imported from its canonical home in
// `mp_qshared::shared`.

/// Raven `MAX_HEIGHTMAP_SIZE`.
///
/// Source: `oracle/codemp/client/client.h:141`
const MAX_HEIGHTMAP_SIZE: usize = 16000;

/// Raven `MAX_AUTOMAP_SYMBOLS`.
///
/// Source: `oracle/codemp/client/client.h:151`
const MAX_AUTOMAP_SYMBOLS: usize = 512;

/// Raven `clientConnection_t` — client's connection state to the current server.
///
/// Raven: state for reconnecting/downloading/demo playback, cleared each time
/// a server connection is established or dropped.
/// Type definition source: `oracle/codemp/client/client.h:166-234`
#[repr(C)]
pub struct clientConnection_t {
    pub clientNum: i32,
    pub lastPacketSentTime: i32, // for retransmits during connection
    pub lastPacketTime: i32,     // for timeouts

    pub serverAddress: netadr_t,
    pub connectTime: i32,                           // for connection retransmits
    pub connectPacketCount: i32,                    // for display on connection dialog
    pub serverMessage: [c_char; MAX_STRING_TOKENS], // for display on connection dialog

    pub challenge: i32,    // from the server to use for connecting
    pub checksumFeed: i32, // from the server for checksum calculations

    // these are our reliable messages that go to the server
    pub reliableSequence: i32,
    pub reliableAcknowledge: i32, // the last one the server has executed
    pub reliableCommands: [[c_char; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],

    // server message (unreliable) and command (reliable) sequence
    // numbers are NOT cleared at level changes, but continue to
    // increase as long as the connection is valid

    // message sequence is used by both the network layer and the
    // delta compression layer
    pub serverMessageSequence: i32,

    // reliable messages received from server
    pub serverCommandSequence: i32,
    pub lastExecutedServerCommand: i32, // last server command grabbed or executed with CL_GetServerCommand
    pub serverCommands: [[c_char; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],

    // file transfer from server
    pub download: fileHandle_t,
    pub downloadTempName: [c_char; MAX_OSPATH],
    pub downloadName: [c_char; MAX_OSPATH],
    pub downloadNumber: i32,
    pub downloadBlock: i32,                      // block we are waiting for
    pub downloadCount: i32,                      // how many bytes we got
    pub downloadSize: i32,                       // how many bytes we got
    pub downloadList: [c_char; MAX_INFO_STRING], // list of paks we need to download
    pub downloadRestart: qboolean, // if true, we need to do another FS_Restart because we downloaded a pak

    // demo information
    pub demoName: [c_char; MAX_QPATH],
    pub spDemoRecording: qboolean,
    pub demorecording: qboolean,
    pub demoplaying: qboolean,
    pub demowaiting: qboolean, // don't record until a non-delta message is received
    pub firstDemoFrameSkipped: qboolean,
    pub demofile: fileHandle_t,

    pub timeDemoFrames: i32,   // counter of rendered frames
    pub timeDemoStart: i32,    // cls.realtime before first frame
    pub timeDemoBaseTime: i32, // each frame will be at this time + frameNum * 50

    // big stuff at end of structure so most offsets are 15 bits or less
    pub netchan: netchan_t,

    //rwwRMG - added:
    pub rmgSeed: i32,
    pub rmgHeightMapSize: i32,
    pub rmgHeightMap: [u8; MAX_HEIGHTMAP_SIZE],
    pub rmgFlattenMap: [u8; MAX_HEIGHTMAP_SIZE],
    pub rmgAutomapSymbols: [RmAutomapSymbol; MAX_AUTOMAP_SYMBOLS],
    pub rmgAutomapSymbolCount: i32,
}

const _: () = assert!(core::mem::size_of::<clientConnection_t>() == 407048);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, clientNum) == 0);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, lastPacketSentTime) == 4);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, lastPacketTime) == 8);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverAddress) == 12);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, connectTime) == 32);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, connectPacketCount) == 36);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverMessage) == 40);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, challenge) == 1064);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, checksumFeed) == 1068);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, reliableSequence) == 1072);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, reliableAcknowledge) == 1076);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, reliableCommands) == 1080);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverMessageSequence) == 132152);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverCommandSequence) == 132156);
const _: () =
    assert!(core::mem::offset_of!(clientConnection_t, lastExecutedServerCommand) == 132160);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverCommands) == 132164);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, download) == 263236);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadTempName) == 263240);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadName) == 264264);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadNumber) == 265288);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadBlock) == 265292);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadCount) == 265296);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadSize) == 265300);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadList) == 265304);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, downloadRestart) == 266328);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, demoName) == 266332);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, spDemoRecording) == 266396);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, demorecording) == 266400);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, demoplaying) == 266404);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, demowaiting) == 266408);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, firstDemoFrameSkipped) == 266412);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, demofile) == 266416);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, timeDemoFrames) == 266420);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, timeDemoStart) == 266424);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, timeDemoBaseTime) == 266428);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, netchan) == 266432);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, rmgSeed) == 364796);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, rmgHeightMapSize) == 364800);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, rmgHeightMap) == 364804);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, rmgFlattenMap) == 380804);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, rmgAutomapSymbols) == 396804);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, rmgAutomapSymbolCount) == 407044);
