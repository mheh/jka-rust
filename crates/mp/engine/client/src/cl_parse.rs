//! `cl_parse.cpp` — server-to-client message parsing.
//!
//! Source: `oracle/codemp/client/cl_parse.cpp`

use core::ffi::{c_char, c_int, CStr};

use mp_abi::cgame::exports::MpCgameExport;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{Com_DPrintf, Com_Memcpy, Com_Memset};
use mp_engine_qcommon::cvar_fns::{
    Cvar_Set, Cvar_SetCheatState, Cvar_SetValue, Cvar_VariableString, Cvar_VariableValue,
};
use mp_engine_qcommon::files::unz_types::z_stream;
use mp_engine_qcommon::files_common::{
    FS_FCloseFile, FS_PureServerSetLoadedPaks, FS_Write,
};
use mp_engine_qcommon::files_pc::{
    FS_ConditionalRestart, FS_PureServerSetReferencedPaks, FS_SV_FOpenFileWrite, FS_SV_Rename,
    FS_UpdateGamedir,
};
use mp_engine_qcommon::msg::{
    MSG_Bitstream, MSG_CheckNETFPSFOverrides, MSG_ReadBigString, MSG_ReadBits, MSG_ReadByte,
    MSG_ReadData, MSG_ReadDeltaEntity, MSG_ReadDeltaPlayerstate, MSG_ReadLong, MSG_ReadShort,
    MSG_ReadString,
};
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_engine_qcommon::qcommon::net_limits::{
    MAX_MSGLEN, MAX_RELIABLE_COMMANDS, PACKET_BACKUP, PACKET_MASK,
};
use mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::{
    svc_baseline, svc_configstring, svc_gamestate, svc_EOF,
};
use mp_bg::public::configstring::{CS_SERVERINFO, CS_SYSTEMINFO};
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::game_state::{MAX_CONFIGSTRINGS, MAX_GAMESTATE_CHARS};
use mp_qshared::shared::limits::{GENTITYNUM_BITS, MAX_GENTITIES};
use native_string::atoi::atoi;
use native_string::info::Info_ValueForKey;
use native_string::q_string::Q_stricmp;
use native_string::q_strncpyz::Q_strncpyz;
use native_types::{qboolean, qfalse, qtrue, MAX_QPATH};

use crate::cl_console::Con_Close;
use crate::cl_input::CL_WritePacket;
use crate::cl_main::{CL_AddReliableCommand, CL_ClearState, CL_InitDownloads, CL_NextDownload};
use crate::client::cl_snapshot_t::clSnapshot_t;
use crate::client::client_connection_t::MAX_HEIGHTMAP_SIZE;
use crate::client::client_consts::MAX_PARSE_ENTITIES;
use crate::client_host::Client;

/// Raven `SHOWNET` - the debug trace line for one read op at `cl_shownet >= 2`.
///
/// PORT-NOTE(shape): `cl_shownet` and `Com_Printf` need `common: &mut Common`,
/// which this packet's resolved signature never carries. `cl` stands in for
/// `common` at both call sites until integration rewires the receiver.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:41-45`
pub fn SHOWNET(cl: &mut Client, msg: *mut msg_t, s: *mut c_char) {
    //TODO: Port cl_shownet
    if cl.cl_shownet >= 2 {
        let readcount = unsafe { (*msg).readcount - 1 };
        let text = unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned();
        com_printf(cl, &format!("{:3}:{}\n", readcount, text));
    }
}

/// Raven `CL_DeltaEntity` - unpacks one wire entity into the parse-entities ring.
///
/// PORT-NOTE(shape): `MSG_ReadDeltaEntity` needs `view: &mut EngineHostView`,
/// which this packet's resolved signature never carries. `cl` stands in.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:65-87`
pub fn CL_DeltaEntity(
    cl: &mut Client,
    msg: *mut msg_t,
    frame: *mut clSnapshot_t,
    newnum: c_int,
    old: *mut entityState_t,
    unchanged: qboolean,
) {
    // save the parsed entity state into the big circular buffer so
    // it can be used as the source for a later delta
    let index = (cl.cl.parseEntitiesNum & (MAX_PARSE_ENTITIES - 1)) as usize;
    let state: *mut entityState_t = &mut cl.cl.parseEntities[index];

    if unchanged != qfalse {
        unsafe { *state = *old };
    } else {
        MSG_ReadDeltaEntity(cl, msg, old, state, newnum);
    }

    if unsafe { (*state).number } == (MAX_GENTITIES as c_int - 1) {
        return; // entity was delta removed
    }
    cl.cl.parseEntitiesNum += 1;
    unsafe { (*frame).numEntities += 1 };
}

