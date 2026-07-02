#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::qcommon::entity_state::entityState_t;
use sp_qshared::common::sp::qcommon::usercmd::usercmd_t;
use sp_qshared::shared::{gameState_t, qboolean, vec3_t, MAX_QPATH};

use super::cl_snapshot_t::clSnapshot_t;

/// Raven `clientActive_t` — the client's active game state (parsed from the
/// server, plus cgame-communicated values); reset on every level change.
///
/// Type definition source: `oracle/oracle/code/client/client.h:53-110`
#[repr(C)]
pub struct clientActive_t {
    pub timeoutcount: i32,

    /// latest received from server
    pub frame: clSnapshot_t,

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
    /// set on parse, cleared when CL_AdjustTimeDelta looks at it
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

    /// current weapon to add to usercmd_t
    pub cgameUserCmdValue: i32,
    pub cgameSensitivity: f32,

    /// cmds[cmdNumber] is the predicted command, [cmdNumber-1] is the last
    /// properly generated command
    /// each mesage will send several old cmds
    pub cmds: [usercmd_t; 64],
    /// incremented each frame, because multiple
    /// frames may need to be packed into a single packet
    pub cmdNumber: i32,

    /// cls.realtime sent, for calculating pings
    pub packetTime: [i32; 16],
    /// cmdNumber when packet was sent
    pub packetCmdNumber: [i32; 16],

    /// the client maintains its own idea of view angles, which are
    /// sent to the server each frame.  It is cleared to 0 upon entering each level.
    /// the server sends a delta each frame which is added to the locally
    /// tracked view angles to account for standing on rotating objects,
    /// and teleport direction changes
    pub viewangles: vec3_t,

    /// these are just parsed out of the configstrings for convenience
    pub serverId: i32,

    /// cls.realtime for first cinematic frame (FIXME: NO LONGER USED!, but I wasn't sure if I could remove it because of struct sizes assumed elsewhere? -Ste)
    pub cinematictime: i32,

    /// big stuff at end of structure so most offsets are 15 bits or less
    pub frames: [clSnapshot_t; 16],

    pub parseEntities: [entityState_t; 512],

    /// DJC added - making force powers in single player work like those in
    /// multiplayer.  This makes hot swapping code more portable.
    pub gcmdSendValue: qboolean,
    pub gcmdValue: u8,
}

const _: () = assert!(core::mem::size_of::<clientActive_t>() == 248800);
const _: () = assert!(core::mem::offset_of!(clientActive_t, timeoutcount) == 0);
const _: () = assert!(core::mem::offset_of!(clientActive_t, frame) == 8);
const _: () = assert!(core::mem::offset_of!(clientActive_t, serverTime) == 5080);
const _: () = assert!(core::mem::offset_of!(clientActive_t, oldServerTime) == 5084);
const _: () = assert!(core::mem::offset_of!(clientActive_t, oldFrameServerTime) == 5088);
const _: () = assert!(core::mem::offset_of!(clientActive_t, serverTimeDelta) == 5092);
const _: () = assert!(core::mem::offset_of!(clientActive_t, extrapolatedSnapshot) == 5096);
const _: () = assert!(core::mem::offset_of!(clientActive_t, newSnapshots) == 5100);
const _: () = assert!(core::mem::offset_of!(clientActive_t, gameState) == 5104);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mapname) == 26308);
const _: () = assert!(core::mem::offset_of!(clientActive_t, parseEntitiesNum) == 26372);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mouseDx) == 26376);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mouseDy) == 26384);
const _: () = assert!(core::mem::offset_of!(clientActive_t, mouseIndex) == 26392);
const _: () = assert!(core::mem::offset_of!(clientActive_t, joystickAxis) == 26396);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameUserCmdValue) == 26420);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cgameSensitivity) == 26424);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cmds) == 26428);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cmdNumber) == 28220);
const _: () = assert!(core::mem::offset_of!(clientActive_t, packetTime) == 28224);
const _: () = assert!(core::mem::offset_of!(clientActive_t, packetCmdNumber) == 28288);
const _: () = assert!(core::mem::offset_of!(clientActive_t, viewangles) == 28352);
const _: () = assert!(core::mem::offset_of!(clientActive_t, serverId) == 28364);
const _: () = assert!(core::mem::offset_of!(clientActive_t, cinematictime) == 28368);
const _: () = assert!(core::mem::offset_of!(clientActive_t, frames) == 28376);
const _: () = assert!(core::mem::offset_of!(clientActive_t, parseEntities) == 109528);
const _: () = assert!(core::mem::offset_of!(clientActive_t, gcmdSendValue) == 248792);
const _: () = assert!(core::mem::offset_of!(clientActive_t, gcmdValue) == 248796);
