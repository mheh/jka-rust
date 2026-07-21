#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use core::mem::zeroed;
use core::ptr::null_mut;

use mp_engine_qcommon::qcommon::net_limits::{
    MAX_DOWNLOAD_WINDOW, MAX_RELIABLE_COMMANDS, PACKET_BACKUP,
};
use mp_engine_qcommon::qcommon::netchan_t::netchan_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::{fileHandle_t, qboolean, MAX_INFO_STRING, MAX_STRING_CHARS};

use super::client_snapshot_t::clientSnapshot_t;
use super::client_state_t::clientState_t;

// `MAX_INFO_STRING` (`q_shared.h:384`) imported from its canonical home in
// `mp_qshared::shared`.

/// Raven `client_t` — server-side per-client connection state.
///
/// (§D12 internal-only shape: `client_t` never crosses the DLL seam — the game
/// module sees `playerState`/`userinfo` only — so `name`/`downloadName` are
/// owned `String`s and the old `#[repr(C)]` layout asserts are dropped. Index
/// math over `svs.clients` still uses `size_of::<client_t>()` as the Vec's
/// element stride, which is repr-independent.)
///
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:124-182`
pub struct client_t {
    pub state: clientState_t,
    /// name, etc
    pub userinfo: [c_char; MAX_INFO_STRING],

    /// see if he has been sent an svc_setgame
    pub sentGamedir: qboolean,

    pub reliableCommands: [[c_char; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],
    /// last added reliable message, not necesarily sent or acknowledged yet
    pub reliableSequence: i32,
    /// last acknowledged reliable message
    pub reliableAcknowledge: i32,
    /// last sent reliable message, not necesarily acknowledged yet
    pub reliableSent: i32,
    pub messageAcknowledge: i32,

    /// netchan->outgoingSequence of gamestate
    pub gamestateMessageNum: i32,
    pub challenge: i32,

    pub lastUsercmd: usercmd_t,
    /// for delta compression
    pub lastMessageNum: i32,
    /// reliable client message sequence
    pub lastClientCommand: i32,
    pub lastClientCommandString: [c_char; MAX_STRING_CHARS],
    /// SV_GentityNum(clientnum)
    pub gentity: *mut sharedEntity_t,
    /// extracted from userinfo, high bits masked
    pub name: String,

    // downloading
    /// if not empty string, we are downloading
    pub downloadName: String,
    /// file being downloaded
    pub download: fileHandle_t,
    /// total bytes (can't use EOF because of paks)
    pub downloadSize: i32,
    /// bytes sent
    pub downloadCount: i32,
    /// last block we sent to the client, awaiting ack
    pub downloadClientBlock: i32,
    /// current block number
    pub downloadCurrentBlock: i32,
    /// last block we xmited
    pub downloadXmitBlock: i32,
    /// the buffers for the download blocks
    pub downloadBlocks: [*mut u8; MAX_DOWNLOAD_WINDOW],
    pub downloadBlockSize: [i32; MAX_DOWNLOAD_WINDOW],
    /// We have sent the EOF block
    pub downloadEOF: qboolean,
    /// time we last got an ack from the client
    pub downloadSendTime: i32,

    /// frame last client usercmd message
    pub deltaMessage: i32,
    /// svs.time when another reliable command will be allowed
    pub nextReliableTime: i32,
    /// svs.time when packet was last received
    pub lastPacketTime: i32,
    /// svs.time when connection started
    pub lastConnectTime: i32,
    /// send another snapshot when svs.time >= nextSnapshotTime
    pub nextSnapshotTime: i32,
    /// true if nextSnapshotTime was set based on rate instead of snapshotMsec
    pub rateDelayed: qboolean,
    /// must timeout a few frames in a row so debugging doesn't break
    pub timeoutCount: i32,
    /// updates can be delta'd from here
    pub frames: [clientSnapshot_t; PACKET_BACKUP],
    pub ping: i32,
    /// bytes / second
    pub rate: i32,
    /// requests a snapshot every snapshotMsec unless rate choked
    pub snapshotMsec: i32,
    pub pureAuthentic: i32,
    pub netchan: netchan_t,

    /// if > svs.time && count > x, deny change -rww
    pub lastUserInfoChange: i32,
    /// allow a certain number of changes within a certain time period -rww
    pub lastUserInfoCount: i32,
}

/// Manifest alias: siblings importing the oracle tag name `client_s` resolve
/// to the typedef `client_t`.
pub type client_s = client_t;

impl Default for client_t {
    /// Raven zero-fills `client_t` wholesale (`Z_Malloc` TAG-zeroed /
    /// `Com_Memset`). Every field but the two owned `String`s is POD (no
    /// `Drop`); the all-POD aggregates (`usercmd_t`, `frames`, `netchan_t`) use
    /// per-field `zeroed()`, verified free of `String`/`Vec`/`Drop`; the two
    /// name fields default to empty (`downloadName` empty == "no download").
    fn default() -> Self {
        client_t {
            state: clientState_t::CS_FREE,
            userinfo: [0; MAX_INFO_STRING],
            sentGamedir: 0,
            reliableCommands: [[0; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],
            reliableSequence: 0,
            reliableAcknowledge: 0,
            reliableSent: 0,
            messageAcknowledge: 0,
            gamestateMessageNum: 0,
            challenge: 0,
            lastUsercmd: unsafe { zeroed() },
            lastMessageNum: 0,
            lastClientCommand: 0,
            lastClientCommandString: [0; MAX_STRING_CHARS],
            gentity: null_mut(),
            name: String::new(),
            downloadName: String::new(),
            download: 0,
            downloadSize: 0,
            downloadCount: 0,
            downloadClientBlock: 0,
            downloadCurrentBlock: 0,
            downloadXmitBlock: 0,
            downloadBlocks: [null_mut(); MAX_DOWNLOAD_WINDOW],
            downloadBlockSize: [0; MAX_DOWNLOAD_WINDOW],
            downloadEOF: 0,
            downloadSendTime: 0,
            deltaMessage: 0,
            nextReliableTime: 0,
            lastPacketTime: 0,
            lastConnectTime: 0,
            nextSnapshotTime: 0,
            rateDelayed: 0,
            timeoutCount: 0,
            frames: unsafe { zeroed() },
            ping: 0,
            rate: 0,
            snapshotMsec: 0,
            pureAuthentic: 0,
            netchan: unsafe { zeroed() },
            lastUserInfoChange: 0,
            lastUserInfoCount: 0,
        }
    }
}