/// Raven `CL_ParseSetGame` - reads the mod dir override the server pushed down.
///
/// PORT-NOTE(shape): the resolved signature carries no state receiver at all,
/// yet every callee needs `common`/`view`. `common`/`view` are referenced
/// directly below and must be threaded in at integration.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:341-373`
pub fn CL_ParseSetGame(msg: *mut msg_t) {
    let mut new_game_dir = [0 as c_char; MAX_QPATH];
    let mut i: usize = 0;

    while i < MAX_QPATH {
        let next = MSG_ReadByte(common, msg) as u8 as c_char;

        if next != 0 {
            // if next is 0 then we have finished reading to the end of the message
            new_game_dir[i] = next;
        } else {
            break;
        }
        i += 1;
    }
    new_game_dir[i] = 0;

    let new_game_dir_str = unsafe { CStr::from_ptr(new_game_dir.as_ptr()) }
        .to_string_lossy()
        .into_owned();

    Cvar_Set(view, "fs_game", &new_game_dir_str);

    // Update the search path for the mod dir
    FS_UpdateGamedir(view);

    // Now update the overrides manually
    MSG_CheckNETFPSFOverrides(view, qfalse);
    MSG_CheckNETFPSFOverrides(view, qtrue);
}

/// Raven `CL_SystemInfoChanged` - re-syncs local cvars from the server's `sv_serverinfo`.
///
/// PORT-NOTE(shape): `Cvar_SetCheatState`/`FS_PureServerSet*`/`Cvar_Set`/
/// `FS_ConditionalRestart` need `view: &mut EngineHostView`, and
/// `Cvar_VariableString`/`Cvar_VariableValue` need `common: &Common`; neither
/// receiver is in this packet's resolved signature. `cl` stands in for both
/// until integration rewires the receivers.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:392-448`
pub fn CL_SystemInfoChanged(cl: &mut Client) {
    //TODO: Port CS_SYSTEMINFO
    let offset = cl.cl.gameState.stringOffsets[CS_SYSTEMINFO as usize] as usize;
    let system_info = unsafe {
        CStr::from_ptr(cl.cl.gameState.stringData.as_ptr().add(offset))
            .to_string_lossy()
            .into_owned()
    };
    cl.cl.serverId = atoi(&Info_ValueForKey(&system_info, "sv_serverid"));

    // don't set any vars when playing a demo
    if cl.clc.demoplaying != qfalse {
        return;
    }

    let s = Info_ValueForKey(&system_info, "sv_cheats");
    if atoi(&s) == 0 {
        Cvar_SetCheatState(cl);
    }

    // check pure server string
    let s = Info_ValueForKey(&system_info, "sv_paks");
    let t = Info_ValueForKey(&system_info, "sv_pakNames");
    FS_PureServerSetLoadedPaks(cl, &s, &t);

    let s = Info_ValueForKey(&system_info, "sv_referencedPaks");
    let t = Info_ValueForKey(&system_info, "sv_referencedPakNames");
    FS_PureServerSetReferencedPaks(cl, &s, &t);

    let mut game_set = qfalse;
    // scan through all the variables in the systeminfo and locally set cvars to match
    let mut cursor: &str = &system_info;
    loop {
        let mut key = String::new();
        let mut value = String::new();
        Info_NextPair(&mut cursor, &mut key, &mut value);
        if key.is_empty() {
            break;
        }
        // ehw!
        if Q_stricmp(&key, "fs_game") == 0 {
            game_set = qtrue;
        }

        Cvar_Set(cl, &key, &value);
    }
    // if game folder should not be set and it is set at the client side
    if game_set == qfalse && !Cvar_VariableString(cl, "fs_game").is_empty() {
        Cvar_Set(cl, "fs_game", "");
    }
    cl.cl_connectedToPureServer = Cvar_VariableValue(cl, "sv_pure") as c_int;

    cl.cl_connectedGAME = atoi(&Info_ValueForKey(&system_info, "vm_game"));
    cl.cl_connectedCGAME = atoi(&Info_ValueForKey(&system_info, "vm_cgame"));
    cl.cl_connectedUI = atoi(&Info_ValueForKey(&system_info, "vm_ui"));
}

