#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::game_state::gameState_t;
use mp_qshared::shared::{qboolean, vec3_t, MAX_QPATH};

use super::cl_snapshot_t::clSnapshot_t;
use super::out_packet_t::outPacket_t;

/// Raven `clientActive_t` — the client's active game state (parsed from the
/// server, plus cgame-communicated values); reset on every level change.
///
/// Type definition source: `oracle/codemp/client/client.h:75-137`
#[repr(C)]
pub struct clientActive_t {
    /// it requres several frames in a timeout condition
    /// to disconnect, preventing debugging breaks from
    /// causing immediate disconnects on continue
    pub timeoutcount: i32,
    /// latest received from server
    pub snap: clSnapshot_t,

    /// may be paused during play
    pub serverTime: i32,
    /// to prevent time from flowing bakcwards
    pub oldServerTime: i32,
    /// to check tournament restarts
    pub oldFrameServerTime: i32,
    /// cl.serverTime = cls.realtime + cl.serverTimeDelta
    /// this value changes as net lag varies
    pub serverTimeDelta: i32,
    /// set if any cgame frame has been forced to extrapolate
    /// cleared when CL_AdjustTimeDelta looks at it
    pub extrapolatedSnapshot: qboolean,
    /// set on parse of any valid packet
    pub newSnapshots: qboolean,

    /// configstrings
    pub gameState: gameState_t,
    /// extracted from CS_SERVERINFO
    pub mapname: [i8; MAX_QPATH],

    /// index (not anded off) into cl_parse_entities[]
    pub parseEntitiesNum: i32,

    /// added to by mouse events
    pub mouseDx: [i32; 2],
    pub mouseDy: [i32; 2],
    pub mouseIndex: i32,
    /// set by joystick events
    pub joystickAxis: [i32; 6],

    // cgame communicates a few values to the client system
    /// current weapon to add to usercmd_t
    pub cgameUserCmdValue: i32,
    pub cgameViewAngleForce: vec3_t,
    pub cgameViewAngleForceTime: i32,
    pub cgameSensitivity: f32,

    pub cgameForceSelection: i32,
    pub cgameInvenSelection: i32,

    pub gcmdSendValue: qboolean,
    pub gcmdSentValue: qboolean,
    pub gcmdValue: u8,

    /// cmds[cmdNumber] is the predicted command, [cmdNumber-1] is the last
    /// properly generated command
    /// each mesage will send several old cmds
    pub cmds: [usercmd_t; 64],
    /// incremented each frame, because multiple
    /// frames may need to be packed into a single packet
    pub cmdNumber: i32,

    /// information about each packet we have sent out
    pub outPackets: [outPacket_t; 32],

    /// the client maintains its own idea of view angles, which are
    /// sent to the server each frame.  It is cleared to 0 upon entering each level.
    /// the server sends a delta each frame which is added to the locally
    /// tracked view angles to account for standing on rotating objects,
    /// and teleport direction changes
    pub viewangles: vec3_t,

    /// included in each client message so the server
    /// can tell if it is for a prior map_restart
    pub serverId: i32,
    // big stuff at end of structure so most offsets are 15 bits or less
    pub snapshots: [clSnapshot_t; 32],

    /// for delta compression when not in previous frame
    pub entityBaselines: [entityState_t; 1024],

    pub parseEntities: [entityState_t; 2048],

    pub mSharedMemory: *mut i8,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<clientActive_t>() == 1764304);
const _: () = assert!(core::mem::offset_of!(clientActive_t, timeoutcount) == 0);
const _: () = assert!(core::mem::offset_of!(clientActive_t, snap) == 4);
const _: () = assert!(core::mem::offset_of!(clientActive_t, serverTime) == 3180);
const _: () = assert!(core::mem::offset_of!(clientActive_t, oldServerTime) == 3184);
const _: () = assert!(core::mem::offset_of!(clientActive_t, oldFrameServerTime) == 3188);
const _: () = assert!(core::mem::offset_of!(clientActive_t, serverTimeDelta) == 3192);
const _: () = assert!(core::mem::offset_of!(clientActive_t, extrapolatedSnapshot) == 3196);
const _: () = assert!(core::mem::offset_of!(clientActive_t, newSnapshots) == 3200);
const _: () = assert!(core::mem::offset_of!(clientActive_t, gameState) == 3204);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mapname) == 26008);
const _: () = assert!(core::mem::offset_of!(clientActive_t, parseEntitiesNum) == 26072);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mouseDx) == 26076);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mouseDy) == 26084);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mouseIndex) == 26092);
const _: () = assert!(core::mem::offset_of!(clientActive_t, joystickAxis) == 26096);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameUserCmdValue) == 26120);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameViewAngleForce) == 26124);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameViewAngleForceTime) == 26136);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameSensitivity) == 26140);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameForceSelection) == 26144);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameInvenSelection) == 26148);
const _: () = assert!(core::mem::offset_of!(clientActive_t, gcmdSendValue) == 26152);
const _: () = assert!(core::mem::offset_of!(clientActive_t, gcmdSentValue) == 26156);
const _: () = assert!(core::mem::offset_of!(clientActive_t, gcmdValue) == 26160);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cmds) == 26164);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cmdNumber) == 27956);
const _: () = assert!(core::mem::offset_of!(clientActive_t, outPackets) == 27960);
const _: () = assert!(core::mem::offset_of!(clientActive_t, viewangles) == 28344);
const _: () = assert!(core::mem::offset_of!(clientActive_t, serverId) == 28356);
const _: () = assert!(core::mem::offset_of!(clientActive_t, snapshots) == 28360);
const _: () = assert!(core::mem::offset_of!(clientActive_t, entityBaselines) == 129992);
const _: () = assert!(core::mem::offset_of!(clientActive_t, parseEntities) == 674760);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientActive_t, mSharedMemory) == 1764296);