/// Raven `CL_ParseAutomapSymbols` - unpacks the RMG automap marker list.
///
/// PORT-NOTE(shape): `MSG_Read*` need `common: &mut Common`, absent from this
/// packet's resolved signature. `cl` stands in.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:450-463`
pub fn CL_ParseAutomapSymbols(cl: &mut Client, msg: *mut msg_t) {
    let count = MSG_ReadShort(cl, msg) as u16 as c_int;
    cl.clc.rmgAutomapSymbolCount = count;

    for i in 0..count {
        let m_type = MSG_ReadByte(cl, msg);
        let m_side = MSG_ReadByte(cl, msg);
        let origin_x = MSG_ReadLong(cl, msg) as f32;
        let origin_y = MSG_ReadLong(cl, msg) as f32;

        let sym = &mut cl.clc.rmgAutomapSymbols[i as usize];
        sym.mType = m_type;
        sym.mSide = m_side;
        sym.mOrigin[0] = origin_x;
        sym.mOrigin[1] = origin_y;
    }
}

/// Raven `CL_GetValueForHidden` - returns the hidden-cvar value the server stashed.
///
/// Raven: string arg here just in case I want to add more sometime and make a lookup table.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:763-766`
pub fn CL_GetValueForHidden(cl: &mut Client, s: *const c_char) -> c_int {
    let _ = s;
    //TODO: Port hiddenCvarVal
    let hidden = unsafe { CStr::from_ptr(cl.hiddenCvarVal.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    atoi(&hidden)
}

/// Raven `CL_ParseCommandString` - stores an incoming reliable server command.
///
/// PORT-NOTE(shape): `MSG_ReadLong`/`MSG_ReadString` need `common: &mut Common`,
/// absent from this packet's resolved signature. `cl` stands in.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:776-846`
pub fn CL_ParseCommandString(cl: &mut Client, msg: *mut msg_t) {
    let seq = MSG_ReadLong(cl, msg);
    let s = MSG_ReadString(cl, msg);

    // see if we have already executed stored it off
    if cl.clc.serverCommandSequence >= seq {
        return;
    }
    cl.clc.serverCommandSequence = seq;

    let index = (seq & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize;

    let destsize = cl.clc.serverCommands[index].len();
    Q_strncpyz(&mut cl.clc.serverCommands[index], &s, destsize);
}

/// Raven `CL_ParsePacketEntities` - deltas the wire entity list against the last frame.
///
/// PORT-NOTE(shape): `MSG_ReadBits`/`Com_Printf` need `common: &mut Common`,
/// absent from this packet's resolved signature. `cl` stands in.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:95-195`
pub fn CL_ParsePacketEntities(
    cl: &mut Client,
    msg: *mut msg_t,
    oldframe: *mut clSnapshot_t,
    newframe: *mut clSnapshot_t,
) {
    unsafe {
        (*newframe).parseEntitiesNum = cl.cl.parseEntitiesNum;
        (*newframe).numEntities = 0;
    }

    // delta from the entities present in oldframe
    let mut oldindex: c_int = 0;
    let mut oldstate: *mut entityState_t = core::ptr::null_mut();
    let mut oldnum: c_int;
    if oldframe.is_null() {
        oldnum = 99999;
    } else if oldindex >= unsafe { (*oldframe).numEntities } {
        oldnum = 99999;
    } else {
        //TODO: Port MAX_PARSE_ENTITIES
        let idx = ((unsafe { (*oldframe).parseEntitiesNum } + oldindex) & (MAX_PARSE_ENTITIES - 1))
            as usize;
        oldstate = &mut cl.cl.parseEntities[idx];
        oldnum = unsafe { (*oldstate).number };
    }

    loop {
        // read the entity index number
        let newnum = MSG_ReadBits(cl, msg, GENTITYNUM_BITS);

        if newnum == (MAX_GENTITIES as c_int - 1) {
            break;
        }

        if unsafe { (*msg).readcount } > unsafe { (*msg).cursize } {
            com_error(
                errorParm_t::ERR_DROP,
                "CL_ParsePacketEntities: end of message".to_string(),
            );
        }

        while oldnum < newnum {
            // one or more entities from the old packet are unchanged
            if cl.cl_shownet == 3 {
                com_printf(
                    cl,
                    &format!(
                        "{:3}:  unchanged: {}\n",
                        unsafe { (*msg).readcount },
                        oldnum
                    ),
                );
            }
            CL_DeltaEntity(cl, msg, newframe, oldnum, oldstate, qtrue);

            oldindex += 1;

            if oldindex >= unsafe { (*oldframe).numEntities } {
                oldnum = 99999;
            } else {
                let idx = ((unsafe { (*oldframe).parseEntitiesNum } + oldindex)
                    & (MAX_PARSE_ENTITIES - 1)) as usize;
                oldstate = &mut cl.cl.parseEntities[idx];
                oldnum = unsafe { (*oldstate).number };
            }
        }
        if oldnum == newnum {
            // delta from previous state
            if cl.cl_shownet == 3 {
                com_printf(
                    cl,
                    &format!("{:3}:  delta: {}\n", unsafe { (*msg).readcount }, newnum),
                );
            }
            CL_DeltaEntity(cl, msg, newframe, newnum, oldstate, qfalse);

            oldindex += 1;

            if oldindex >= unsafe { (*oldframe).numEntities } {
                oldnum = 99999;
            } else {
                let idx = ((unsafe { (*oldframe).parseEntitiesNum } + oldindex)
                    & (MAX_PARSE_ENTITIES - 1)) as usize;
                oldstate = &mut cl.cl.parseEntities[idx];
                oldnum = unsafe { (*oldstate).number };
            }
            continue;
        }

        if oldnum > newnum {
            // delta from baseline
            if cl.cl_shownet == 3 {
                com_printf(
                    cl,
                    &format!("{:3}:  baseline: {}\n", unsafe { (*msg).readcount }, newnum),
                );
            }
            let baseline: *mut entityState_t = &mut cl.cl.entityBaselines[newnum as usize];
            CL_DeltaEntity(cl, msg, newframe, newnum, baseline, qfalse);
            continue;
        }
    }

    // any remaining entities in the old frame are copied over
    while oldnum != 99999 {
        // one or more entities from the old packet are unchanged
        if cl.cl_shownet == 3 {
            com_printf(
                cl,
                &format!(
                    "{:3}:  unchanged: {}\n",
                    unsafe { (*msg).readcount },
                    oldnum
                ),
            );
        }
        CL_DeltaEntity(cl, msg, newframe, oldnum, oldstate, qtrue);

        oldindex += 1;

        if oldindex >= unsafe { (*oldframe).numEntities } {
            oldnum = 99999;
        } else {
            let idx = ((unsafe { (*oldframe).parseEntitiesNum } + oldindex)
                & (MAX_PARSE_ENTITIES - 1)) as usize;
            oldstate = &mut cl.cl.parseEntities[idx];
            oldnum = unsafe { (*oldstate).number };
        }
    }
}

/// Raven `CL_ParseRMG` - unpacks the streamed RMG heightmap/flattenmap blocks.
///
/// PORT-NOTE(shape): `MSG_Read*` need `common: &mut Common`, absent from this
/// packet's resolved signature. `cl` stands in. `inflate`/`inflateInit`/
/// `inflateEnd`/`z_stream` are the vendored zlib seam (plan §1); no matching
/// wrapper exists yet, so the calls below are literal placeholders for the
/// seam module integration must supply.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:465-526`
pub fn CL_ParseRMG(cl: &mut Client, msg: *mut msg_t) {
    cl.clc.rmgHeightMapSize = MSG_ReadShort(cl, msg) as u16 as c_int;
    if cl.clc.rmgHeightMapSize == 0 {
        return;
    }

    let mut zdata: z_stream = unsafe { core::mem::zeroed() };
    let size: c_int;
    // §19: Raven leaves `heightmap1` uninitialized; every read path below
    // fills it from the wire before `inflate` consumes it, so a zero-init
    // here is a safe stand-in for the UB read Raven never actually takes.
    let mut heightmap1 = [0u8; 15000];

    if MSG_ReadBits(cl, msg, 1) != 0 {
        // Read the heightmap
        Com_Memset(
            &mut zdata as *mut z_stream as *mut (),
            0,
            core::mem::size_of::<z_stream>(),
        );
        inflateInit(&mut zdata, Z_SYNC_FLUSH);

        MSG_ReadData(
            cl,
            msg,
            heightmap1.as_mut_ptr() as *mut (),
            cl.clc.rmgHeightMapSize,
        );

        zdata.next_in = heightmap1.as_mut_ptr();
        zdata.avail_in = cl.clc.rmgHeightMapSize as u32;
        zdata.next_out = cl.clc.rmgHeightMap.as_mut_ptr();
        zdata.avail_out = MAX_HEIGHTMAP_SIZE as u32;
        inflate(&mut zdata);

        cl.clc.rmgHeightMapSize = zdata.total_out as c_int;

        inflateEnd(&mut zdata);
    } else {
        MSG_ReadData(
            cl,
            msg,
            cl.clc.rmgHeightMap.as_mut_ptr() as *mut (),
            cl.clc.rmgHeightMapSize,
        );
    }

    size = MSG_ReadShort(cl, msg) as u16 as c_int;

    if MSG_ReadBits(cl, msg, 1) != 0 {
        // Read the flatten map
        Com_Memset(
            &mut zdata as *mut z_stream as *mut (),
            0,
            core::mem::size_of::<z_stream>(),
        );
        inflateInit(&mut zdata, Z_SYNC_FLUSH);

        MSG_ReadData(cl, msg, heightmap1.as_mut_ptr() as *mut (), size);

        zdata.next_in = heightmap1.as_mut_ptr();
        zdata.avail_in = cl.clc.rmgHeightMapSize as u32;
        zdata.next_out = cl.clc.rmgFlattenMap.as_mut_ptr();
        zdata.avail_out = MAX_HEIGHTMAP_SIZE as u32;
        inflate(&mut zdata);
        inflateEnd(&mut zdata);
    } else {
        MSG_ReadData(cl, msg, cl.clc.rmgFlattenMap.as_mut_ptr() as *mut (), size);
    }

    // Read the seed
    cl.clc.rmgSeed = MSG_ReadLong(cl, msg);

    CL_ParseAutomapSymbols(cl, msg);
}

/// Raven `CL_ParseSnapshot` - decompresses one server snapshot into `cl.snap`.
///
/// PORT-NOTE(shape): `MSG_Read*`/`Com_Printf`/`Com_DPrintf`/`Com_Memset` need
/// `common: &mut Common`, absent from this packet's resolved signature. `cl`
/// stands in.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:207-328`
pub fn CL_ParseSnapshot(cl: &mut Client, msg: *mut msg_t) {
    // read in the new snapshot to a temporary buffer
    // we will only copy to cl.snap if it is valid
    let mut new_snap: clSnapshot_t = unsafe { core::mem::zeroed() };
    Com_Memset(
        &mut new_snap as *mut clSnapshot_t as *mut (),
        0,
        core::mem::size_of::<clSnapshot_t>(),
    );

    // we will have read any new server commands in this
    // message before we got to svc_snapshot
    new_snap.serverCommandNum = cl.clc.serverCommandSequence;

    new_snap.serverTime = MSG_ReadLong(cl, msg);

    new_snap.messageNum = cl.clc.serverMessageSequence;

    let delta_num = MSG_ReadByte(cl, msg);
    if delta_num == 0 {
        new_snap.deltaNum = -1;
    } else {
        new_snap.deltaNum = new_snap.messageNum - delta_num;
    }
    new_snap.snapFlags = MSG_ReadByte(cl, msg);

    // If the frame is delta compressed from data that we
    // no longer have available, we must suck up the rest of
    // the frame, but not use it, then ask for a non-compressed
    // message
    let mut old: *mut clSnapshot_t = core::ptr::null_mut();
    if new_snap.deltaNum <= 0 {
        new_snap.valid = qtrue; // uncompressed frame
        cl.clc.demowaiting = qfalse; // we can start recording now
    } else {
        let idx = (new_snap.deltaNum as usize) & PACKET_MASK;
        old = &mut cl.cl.snapshots[idx];
        if unsafe { (*old).valid } == qfalse {
            // should never happen
            com_printf(cl, "Delta from invalid frame (not supposed to happen!).\n");
        } else if unsafe { (*old).messageNum } != new_snap.deltaNum {
            // The frame that the server did the delta from
            // is too old, so we can't reconstruct it properly.
            com_printf(cl, "Delta frame too old.\n");
        } else if cl.cl.parseEntitiesNum - unsafe { (*old).parseEntitiesNum }
            > MAX_PARSE_ENTITIES - 128
        {
            Com_DPrintf(cl, "Delta parseEntitiesNum too old.\n");
        } else {
            new_snap.valid = qtrue; // valid delta parse
        }
    }

    // read areamask
    let len = MSG_ReadByte(cl, msg);
    MSG_ReadData(cl, msg, new_snap.areamask.as_mut_ptr() as *mut (), len);

    // read playerinfo
    SHOWNET(cl, msg, c"playerstate".as_ptr() as *mut c_char);
    if !old.is_null() {
        MSG_ReadDeltaPlayerstate(cl, msg, &mut unsafe { (*old).ps }, &mut new_snap.ps);
        if new_snap.ps.m_iVehicleNum != 0 {
            // this means we must have written our vehicle's ps too
            MSG_ReadDeltaPlayerstate(
                cl,
                msg,
                &mut unsafe { (*old).vps },
                &mut new_snap.vps,
                qtrue,
            );
        }
    } else {
        MSG_ReadDeltaPlayerstate(cl, msg, core::ptr::null_mut(), &mut new_snap.ps);
        if new_snap.ps.m_iVehicleNum != 0 {
            // this means we must have written our vehicle's ps too
            MSG_ReadDeltaPlayerstate(cl, msg, core::ptr::null_mut(), &mut new_snap.vps, qtrue);
        }
    }

    // read packet entities
    SHOWNET(cl, msg, c"packet entities".as_ptr() as *mut c_char);
    CL_ParsePacketEntities(cl, msg, old, &mut new_snap);

    // if not valid, dump the entire thing now that it has
    // been properly read
    if new_snap.valid == qfalse {
        return;
    }

    // clear the valid flags of any snapshots between the last
    // received and this one, so if there was a dropped packet
    // it won't look like something valid to delta from next
    // time we wrap around in the buffer
    let mut old_message_num = cl.cl.snap.messageNum + 1;

    if new_snap.messageNum - old_message_num >= PACKET_BACKUP as c_int {
        old_message_num = new_snap.messageNum - (PACKET_BACKUP as c_int - 1);
    }
    while old_message_num < new_snap.messageNum {
        cl.cl.snapshots[(old_message_num as usize) & PACKET_MASK].valid = qfalse;
        old_message_num += 1;
    }

    // copy to the current good spot
    cl.cl.snap = new_snap;
    cl.cl.snap.ping = 999;
    // calculate ping time
    for i in 0..PACKET_BACKUP as c_int {
        let packet_num = ((cl.clc.netchan.outgoingSequence - 1 - i) as usize) & PACKET_MASK;
        if cl.cl.snap.ps.commandTime >= cl.cl.outPackets[packet_num].p_serverTime {
            cl.cl.snap.ping = cl.cls.realtime - cl.cl.outPackets[packet_num].p_realtime;
            break;
        }
    }
    // save the frame off in the backup array for later delta comparisons
    let idx = (cl.cl.snap.messageNum as usize) & PACKET_MASK;
    cl.cl.snapshots[idx] = cl.cl.snap;

    if cl.cl_shownet == 3 {
        com_printf(
            cl,
            &format!(
                "   snapshot:{}  delta:{}  ping:{}\n",
                cl.cl.snap.messageNum, cl.cl.snap.deltaNum, cl.cl.snap.ping
            ),
        );
    }

    cl.cl.newSnapshots = qtrue;
}

/// Raven `CL_ParseDownload` - writes one incoming download block to disk.
///
/// PORT-NOTE(shape): `Cvar_Set`/`Cvar_SetValue` need `view: &mut EngineHostView`,
/// absent from this packet's resolved signature. `common` stands in.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:674-761`
pub fn CL_ParseDownload(common: &mut Common, cl: &mut Client, msg: *mut msg_t) {
    let mut data = [0u8; MAX_MSGLEN];

    // read the data
    let block = MSG_ReadShort(common, msg) as u16 as c_int;

    if block == 0 {
        // block zero is special, contains file size
        cl.clc.downloadSize = MSG_ReadLong(common, msg);

        Cvar_SetValue(common, "cl_downloadSize", cl.clc.downloadSize as f32);

        if cl.clc.downloadSize < 0 {
            com_error(errorParm_t::ERR_DROP, MSG_ReadString(common, msg));
            return;
        }
    }

    let size = MSG_ReadShort(common, msg) as u16 as c_int;
    if size > 0 {
        MSG_ReadData(common, msg, data.as_mut_ptr() as *mut (), size);
    }

    if cl.clc.downloadBlock != block {
        Com_DPrintf(
            common,
            &format!(
                "CL_ParseDownload: Expected block {}, got {}\n",
                cl.clc.downloadBlock, block
            ),
        );
        return;
    }

    // open the file if not opened yet
    if cl.clc.download == 0 {
        let temp_name = unsafe { CStr::from_ptr(cl.clc.downloadTempName.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        if temp_name.is_empty() {
            com_printf(
                common,
                "Server sending download, but no download was requested\n",
            );
            CL_AddReliableCommand(cl, "stopdl");
            return;
        }

        cl.clc.download = FS_SV_FOpenFileWrite(common, &temp_name);

        if cl.clc.download == 0 {
            com_printf(common, &format!("Could not create {}\n", temp_name));
            CL_AddReliableCommand(cl, "stopdl");
            CL_NextDownload(common, cl);
            return;
        }
    }

    if size != 0 {
        FS_Write(common, data.as_ptr() as *const (), size, cl.clc.download);
    }

    CL_AddReliableCommand(cl, &format!("nextdl {}", cl.clc.downloadBlock));
    cl.clc.downloadBlock += 1;

    cl.clc.downloadCount += size;

    // So UI gets access to it
    Cvar_SetValue(common, "cl_downloadCount", cl.clc.downloadCount as f32);

    if size == 0 {
        // A zero length block means EOF
        if cl.clc.download != 0 {
            FS_FCloseFile(common, cl.clc.download);
            cl.clc.download = 0;

            // rename the file
            let temp_name = unsafe { CStr::from_ptr(cl.clc.downloadTempName.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            let download_name = unsafe { CStr::from_ptr(cl.clc.downloadName.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            FS_SV_Rename(common, &temp_name, &download_name);
        }
        cl.clc.downloadTempName[0] = 0;
        cl.clc.downloadName[0] = 0;
        Cvar_Set(common, "cl_downloadName", "");

        // send intentions now
        // We need this because without it, we would hold the last nextdl and then start
        // loading right away.  If we take a while to load, the server is happily trying
        // to send us that last block over and over.
        // Write it twice to help make sure we acknowledge the download
        CL_WritePacket(cl);
        CL_WritePacket(cl);

        // get another file if needed
        CL_NextDownload(common, cl);
    }
}

/// Raven `CL_ParseGamestate` - parses a full gamestate message (configstrings + baselines).
///
/// PORT-NOTE(shape): `Cvar_Set`/`FS_ConditionalRestart` need
/// `view: &mut EngineHostView`, absent from this packet's resolved signature.
/// `common` stands in.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:533-662`
pub fn CL_ParseGamestate(common: &mut Common, cl: &mut Client, msg: *mut msg_t) {
    Con_Close(common, cl);

    cl.clc.connectPacketCount = 0;

    // wipe local client state
    CL_ClearState(cl);

    // a gamestate always marks a server command sequence
    cl.clc.serverCommandSequence = MSG_ReadLong(common, msg);

    // parse all the configstrings and baselines
    cl.cl.gameState.dataCount = 1; // leave a 0 at the beginning for uninitialized configstrings
    loop {
        let cmd = MSG_ReadByte(common, msg);

        if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_EOF as c_int {
            break;
        }

        if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_configstring as c_int {
            let start = unsafe { (*msg).readcount };

            let i = MSG_ReadShort(common, msg);
            if i < 0 || i >= MAX_CONFIGSTRINGS as c_int {
                com_error(
                    errorParm_t::ERR_DROP,
                    "configstring > MAX_CONFIGSTRINGS".to_string(),
                );
            }
            let s = MSG_ReadBigString(common, msg);

            if cl.cl_shownet >= 2 {
                com_printf(common, &format!("{:3}: {}: {}\n", start, i, s));
            }

            let len = s.len();

            if len + 1 + cl.cl.gameState.dataCount as usize > MAX_GAMESTATE_CHARS {
                com_error(
                    errorParm_t::ERR_DROP,
                    "MAX_GAMESTATE_CHARS exceeded".to_string(),
                );
            }

            // append it to the gameState string buffer
            cl.cl.gameState.stringOffsets[i as usize] = cl.cl.gameState.dataCount;
            let data_count = cl.cl.gameState.dataCount as usize;
            Com_Memcpy(
                unsafe { cl.cl.gameState.stringData.as_mut_ptr().add(data_count) as *mut () },
                s.as_ptr() as *const (),
                len + 1,
            );
            cl.cl.gameState.dataCount += (len + 1) as c_int;
        } else if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_baseline as c_int {
            let newnum = MSG_ReadBits(common, msg, GENTITYNUM_BITS);
            if newnum < 0 || newnum >= MAX_GENTITIES as c_int {
                com_error(
                    errorParm_t::ERR_DROP,
                    format!("Baseline number out of range: {}", newnum),
                );
            }
            let mut nullstate: entityState_t = unsafe { core::mem::zeroed() };
            Com_Memset(
                &mut nullstate as *mut entityState_t as *mut (),
                0,
                core::mem::size_of::<entityState_t>(),
            );
            let es: *mut entityState_t = &mut cl.cl.entityBaselines[newnum as usize];
            MSG_ReadDeltaEntity(cl, msg, &mut nullstate, es, newnum);
        } else {
            com_error(
                errorParm_t::ERR_DROP,
                "CL_ParseGamestate: bad command byte".to_string(),
            );
        }
    }

    cl.clc.clientNum = MSG_ReadLong(common, msg);
    // read the checksum feed
    cl.clc.checksumFeed = MSG_ReadLong(common, msg);

    CL_ParseRMG(cl, msg); // rwwRMG - get info for it from the server

    // parse serverId and other cvars
    CL_SystemInfoChanged(cl);

    // reinitialize the filesystem if the game directory has changed
    if FS_ConditionalRestart(common, cl.clc.checksumFeed) != qfalse {
        // don't set to true because we yet have to start downloading
        // enabling this can cause double loading of a map when connecting to
        // a server which has a different game directory set
        //clc.downloadRestart = qtrue;
    }

    // This used to call CL_StartHunkUsers, but now we enter the download state before loading the
    // cgame
    CL_InitDownloads(common, cl);

    // make sure the game starts
    Cvar_Set(common, "cl_paused", "0");
}

/// Raven `CL_ParseServerMessage` - the top-level per-message server command dispatch.
///
/// PORT-NOTE(shape): `SHOWNET` needs `common: &mut Common` internally (see its
/// own note); the call here passes `cl` only, matching its resolved signature.
///
/// Source: `oracle/codemp/client/cl_parse.cpp:854-1029`
pub fn CL_ParseServerMessage(common: &mut Common, cl: &mut Client, msg: *mut msg_t) {
    if cl.cl_shownet == 1 {
        com_printf(common, &format!("{} ", unsafe { (*msg).cursize }));
    } else if cl.cl_shownet >= 2 {
        com_printf(common, "------------------\n");
    }

    MSG_Bitstream(msg);

    // get the reliable sequence acknowledge number
    cl.clc.reliableAcknowledge = MSG_ReadLong(common, msg);
    //
    if cl.clc.reliableAcknowledge < cl.clc.reliableSequence - MAX_RELIABLE_COMMANDS as c_int {
        cl.clc.reliableAcknowledge = cl.clc.reliableSequence;
    }

    //
    // parse the message
    //
    loop {
        if unsafe { (*msg).readcount } > unsafe { (*msg).cursize } {
            com_error(
                errorParm_t::ERR_DROP,
                "CL_ParseServerMessage: read past end of server message".to_string(),
            );
            break;
        }

        let cmd = MSG_ReadByte(common, msg);

        if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_EOF as c_int {
            SHOWNET(cl, msg, c"END OF MESSAGE".as_ptr() as *mut c_char);
            break;
        }

        if cl.cl_shownet >= 2 {
            //TODO: Port svc_strings
            if svc_strings[cmd as usize].is_empty() {
                com_printf(
                    common,
                    &format!("{:3}:BAD CMD {}\n", unsafe { (*msg).readcount - 1 }, cmd),
                );
            } else {
                SHOWNET(cl, msg, svc_strings[cmd as usize].as_ptr() as *mut c_char);
            }
        }

        // other commands
        if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_nop as c_int {
            // no-op
        } else if cmd
            == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_serverCommand as c_int
        {
            CL_ParseCommandString(cl, msg);
        } else if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_gamestate as c_int {
            CL_ParseGamestate(common, cl, msg);
        } else if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_snapshot as c_int {
            CL_ParseSnapshot(cl, msg);
        } else if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_setgame as c_int {
            CL_ParseSetGame(msg);
        } else if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_download as c_int {
            CL_ParseDownload(common, cl, msg);
        } else if cmd == mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_mapchange as c_int {
            if !cl.cgvm.is_null() {
                VM_Call(common, cl.cgvm, MpCgameExport::CG_MAP_CHANGE as c_int, &[]);
            }
        } else {
            com_error(
                errorParm_t::ERR_DROP,
                "CL_ParseServerMessage: Illegible server message\n".to_string(),
            );
            break;
        }
    }
}
