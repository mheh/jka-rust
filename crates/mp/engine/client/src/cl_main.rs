//! Raven `cl_main.cpp` — the client connection spine, demo record/playback, the
//! server browser and ping lists, and the per-frame client tick.
//!
//! Source: `oracle/codemp/client/cl_main.cpp`

#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_mut,
    unused_assignments,
    unused_imports,
    unused_unsafe
)]

use std::ffi::{CStr, CString};
use std::mem::take;
use std::os::raw::{c_char, c_int};
use std::sync::Arc;

use native_platform::sys_main::{Sys_CheckCD, Sys_MonkeyShouldBeSpanked};
use native_string::atoi::atoi;
use native_string::info::{Info_SetValueForKey, Info_ValueForKey};
use native_string::q_string::{Q_strcat, Q_stricmp, Q_strncmp};
use native_string::q_strncpyz::Q_strncpyz;
use native_types::{byte, qboolean, qfalse, qtrue, word, MAX_QPATH};

use mp_abi::ui::exports::MpUiExport;
use mp_abi::ui::public::ui_menu_command_t::{UIMENU_MAIN, UIMENU_NONE};
use mp_engine_qcommon::cm_load::CM_ClearMap;
use mp_engine_qcommon::cmd_common::{
    Cbuf_AddText, Cbuf_Execute, Cbuf_ExecuteText, Cmd_Argc, Cmd_Args, Cmd_Argv, Cmd_TokenizeString,
};
use mp_engine_qcommon::cmd_pc::{Cmd_AddCommand, Cmd_RemoveCommand};
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{
    Com_DPrintf, Com_EventLoop, Com_Memset, Com_Milliseconds, Info_Print,
};
use mp_engine_qcommon::cvar_fns::{
    Cvar_Get, Cvar_InfoString, Cvar_Set, Cvar_SetValue, Cvar_VariableIntegerValue,
    Cvar_VariableString, Cvar_VariableStringBuffer, Cvar_VariableValue,
};
use mp_engine_qcommon::files_common::{
    FS_FCloseFile, FS_FOpenFileRead, FS_FOpenFileWrite, FS_Read, FS_ReadFile, FS_Restart, FS_Write,
};
use mp_engine_qcommon::files_pc::{
    FS_ClearPakReferences, FS_ComparePaks, FS_ConditionalRestart, FS_LoadedPakNames,
    FS_ReferencedPakNames, FS_ReferencedPakPureChecksums,
};
use mp_engine_qcommon::msg::{
    MSG_BeginReadingOOB, MSG_Bitstream, MSG_Init, MSG_ReadLong, MSG_ReadString, MSG_ReadStringLine,
    MSG_WriteBigString, MSG_WriteBits, MSG_WriteByte, MSG_WriteData, MSG_WriteDeltaEntity,
    MSG_WriteLong, MSG_WriteShort,
};
use mp_engine_qcommon::net_chan::{
    NET_AdrToString, NET_CompareAdr, NET_CompareBaseAdr, NET_IsLocalAddress, NET_OutOfBandData,
    NET_OutOfBandPrint, NET_SendPacket, NET_StringToAdr, Netchan_Setup,
};
use mp_engine_qcommon::qcommon::filesystem_limits::{FS_CGAME_REF, FS_UI_REF};
use mp_engine_qcommon::qcommon::netchan_t::netchan_t;
use mp_engine_qcommon::qcommon::net_limits::{MAX_MSGLEN, MAX_RELIABLE_COMMANDS};
use mp_engine_qcommon::qcommon::protocol::{
    MASTER_SERVER_NAME, NUM_SERVER_PORTS, PORT_MASTER, PORT_SERVER, PORT_UPDATE, PROTOCOL_VERSION,
    UPDATE_SERVER_NAME,
};
use mp_engine_qcommon::stringed::api::{se_check_for_language_updates, SE_GetString};
use mp_engine_qcommon::sys_net::Sys_ShowIP;
use mp_engine_qcommon::timing::sys_milliseconds;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_engine_qcommon::z_memman_pc::{Hunk_Clear, Hunk_ClearToMark};
use mp_engine_server::sv_init::SV_Shutdown;
use mp_engine_server::sv_main::SV_Frame;
use mp_game::prelude::byte as game_byte;
use mp_game::q_shared_cvar_flags::{
    CVAR_ARCHIVE, CVAR_ROM, CVAR_SERVERINFO, CVAR_TEMP, CVAR_USERINFO,
};
use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t;
use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::connstate::connstate_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::game_state::MAX_CONFIGSTRINGS;
use mp_qshared::shared::limits::{
    MAX_GENTITIES, MAX_INFO_STRING, MAX_INFO_VALUE, MAX_NAME_LENGTH, MAX_STRING_CHARS,
};
use mp_qshared::shared::print_parm::printParm_t;
use mp_qshared::shared::q_string::Com_sprintf;
use mp_qshared::shared::char_sizes::SMALLCHAR_WIDTH;
use mp_qshared::shared::keycatch::KEYCATCH_UI;
use mp_qshared::shared::limits::{MAX_PINGREQUESTS, MAX_SERVERSTATUSREQUESTS};
use mp_qshared::shared::server_address::{AS_FAVORITES, AS_GLOBAL, AS_LOCAL, AS_MPLAYER};
use mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::{
    svc_baseline, svc_configstring, svc_gamestate, svc_EOF,
};
use mp_engine_icarus::q3_interface::{S_COLOR_RED, S_COLOR_YELLOW};
use mp_renderer::hook_install::{re_from_view, rm_from_view};
use mp_renderer::render_state::placeholders::GlConfig;
use mp_renderer::tr_init::RE_Shutdown;
use mp_renderer::tr_model::frontend::RE_BeginRegistration;
use mp_renderer::tr_shader::{RE_RegisterShader, RE_RegisterShaderNoMip};

use crate::client::client_consts::RETRANSMIT_TIMEOUT;
use crate::cl_cgame::{CL_InitCGame, CL_SetCGameTime, CL_ShutdownCGame};
use crate::cl_cin::{CL_PlayCinematic_f, SCR_RunCinematic, SCR_StopCinematic};
use crate::cl_console::{Con_Close, Con_Init, Con_RunConsole};
use crate::cl_input::{CL_InitInput, CL_SendCmd, CL_WritePacket};
use crate::cl_net_chan::CL_Netchan_Process;
use crate::cl_parse::CL_ParseServerMessage;
use crate::cl_referee::ref_headless;
use crate::cl_scrn::{SCR_DebugGraph, SCR_Init, SCR_UpdateScreen};
use crate::cl_ui::{CL_InitUI, CL_ShutdownUI};
use crate::client::server_status_t::serverStatus_t;
use crate::client::cl_main_consts::{
    G2_VERT_SPACE_CLIENT_SIZE, MAXPRINTMSG, MAX_SERVERSPERPACKET, MAX_STRINGED_SV_STRING,
    MODEL_CHANGE_DELAY,
};
use crate::client::client_connection_t::MAX_OSPATH;
use crate::client::client_static_t::{MAX_GLOBAL_SERVERS, MAX_OTHER_SERVERS};
use crate::client::ping_t::ping_t;
use crate::client::server_address_t::serverAddress_t;
use crate::client::server_info_t::serverInfo_t;
use crate::client_host::{cl_from_view, Client};
use crate::client_host::snd_from_view;
use crate::snd_dma::{
    S_BeginRegistration, S_ClearSoundBuffer, S_DisableSounds, S_Init, S_Shutdown, S_StopAllSounds,
    S_Update,
};
use crate::snd_dma::S_RestartMusic;

// PORT-NOTE(latin1-scratch): Raven passes `char[]` scratch buffers straight into
// `strlen`/`strcmp`/printf. The ported callees take `&str`, so each site reads
// the NUL-terminated Latin-1 bytes out of the array into an owned `String`
// (#13 string campaign). No behavior changes, only the representation.

/// `CL_AddReliableCommand` — queues a reliable command for the server.
///
/// Raven: if we would be losing an old command that has not been acknowledged,
/// we must drop the connection.
/// Source: `oracle/codemp/client/cl_main.cpp:156-167`
pub fn CL_AddReliableCommand(cl: &mut Client, cmd: *const c_char) {
    let index: c_int;

    // if we would be losing an old command that hasn't been acknowledged,
    // we must drop the connection
    if cl.clc.reliableSequence - cl.clc.reliableAcknowledge > MAX_RELIABLE_COMMANDS as c_int {
        com_error(errorParm_t::ERR_DROP, "Client command overflow".to_string());
    }
    cl.clc.reliableSequence += 1;
    index = cl.clc.reliableSequence & (MAX_RELIABLE_COMMANDS as c_int - 1);
    let src: String = unsafe { CStr::from_ptr(cmd) }
        .to_string_lossy()
        .into_owned();
    let destsize = cl.clc.reliableCommands[index as usize].len();
    Q_strncpyz(&mut cl.clc.reliableCommands[index as usize], &src, destsize);
}

/// `CL_ChangeReliableCommand` — corrupts the newest reliable command on purpose.
///
/// Raven: the monkey test appends a newline to the pending command.
/// Source: `oracle/codemp/client/cl_main.cpp:174-185`
pub fn CL_ChangeReliableCommand(common: &mut Common, cl: &mut Client) {
    let r: c_int;
    let index: c_int;
    let mut l: c_int;

    // Raven's `random()` macro is `(rand() & 0x7fff) / (float)0x7fff`; the int
    // cast then truncates it to 0, so `r` is dead. The draw still happens.
    let draw = common.qrand.rand();
    r = cl.clc.reliableSequence - (((draw & 0x7fff) as f32 / 0x7fff as f32) as c_int * 5);
    index = cl.clc.reliableSequence & (MAX_RELIABLE_COMMANDS as c_int - 1);
    l = cl.clc.reliableCommands[index as usize]
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(MAX_STRING_CHARS as usize) as c_int;
    if l >= MAX_STRING_CHARS as c_int - 1 {
        l = MAX_STRING_CHARS as c_int - 2;
    }
    cl.clc.reliableCommands[index as usize][l as usize] = b'\n' as c_char;
    cl.clc.reliableCommands[index as usize][(l + 1) as usize] = 0;
}

/// `CL_WriteDemoMessage` — appends one server message to the open demo file.
///
/// Raven: the packet sequencing information is skipped by `headerBytes`.
/// Source: `oracle/codemp/client/cl_main.cpp:218-231`
pub fn CL_WriteDemoMessage(
    common: &mut Common,
    cl: &mut Client,
    msg: *mut msg_t,
    headerBytes: c_int,
) {
    let mut len: c_int;
    let mut swlen: c_int;

    // write the packet sequence
    len = cl.clc.serverMessageSequence;
    swlen = len.to_le();
    FS_Write(
        common,
        &swlen as *const c_int as *const (),
        4,
        cl.clc.demofile,
    );

    // skip the packet sequencing information
    len = unsafe { (*msg).cursize } - headerBytes;
    swlen = len.to_le();
    FS_Write(
        common,
        &swlen as *const c_int as *const (),
        4,
        cl.clc.demofile,
    );
    let body = unsafe { (*msg).data.offset(headerBytes as isize) };
    FS_Write(common, body as *const (), len, cl.clc.demofile);
}

/// `CL_StopRecord_f` — closes the demo file and stops recording.
///
/// Source: `oracle/codemp/client/cl_main.cpp:241-258`
pub fn CL_StopRecord_f(common: &mut Common, cl: &mut Client) {
    let len: c_int;

    if cl.clc.demorecording == qfalse {
        com_printf(common, "Not recording a demo.\n");
        return;
    }

    // finish up
    len = -1;
    FS_Write(
        common,
        &len as *const c_int as *const (),
        4,
        cl.clc.demofile,
    );
    FS_Write(
        common,
        &len as *const c_int as *const (),
        4,
        cl.clc.demofile,
    );
    FS_FCloseFile(common, cl.clc.demofile);
    cl.clc.demofile = 0;
    cl.clc.demorecording = qfalse;
    cl.clc.spDemoRecording = qfalse;
    com_printf(common, "Stopped demo.\n");
}

/// `CL_DemoFilename` — builds the auto-numbered demo file name.
///
/// Raven: numbers outside 0..9999 fall back to the `demo9999.tga` name.
/// Source: `oracle/codemp/client/cl_main.cpp:265-283`
pub fn CL_DemoFilename(number: c_int, fileName: *mut c_char) {
    let mut number = number;
    let a: c_int;
    let b: c_int;
    let c: c_int;
    let d: c_int;

    if number < 0 || number > 9999 {
        Com_sprintf(fileName, MAX_OSPATH as c_int, "demo9999.tga");
        return;
    }

    a = number / 1000;
    number -= a * 1000;
    b = number / 100;
    number -= b * 100;
    c = number / 10;
    number -= c * 10;
    d = number;

    Com_sprintf(
        fileName,
        MAX_OSPATH as c_int,
        &format!("demo{}{}{}{}", a, b, c, d),
    );
}

/// `CL_StartDemoLoop` — restarts the attract-mode demo loop.
///
/// Source: `oracle/codemp/client/cl_main.cpp:618-622`
pub fn CL_StartDemoLoop(common: &mut Common, cl: &mut Client) {
    // start the demo loop again
    Cbuf_AddText(common, "d1\n");
    cl.cls.keyCatchers = 0;
}

/// `CL_NextDemo` — runs the `nextdemo` cvar as a command, then clears it.
///
/// Source: `oracle/codemp/client/cl_main.cpp:632-646`
pub fn CL_NextDemo(view: &mut EngineHostView) {
    let mut v = [0 as c_char; MAX_STRING_CHARS as usize];

    let next = Cvar_VariableString(view.common, "nextdemo").to_string();
    Q_strncpyz(&mut v, &next, MAX_STRING_CHARS as usize);
    v[MAX_STRING_CHARS as usize - 1] = 0;
    let v_str: String = v
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    Com_DPrintf(view.common, &format!("CL_NextDemo: {}\n", v_str));
    if v[0] == 0 {
        return;
    }

    Cvar_Set(view, "nextdemo", "");
    Cbuf_AddText(view.common, &v_str);
    Cbuf_AddText(view.common, "\n");
    Cbuf_Execute(view);
}

/// `CL_ClearState` — wipes the active client game state.
///
/// Source: `oracle/codemp/client/cl_main.cpp:820-824`
pub fn CL_ClearState(cl: &mut Client) {
    //	S_StopAllSounds();
    let size = core::mem::size_of_val(&*cl.cl);
    Com_Memset(&mut *cl.cl as *mut _ as *mut (), 0, size);
}

/// `CL_RequestMotd` — asks the update server for the message of the day.
///
/// Raven: the challenge is randomized against `Com_Milliseconds` so the reply
/// cannot be spoofed from a fixed seed.
/// Source: `oracle/codemp/client/cl_main.cpp:945-986`
pub fn CL_RequestMotd(view: &mut EngineHostView, cl: &mut Client) {
    let mut info = String::new();

    if view.common.cvar(cl.cl_motd).integer == 0 {
        return;
    }
    com_printf(view.common, &format!("Resolving {}\n", UPDATE_SERVER_NAME));
    if NET_StringToAdr(
        UPDATE_SERVER_NAME.as_ptr() as *const c_char,
        &mut cl.cls.updateServer,
    ) == qfalse
    {
        com_printf(view.common, "Couldn't resolve address\n");
        return;
    }
    cl.cls.updateServer.port = (PORT_UPDATE as u16).to_be();
    let adr = cl.cls.updateServer;
    com_printf(
        view.common,
        &format!(
            "{} resolved to {}.{}.{}.{}:{}\n",
            UPDATE_SERVER_NAME,
            adr.ip[0],
            adr.ip[1],
            adr.ip[2],
            adr.ip[3],
            adr.port.to_be()
        ),
    );

    // NOTE TTimo xoring against Com_Milliseconds, otherwise we may not have a true randomization
    // only srand I could catch before here is tr_noise.c l:26 srand(1001)
    // https://zerowing.idsoftware.com/bugzilla/show_bug.cgi?id=382
    // NOTE: the Com_Milliseconds xoring only affects the lower 16-bit word,
    //   but I decided it was enough randomization
    // Two independent LCG draws, in Raven's left-to-right order.
    let high = view.common.qrand.rand();
    let low = view.common.qrand.rand();
    let challenge = ((high << 16) ^ low) ^ Com_Milliseconds(view);
    let challenge_len = cl.cls.updateChallenge.len() as c_int;
    Com_sprintf(
        cl.cls.updateChallenge.as_mut_ptr(),
        challenge_len,
        &format!("{}", challenge),
    );

    let updateChallenge: String = cl
        .cls
        .updateChallenge
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    // `glconfig_t` keeps these two as `*const c_char`, so the site reads the
    // NUL-terminated bytes out of the pointer.
    let renderer_string: String = unsafe { CStr::from_ptr(cl.cls.glconfig.renderer_string) }
        .to_string_lossy()
        .into_owned();
    let vendor_string: String = unsafe { CStr::from_ptr(cl.cls.glconfig.vendor_string) }
        .to_string_lossy()
        .into_owned();

    Info_SetValueForKey(&mut info, "challenge", &updateChallenge);
    Info_SetValueForKey(&mut info, "renderer", &renderer_string);
    Info_SetValueForKey(&mut info, "rvendor", &vendor_string);
    let version = view.common.cvar(view.common.com_version).string.clone();
    Info_SetValueForKey(&mut info, "version", &version);

    Info_SetValueForKey(
        &mut info,
        "cputype",
        Cvar_VariableString(view.common, "sys_cpustring"),
    );
    Info_SetValueForKey(
        &mut info,
        "mhz",
        Cvar_VariableString(view.common, "sys_cpuspeed"),
    );
    Info_SetValueForKey(
        &mut info,
        "memory",
        Cvar_VariableString(view.common, "sys_memory"),
    );
    Info_SetValueForKey(
        &mut info,
        "joystick",
        Cvar_VariableString(view.common, "in_joystick"),
    );
    Info_SetValueForKey(
        &mut info,
        "colorbits",
        &format!("{}", cl.cls.glconfig.colorBits),
    );

    NET_OutOfBandPrint(
        view.common,
        netsrc_t::NS_CLIENT,
        adr,
        format!("getmotd \"{}\"\n", info),
    );
}

/// `CL_Reconnect_f` — reconnects to the last server we joined.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1126-1133`
pub fn CL_Reconnect_f(view: &mut EngineHostView, cl: &mut Client) {
    let servername: String = cl
        .cls
        .servername
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    if servername.is_empty() || servername == "localhost" {
        com_printf(view.common, "Can't reconnect to localhost.\n");
        return;
    }
    Cvar_Set(view, "ui_singlePlayerActive", "0");
    Cbuf_AddText(view.common, &format!("connect {}\n", servername));
}

/// `CL_Rcon_f` — sends an out-of-band rcon command to a server.
///
/// Raven: the four leading `-1` bytes are the connectionless packet header.
/// Source: `oracle/codemp/client/cl_main.cpp:1220-1264`
pub fn CL_Rcon_f(common: &mut Common, cl: &mut Client) {
    let mut message: Vec<u8> = Vec::new();
    let mut to: netadr_t = unsafe { core::mem::zeroed() };

    // Raven tests the `string` POINTER, which the cvar system never leaves null.
    // PORT-NOTE(rcon-null): the ported cvar owns a `String`, so this arm is dead.
    if false {
        com_printf(
            common,
            "You must set 'rcon_password' before\nissuing an rcon command.\n",
        );
        return;
    }

    message.push(0xff);
    message.push(0xff);
    message.push(0xff);
    message.push(0xff);

    message.extend_from_slice(b"rcon ");

    let rcon_password = common.cvar(cl.rcon_client_password).string.clone();
    message.extend_from_slice(rcon_password.as_bytes());
    message.extend_from_slice(b" ");

    let argc = Cmd_Argc(common);
    for i in 1..argc {
        let arg = Cmd_Argv(common, i).to_string();
        message.extend_from_slice(arg.as_bytes());
        message.extend_from_slice(b" ");
    }

    if cl.cls.state as c_int >= connstate_t::CA_CONNECTED as c_int {
        to = cl.clc.netchan.remoteAddress;
    } else {
        if common.cvar(cl.rconAddress).string.is_empty() {
            com_printf(
                common,
                "You must either be connected,\nor set the 'rconAddress' cvar\nto issue rcon commands\n",
            );

            return;
        }
        let rcon_address = common.cvar(cl.rconAddress).string.clone();
        NET_StringToAdr(rcon_address.as_ptr() as *const c_char, &mut to);
        if to.port == 0 {
            to.port = (PORT_SERVER as u16).to_be();
        }
    }

    NET_SendPacket(
        common,
        netsrc_t::NS_CLIENT,
        message.len() as c_int + 1,
        message.as_ptr() as *const (),
        to,
    );
}

/// `CL_OpenedPK3List_f` — prints the pk3 files the filesystem has open.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1400-1402`
pub fn CL_OpenedPK3List_f(common: &mut Common) {
    let names = FS_LoadedPakNames(common);
    let names: String = unsafe { CStr::from_ptr(names) }
        .to_string_lossy()
        .into_owned();
    com_printf(common, &format!("Opened PK3 Names: {}\n", names));
}

/// `CL_ReferencedPK3List_f` — prints the pk3 files the server referenced.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1409-1411`
pub fn CL_ReferencedPK3List_f(common: &mut Common) {
    let names = FS_ReferencedPakNames(common);
    let names: String = unsafe { CStr::from_ptr(names) }
        .to_string_lossy()
        .into_owned();
    com_printf(common, &format!("Referenced PK3 Names: {}\n", names));
}

/// `CL_Configstrings_f` — dumps every non-empty configstring.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1418-1434`
pub fn CL_Configstrings_f(common: &mut Common, cl: &mut Client) {
    let mut ofs: c_int;

    if cl.cls.state as c_int != connstate_t::CA_ACTIVE as c_int {
        com_printf(common, "Not connected to a server.\n");
        return;
    }

    for i in 0..MAX_CONFIGSTRINGS as c_int {
        ofs = cl.cl.gameState.stringOffsets[i as usize];
        if ofs == 0 {
            continue;
        }
        let s: String = cl.cl.gameState.stringData[ofs as usize..]
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();
        com_printf(common, &format!("{:4}: {}\n", i, s));
    }
}

/// `CL_Clientinfo_f` — prints the connection state and the userinfo block.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1441-1448`
pub fn CL_Clientinfo_f(common: &mut Common, cl: &mut Client) {
    com_printf(common, "--------- Client Information ---------\n");
    let state = cl.cls.state as c_int;
    com_printf(common, &format!("state: {}\n", state));
    let servername: String = cl
        .cls
        .servername
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    com_printf(common, &format!("Server: {}\n", servername));
    com_printf(common, "User info settings:\n");
    // PORT-NOTE(info-print): `Cvar_InfoString` returns an owned `String`, and
    // `Info_Print` still takes the raw seam pointer.
    let userinfo = Cvar_InfoString(common, CVAR_USERINFO);
    Info_Print(common, userinfo.as_ptr() as *const c_char);
    com_printf(common, "--------------------------------------\n");
}

/// `CL_CheckForResend` — resends the challenge or connect packet on a timer.
///
/// Raven: the connect packet carries the current userinfo, so the userinfo
/// modified flag is cleared right after we send it.
/// Source: `oracle/codemp/client/cl_main.cpp:1641-1725`
pub fn CL_CheckForResend(common: &mut Common, cl: &mut Client) {
    // don't send anything if playing back a demo
    if cl.clc.demoplaying != qfalse {
        return;
    }

    // resend if we haven't gotten a reply yet
    if cl.cls.state as c_int != connstate_t::CA_CONNECTING as c_int
        && cl.cls.state as c_int != connstate_t::CA_CHALLENGING as c_int
    {
        return;
    }

    if cl.cls.realtime - cl.clc.connectTime < RETRANSMIT_TIMEOUT {
        return;
    }

    cl.clc.connectTime = cl.cls.realtime; // for retransmit requests
    cl.clc.connectPacketCount += 1;

    let state = cl.cls.state as c_int;
    if state == connstate_t::CA_CONNECTING as c_int {
        // requesting a challenge
        let adr = cl.clc.serverAddress;
        NET_OutOfBandPrint(common, netsrc_t::NS_CLIENT, adr, "getchallenge".to_string());
    } else if state == connstate_t::CA_CHALLENGING as c_int {
        // sending back the challenge
        let port = Cvar_VariableValue(common, "net_qport") as c_int;

        let mut info = Cvar_InfoString(common, CVAR_USERINFO);
        Info_SetValueForKey(&mut info, "protocol", &format!("{}", PROTOCOL_VERSION));
        Info_SetValueForKey(&mut info, "qport", &format!("{}", port));
        Info_SetValueForKey(&mut info, "challenge", &format!("{}", cl.clc.challenge));

        let data = format!("connect \"{}\"", info);
        let adr = cl.clc.serverAddress;
        NET_OutOfBandData(
            common,
            netsrc_t::NS_CLIENT,
            adr,
            data.as_ptr() as *mut byte,
            data.len() as c_int,
        );

        // the most current userinfo has been sent, so watch for any
        // newer changes to userinfo variables
        common.cvar_modifiedFlags &= !CVAR_USERINFO;
    } else {
        com_error(
            errorParm_t::ERR_FATAL,
            "CL_CheckForResend: bad cls.state".to_string(),
        );
    }
}

/// `CL_MotdPacket` — accepts the motd reply and publishes it as a cvar.
///
/// Raven: replies whose challenge does not match ours are dropped.
/// Source: `oracle/codemp/client/cl_main.cpp:1771-1792`
pub fn CL_MotdPacket(view: &mut EngineHostView, cl: &mut Client, from: netadr_t) {
    // if not from our server, ignore it
    let updateServer = cl.cls.updateServer;
    if NET_CompareAdr(view.common, from, updateServer) == qfalse {
        return;
    }

    let info = Cmd_Argv(view.common, 1).to_string();

    // check challenge
    let mut challenge = Info_ValueForKey(&info, "challenge");
    let updateChallenge: String = cl
        .cls
        .updateChallenge
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    if challenge != updateChallenge {
        return;
    }

    challenge = Info_ValueForKey(&info, "motd");

    let destsize = cl.cls.updateInfoString.len();
    Q_strncpyz(&mut cl.cls.updateInfoString, &info, destsize);
    Cvar_Set(view, "cl_motdString", &challenge);
}

/// `CL_InitServerInfo` — resets one browser row to a bare address.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1799-1825`
pub fn CL_InitServerInfo(server: *mut serverInfo_t, address: *mut serverAddress_t) {
    unsafe {
        (*server).adr.r#type = netadrtype_t::NA_IP;
        (*server).adr.ip[0] = (*address).ip[0];
        (*server).adr.ip[1] = (*address).ip[1];
        (*server).adr.ip[2] = (*address).ip[2];
        (*server).adr.ip[3] = (*address).ip[3];
        (*server).adr.port = (*address).port;
        (*server).clients = 0;
        (*server).hostName[0] = 0;
        (*server).mapName[0] = 0;
        (*server).maxClients = 0;
        (*server).maxPing = 0;
        (*server).minPing = 0;
        (*server).netType = 0;
        (*server).needPassword = qfalse;
        (*server).trueJedi = 0;
        (*server).weaponDisable = 0;
        (*server).forceDisable = 0;
        (*server).ping = -1;
        (*server).game[0] = 0;
        (*server).gameType = 0;
        //server->pure = qfalse;
    }
}

/// `CL_CheckSVStringEdRef` — expands `@@@` string-editor references in place.
///
/// Raven: "I don't really like doing this. But it utilizes the system that was
/// already in place."
/// Source: `oracle/codemp/client/cl_main.cpp:1951-2018`
pub fn CL_CheckSVStringEdRef(view: &mut EngineHostView, buf: *mut c_char, str: *const c_char) {
    let mut i: c_int = 0;
    let mut b: c_int = 0;
    let mut strLen: c_int = 0;
    let mut gotStrip: qboolean = qfalse;

    unsafe {
        if str.is_null() || *str.offset(0) == 0 {
            if !str.is_null() {
                let mut k = 0isize;
                loop {
                    let ch = *str.offset(k);
                    *buf.offset(k) = ch;
                    if ch == 0 {
                        break;
                    }
                    k += 1;
                }
            }
            return;
        }

        let mut k = 0isize;
        loop {
            let ch = *str.offset(k);
            *buf.offset(k) = ch;
            if ch == 0 {
                break;
            }
            k += 1;
        }

        strLen = k as c_int;

        if strLen >= MAX_STRINGED_SV_STRING {
            return;
        }

        while i < strLen && *str.offset(i as isize) != 0 {
            gotStrip = qfalse;

            if *str.offset(i as isize) == b'@' as c_char && (i + 1) < strLen {
                if *str.offset((i + 1) as isize) == b'@' as c_char && (i + 2) < strLen {
                    if *str.offset((i + 2) as isize) == b'@' as c_char && (i + 3) < strLen {
                        // @@@ should mean to insert a stringed reference here, so insert it into buf at the current place
                        let mut stripRef = [0 as c_char; MAX_STRINGED_SV_STRING as usize];
                        let mut r: c_int = 0;

                        while i < strLen && *str.offset(i as isize) == b'@' as c_char {
                            i += 1;
                        }

                        while i < strLen
                            && *str.offset(i as isize) != 0
                            && *str.offset(i as isize) != b' ' as c_char
                            && *str.offset(i as isize) != b':' as c_char
                            && *str.offset(i as isize) != b'.' as c_char
                            && *str.offset(i as isize) != b'\n' as c_char
                        {
                            stripRef[r as usize] = *str.offset(i as isize);
                            r += 1;
                            i += 1;
                        }
                        stripRef[r as usize] = 0;

                        *buf.offset(b as isize) = 0;
                        let stripRef: String = stripRef
                            .iter()
                            .take_while(|&&c| c != 0)
                            .map(|&c| c as u8 as char)
                            .collect();
                        let stringed = SE_GetString(view, &format!("MP_SVGAME_{}", stripRef));
                        let dest =
                            core::slice::from_raw_parts_mut(buf, MAX_STRINGED_SV_STRING as usize);
                        Q_strcat(dest, MAX_STRINGED_SV_STRING as usize, &stringed);
                        let mut n = 0isize;
                        while *buf.offset(n) != 0 {
                            n += 1;
                        }
                        b = n as c_int;
                    }
                }
            }

            if gotStrip == qfalse {
                *buf.offset(b as isize) = *str.offset(i as isize);
                b += 1;
            }
            i += 1;
        }

        *buf.offset(b as isize) = 0;
    }
}

/// `CL_CheckTimeout` — drops the connection after five silent frames.
///
/// Raven: "timeoutcount saves debugger".
/// Source: `oracle/codemp/client/cl_main.cpp:2212-2229`
pub fn CL_CheckTimeout(view: &mut EngineHostView, cl: &mut Client) {
    //
    // check timeout
    //
    if (view.common.cvar(view.common.cl_paused).integer == 0
        || view.common.cvar(view.common.sv_paused).integer == 0)
        && cl.cls.state as c_int >= connstate_t::CA_CONNECTED as c_int
        && cl.cls.state as c_int != connstate_t::CA_CINEMATIC as c_int
        && (cl.cls.realtime - cl.clc.lastPacketTime) as f32
            > view.common.cvar(cl.cl_timeout).value * 1000.0
    {
        cl.cl.timeoutcount += 1;
        if cl.cl.timeoutcount > 5 {
            // timeoutcount saves debugger
            let psTimedOut = SE_GetString(view, "MP_SVGAME_SERVER_CONNECTION_TIMED_OUT");
            com_printf(view.common, &format!("\n{}\n", psTimedOut));
            com_error(errorParm_t::ERR_DROP, psTimedOut);
            //CL_Disconnect( qtrue );
            return;
        }
    } else {
        cl.cl.timeoutcount = 0;
    }
}

/// `CL_RefPrintf` — the renderer's print hook, routed by print level.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2387-2402`
pub fn CL_RefPrintf(common: &mut Common, print_level: c_int, fmt: *const c_char) {
    // PORT-NOTE(varargs): the caller formats; `fmt` arrives already expanded.
    let msg: String = unsafe { CStr::from_ptr(fmt) }
        .to_string_lossy()
        .into_owned();

    if print_level == printParm_t::PRINT_ALL as c_int {
        com_printf(common, &format!("{}", msg));
    } else if print_level == printParm_t::PRINT_WARNING as c_int {
        com_printf(common, &format!("{}{}", S_COLOR_YELLOW, msg)); // yellow
    } else if print_level == printParm_t::PRINT_DEVELOPER as c_int {
        Com_DPrintf(common, &format!("{}{}", S_COLOR_RED, msg)); // red
    }
}

/// `CL_ShutdownRef` — shuts the renderer down and clears its export table.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2411-2417`
pub fn CL_ShutdownRef(view: &mut EngineHostView, cl: &mut Client) {
    // PORT-NOTE(dec-59.1): the `refexport_t` table is gone, so Raven's null
    // test on `re.Shutdown` and the wipe of the table have no subject left.
    // The site calls the `RE_Shutdown` frontend directly.
    // SAFETY: view-constructor slot, single-threaded, no other cast of the same
    // slot is live across the call.
    let re = unsafe { re_from_view(view) };
    RE_Shutdown(
        view,
        &re.cvars,
        Arc::make_mut(&mut re.sim.published),
        &mut re.img_state,
        &mut re.font,
        true,
    );
}

/// Copy the renderer's owned `GlConfig` into the ABI-frozen `cls.glconfig` that
/// the cgame and ui modules read through `CG_GETGLCONFIG`/`UI_GETGLCONFIG`.
///
/// Raven points the four `const char *` fields at the renderer's own static
/// buffers. The port owns the same four strings on `Client`, so the pointers
/// stay valid for as long as `cls.glconfig` does.
///
/// Source: `oracle/codemp/cgame/tr_types.h:299-325`
fn cl_set_glconfig(cl: &mut Client, glconfig: &GlConfig) {
    cl.glconfigStrings = [
        CString::new(glconfig.renderer_string.as_str()).unwrap_or_default(),
        CString::new(glconfig.vendor_string.as_str()).unwrap_or_default(),
        CString::new(glconfig.version_string.as_str()).unwrap_or_default(),
        CString::new(glconfig.extensions_string.as_str()).unwrap_or_default(),
    ];
    cl.cls.glconfig = glconfig_t {
        renderer_string: cl.glconfigStrings[0].as_ptr(),
        vendor_string: cl.glconfigStrings[1].as_ptr(),
        version_string: cl.glconfigStrings[2].as_ptr(),
        extensions_string: cl.glconfigStrings[3].as_ptr(),
        maxTextureSize: glconfig.max_texture_size,
        maxActiveTextures: glconfig.max_active_textures,
        maxTextureFilterAnisotropy: glconfig.max_texture_filter_anisotropy,
        colorBits: glconfig.color_bits,
        depthBits: glconfig.depth_bits,
        stencilBits: glconfig.stencil_bits,
        deviceSupportsGamma: glconfig.device_supports_gamma as qboolean,
        textureCompression: glconfig.texture_compression,
        textureEnvAddAvailable: glconfig.texture_env_add_available as qboolean,
        clampToEdgeAvailable: glconfig.clamp_to_edge_available as qboolean,
        vidWidth: glconfig.vid_width,
        vidHeight: glconfig.vid_height,
        displayFrequency: glconfig.display_frequency,
        isFullscreen: glconfig.is_fullscreen as qboolean,
        stereoEnabled: glconfig.stereo_enabled as qboolean,
    };
}

/// `CL_InitRenderer` — starts the renderer and registers the console assets.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2424-2435`
pub fn CL_InitRenderer(view: &mut EngineHostView, cl: &mut Client) {
    // this sets up the renderer and calls R_Init
    // SAFETY (both casts): view-constructor slots, single-threaded, no other
    // cast of the same slot is live across the calls.
    let re = unsafe { re_from_view(view) };
    let rm = unsafe { rm_from_view(view) };
    let glconfig = RE_BeginRegistration(
        view,
        &mut re.cvars,
        &mut re.sim,
        &mut re.img_state,
        rm,
        &mut re.frame,
        &mut re.scene,
        &mut re.frame_data,
        &mut re.noise,
        &mut re.rng,
        &mut re.font,
        &mut re.world_effects,
        &mut re.qs,
        &mut re.sky_view,
        &mut re.sky,
    );
    cl_set_glconfig(cl, &glconfig);

    // load character sets
    cl.cls.charSetShader = RE_RegisterShaderNoMip(
        "gfx/2d/charsgrid_med",
        &mut re.qs,
        &mut re.frame,
        Arc::make_mut(&mut re.sim.published),
        view,
        &re.cvars,
        rm,
        &mut re.img_state,
        &mut re.sky_view,
        &mut re.sky,
    );

    cl.cls.whiteShader = RE_RegisterShader(
        "white",
        &mut re.qs,
        &mut re.frame,
        Arc::make_mut(&mut re.sim.published),
        view,
        &re.cvars,
        rm,
        &mut re.img_state,
        &mut re.sky_view,
        &mut re.sky,
    );
    cl.cls.consoleShader = RE_RegisterShader(
        "console",
        &mut re.qs,
        &mut re.frame,
        Arc::make_mut(&mut re.sim.published),
        view,
        &re.cvars,
        rm,
        &mut re.img_state,
        &mut re.sky_view,
        &mut re.sky,
    );
    cl.g_console_field_width = cl.cls.glconfig.vidWidth / SMALLCHAR_WIDTH - 2;
    cl.kg.g_consoleField.widthInChars = cl.g_console_field_width;
}

/// `CL_InitRef` — unpauses the client once the renderer is up.
///
/// DEC-59.1 dropped `refexport_t`, `GetRefAPI`, and `REF_API_VERSION`, so the
/// table fetch and the `re = *ret` copy have no counterpart. The platform shell
/// seats `Engine.re`, and every engine-interior call names an `RE_*` frontend
/// function directly. The unpause is the whole remaining body.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2480-2499`
pub fn CL_InitRef(view: &mut EngineHostView) {
    // unpause so the cgame definately gets a snapshot and renders a frame
    Cvar_Set(view, "cl_paused", "0");
}

/// `CL_SetModel_f` — sets or reports the `model` userinfo cvar.
///
/// Raven (rww): this is currently broken and does not seem to work for
/// connecting clients.
/// Source: `oracle/codemp/client/cl_main.cpp:2507-2536`
pub fn CL_SetModel_f(view: &mut EngineHostView) {
    let mut name = [0 as c_char; 256];

    let arg = Cmd_Argv(view.common, 1).to_string();
    if !arg.is_empty() {
        /*
        //If you wanted to be foolproof you would put this on the server I guess. But that
        //tends to put things out of sync regarding cvar status. And I sort of doubt someone
        //is going to write a client and figure out the protocol so that they can annoy people
        //by changing models real fast.
        int curTime = Com_Milliseconds();
        if (gCLModelDelay > curTime)
        {
            Com_Printf("You can only change your model every %i seconds.\n", (MODEL_CHANGE_DELAY/1000));
            return;
        }

        gCLModelDelay = curTime + MODEL_CHANGE_DELAY;
        */
        //rww: this is currently broken and does not seem to work for connecting clients
        Cvar_Set(view, "model", &arg);
    } else {
        Cvar_VariableStringBuffer(view.common, "model", name.as_mut_ptr(), name.len() as c_int);
        let name: String = name
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();
        com_printf(view.common, &format!("model is set to {}\n", name));
    }
}

/// `CL_SetForcePowers_f` — the `forcepowers` command, deliberately inert.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2538-2540`
pub fn CL_SetForcePowers_f() {}

/// `CL_SetServerInfo` — fills one browser row from a server info string.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2776-2801`
pub fn CL_SetServerInfo(server: *mut serverInfo_t, info: *const c_char, ping: c_int) {
    unsafe {
        if !server.is_null() {
            if !info.is_null() {
                let info: String = CStr::from_ptr(info).to_string_lossy().into_owned();
                (*server).clients = atoi(&Info_ValueForKey(&info, "clients"));
                Q_strncpyz(
                    &mut (*server).hostName,
                    &Info_ValueForKey(&info, "hostname"),
                    MAX_NAME_LENGTH as usize,
                );
                Q_strncpyz(
                    &mut (*server).mapName,
                    &Info_ValueForKey(&info, "mapname"),
                    MAX_NAME_LENGTH as usize,
                );
                (*server).maxClients = atoi(&Info_ValueForKey(&info, "sv_maxclients"));
                Q_strncpyz(
                    &mut (*server).game,
                    &Info_ValueForKey(&info, "game"),
                    MAX_NAME_LENGTH as usize,
                );
                (*server).gameType = atoi(&Info_ValueForKey(&info, "gametype"));
                (*server).netType = atoi(&Info_ValueForKey(&info, "nettype"));
                (*server).minPing = atoi(&Info_ValueForKey(&info, "minping"));
                (*server).maxPing = atoi(&Info_ValueForKey(&info, "maxping"));
                //			server->allowAnonymous = atoi(Info_ValueForKey(info, "sv_allowAnonymous"));
                (*server).needPassword = atoi(&Info_ValueForKey(&info, "needpass")) as qboolean;
                (*server).trueJedi = atoi(&Info_ValueForKey(&info, "truejedi"));
                (*server).weaponDisable = atoi(&Info_ValueForKey(&info, "wdisable"));
                (*server).forceDisable = atoi(&Info_ValueForKey(&info, "fdisable"));
                //			server->pure = (qboolean)atoi(Info_ValueForKey(info, "pure" ));
            }
            (*server).ping = ping;
        }
    }
}

/// `CL_GetServerStatus` — finds or recycles the status slot for an address.
///
/// Raven: an already-retrieved slot is reused before the oldest one.
/// Source: `oracle/codemp/client/cl_main.cpp:2967-2995`
pub fn CL_GetServerStatus(
    common: &mut Common,
    cl: &mut Client,
    from: netadr_t,
) -> *mut serverStatus_t {
    let mut oldest: c_int;
    let mut oldestTime: c_int;

    for i in 0..MAX_SERVERSTATUSREQUESTS as c_int {
        let address = cl.cl_serverStatusList[i as usize].address;
        if NET_CompareAdr(common, from, address) != qfalse {
            return &mut cl.cl_serverStatusList[i as usize] as *mut serverStatus_t;
        }
    }
    for i in 0..MAX_SERVERSTATUSREQUESTS as c_int {
        if cl.cl_serverStatusList[i as usize].retrieved != qfalse {
            return &mut cl.cl_serverStatusList[i as usize] as *mut serverStatus_t;
        }
    }
    oldest = -1;
    oldestTime = 0;
    for i in 0..MAX_SERVERSTATUSREQUESTS as c_int {
        if oldest == -1 || cl.cl_serverStatusList[i as usize].startTime < oldestTime {
            oldest = i;
            oldestTime = cl.cl_serverStatusList[i as usize].startTime;
        }
    }
    if oldest != -1 {
        return &mut cl.cl_serverStatusList[oldest as usize] as *mut serverStatus_t;
    }
    cl.serverStatusCount += 1;
    let slot = cl.serverStatusCount & (MAX_SERVERSTATUSREQUESTS as c_int - 1);
    &mut cl.cl_serverStatusList[slot as usize] as *mut serverStatus_t
}

/// `CL_ServerStatusResponse` — parses a `statusResponse` into a status slot.
///
/// Raven: the cvar block and the player rows are also printed when the request
/// came from the `serverstatus` command.
/// Source: `oracle/codemp/client/cl_main.cpp:3065-3155`
pub fn CL_ServerStatusResponse(
    view: &mut EngineHostView,
    cl: &mut Client,
    from: netadr_t,
    msg: *mut msg_t,
) {
    let mut info = [0 as c_char; MAX_INFO_STRING as usize];
    let mut score: c_int;
    let mut ping: c_int;
    let mut len: c_int;
    let mut serverStatus: *mut serverStatus_t = core::ptr::null_mut();

    for i in 0..MAX_SERVERSTATUSREQUESTS as c_int {
        let address = cl.cl_serverStatusList[i as usize].address;
        if NET_CompareAdr(view.common, from, address) != qfalse {
            serverStatus = &mut cl.cl_serverStatusList[i as usize] as *mut serverStatus_t;
            break;
        }
    }
    // if we didn't request this server status
    if serverStatus.is_null() {
        return;
    }

    let line = MSG_ReadStringLine(view.common, msg);
    let mut s: Vec<u8> = line.as_bytes().to_vec();
    s.push(0);
    let mut p: usize = 0;

    unsafe {
        len = 0;
        let cap = (*serverStatus).string.len() as c_int - len;
        Com_sprintf(
            (*serverStatus).string.as_mut_ptr().offset(len as isize),
            cap,
            &line,
        );

        if (*serverStatus).print != qfalse {
            let address = (*serverStatus).address;
            com_printf(
                view.common,
                &format!(
                    "Server ({}.{}.{}.{}:{})\n",
                    address.ip[0],
                    address.ip[1],
                    address.ip[2],
                    address.ip[3],
                    address.port.to_be()
                ),
            );
            com_printf(view.common, "Server settings:\n");
            // print cvars
            while s[p] != 0 {
                let mut i = 0;
                while i < 2 && s[p] != 0 {
                    if s[p] == b'\\' {
                        p += 1;
                    }
                    let mut l = 0usize;
                    while s[p] != 0 {
                        info[l] = s[p] as c_char;
                        l += 1;
                        if l >= MAX_INFO_STRING as usize - 1 {
                            break;
                        }
                        p += 1;
                        if s[p] == b'\\' {
                            break;
                        }
                    }
                    info[l] = 0;
                    let text: String = info
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| c as u8 as char)
                        .collect();
                    if i != 0 {
                        com_printf(view.common, &format!("{}\n", text));
                    } else {
                        com_printf(view.common, &format!("{:<24}", text));
                    }
                    i += 1;
                }
            }
        }

        len = (*serverStatus)
            .string
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(0) as c_int;
        let cap = (*serverStatus).string.len() as c_int - len;
        Com_sprintf(
            (*serverStatus).string.as_mut_ptr().offset(len as isize),
            cap,
            "\\",
        );

        if (*serverStatus).print != qfalse {
            com_printf(view.common, "\nPlayers:\n");
            com_printf(view.common, "num: score: ping: name:\n");
        }
        let mut i: c_int = 0;
        let mut row = MSG_ReadStringLine(view.common, msg);
        while !row.is_empty() {
            len = (*serverStatus)
                .string
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(0) as c_int;
            let cap = (*serverStatus).string.len() as c_int - len;
            Com_sprintf(
                (*serverStatus).string.as_mut_ptr().offset(len as isize),
                cap,
                &format!("\\{}", row),
            );

            if (*serverStatus).print != qfalse {
                score = 0;
                ping = 0;
                // Raven scans "%d %d" off the head of the row.
                let mut fields = row.split_whitespace();
                if let Some(t) = fields.next() {
                    score = t.parse::<c_int>().unwrap_or(0);
                }
                if let Some(t) = fields.next() {
                    ping = t.parse::<c_int>().unwrap_or(0);
                }
                // Raven walks past two spaces to reach the name, or falls back.
                let name = match row.find(' ') {
                    None => "unknown".to_string(),
                    Some(first) => match row[first + 1..].find(' ') {
                        None => "unknown".to_string(),
                        Some(second) => row[first + 1 + second + 1..].to_string(),
                    },
                };
                com_printf(
                    view.common,
                    &format!("{:<2}   {:<3}    {:<3}   {}\n", i, score, ping, name),
                );
            }
            i += 1;
            row = MSG_ReadStringLine(view.common, msg);
        }
        len = (*serverStatus)
            .string
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(0) as c_int;
        let cap = (*serverStatus).string.len() as c_int - len;
        Com_sprintf(
            (*serverStatus).string.as_mut_ptr().offset(len as isize),
            cap,
            "\\",
        );

        (*serverStatus).time = Com_Milliseconds(view);
        (*serverStatus).address = from;
        (*serverStatus).pending = qfalse;
        if (*serverStatus).print != qfalse {
            (*serverStatus).retrieved = qtrue;
        }
    }
}

/// `CL_LocalServers_f` — broadcasts `getinfo` on every server port.
///
/// Raven: each message goes out twice in case one is dropped.
/// Source: `oracle/codemp/client/cl_main.cpp:3162-3200`
pub fn CL_LocalServers_f(common: &mut Common, cl: &mut Client) {
    let message: &[u8];
    let mut to: netadr_t = unsafe { core::mem::zeroed() };

    com_printf(common, "Scanning for servers on the local network...\n");

    // reset the list, waiting for response
    cl.cls.numlocalservers = 0;
    cl.cls.pingUpdateSource = AS_LOCAL;

    for i in 0..MAX_OTHER_SERVERS as c_int {
        let b: qboolean = cl.cls.localServers[i as usize].visible;
        let size = core::mem::size_of::<serverInfo_t>();
        Com_Memset(
            &mut cl.cls.localServers[i as usize] as *mut serverInfo_t as *mut (),
            0,
            size,
        );
        cl.cls.localServers[i as usize].visible = b;
    }
    let size = core::mem::size_of::<netadr_t>();
    Com_Memset(&mut to as *mut netadr_t as *mut (), 0, size);

    // The 'xxx' in the message is a challenge that will be echoed back
    // by the server.  We don't care about that here, but master servers
    // can use that to prevent spoofed server responses from invalid ip
    message = b"\xff\xff\xff\xffgetinfo xxx";

    // send each message twice in case one is dropped
    for _i in 0..2 {
        // send a broadcast packet on each server port
        // we support multiple server ports so a single machine
        // can nicely run multiple servers
        for j in 0..NUM_SERVER_PORTS as c_int {
            to.port = ((PORT_SERVER as c_int + j) as u16).to_be();

            to.r#type = netadrtype_t::NA_BROADCAST;
            NET_SendPacket(
                common,
                netsrc_t::NS_CLIENT,
                message.len() as c_int,
                message.as_ptr() as *const (),
                to,
            );

            to.r#type = netadrtype_t::NA_BROADCAST_IPX;
            NET_SendPacket(
                common,
                netsrc_t::NS_CLIENT,
                message.len() as c_int,
                message.as_ptr() as *const (),
                to,
            );
        }
    }
}

/// `CL_GlobalServers_f` — asks a master server for its server list.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3208-3255`
pub fn CL_GlobalServers_f(common: &mut Common, cl: &mut Client) {
    let mut to: netadr_t = unsafe { core::mem::zeroed() };
    let mut command = String::new();

    if Cmd_Argc(common) < 3 {
        com_printf(
            common,
            "usage: globalservers <master# 0-1> <protocol> [keywords]\n",
        );
        return;
    }

    cl.cls.masterNum = atoi(Cmd_Argv(common, 1));

    com_printf(common, "Requesting servers from the master...\n");

    // reset the list, waiting for response
    // -1 is used to distinguish a "no response"

    /*	if( cls.masterNum == 1 ) {
        NET_StringToAdr( "master.quake3world.com", &to );
        cls.nummplayerservers = -1;
        cls.pingUpdateSource = AS_MPLAYER;
    }
    else
    */
    {
        NET_StringToAdr(MASTER_SERVER_NAME.as_ptr() as *const c_char, &mut to);
        cl.cls.numglobalservers = -1;
        cl.cls.pingUpdateSource = AS_GLOBAL;
    }
    to.r#type = netadrtype_t::NA_IP;
    to.port = (PORT_MASTER as u16).to_be();

    command = format!("getservers {}", Cmd_Argv(common, 2));

    // tack on keywords
    let count = Cmd_Argc(common);
    for i in 3..count {
        command = format!("{} {}", command, Cmd_Argv(common, i));
    }

    // if we are a demo, automatically add a "demo" keyword
    if Cvar_VariableValue(common, "fs_restrict") != 0.0 {
        command = format!("{} demo", command);
    }

    NET_OutOfBandPrint(common, netsrc_t::NS_SERVER, to, command);
}

/// `CL_GetPingInfo` — copies one ping slot's info string out to the UI.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3321-3332`
pub fn CL_GetPingInfo(cl: &mut Client, n: c_int, buf: *mut c_char, buflen: c_int) {
    if cl.cl_pinglist[n as usize].adr.port == 0 {
        // empty slot
        if buflen != 0 {
            unsafe {
                *buf.offset(0) = 0;
            }
        }
        return;
    }

    let info: String = cl.cl_pinglist[n as usize]
        .info
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    let dest = unsafe { core::slice::from_raw_parts_mut(buf, buflen as usize) };
    Q_strncpyz(dest, &info, buflen as usize);
}

/// `CL_ClearPing` — frees one ping slot.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3339-3345`
pub fn CL_ClearPing(cl: &mut Client, n: c_int) {
    if n < 0 || n >= MAX_PINGREQUESTS as c_int {
        return;
    }

    cl.cl_pinglist[n as usize].adr.port = 0;
}

/// `CL_GetPingQueueCount` — counts the ping slots that are in use.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3352-3368`
pub fn CL_GetPingQueueCount(cl: &mut Client) -> c_int {
    let mut count: c_int;

    count = 0;

    for i in 0..MAX_PINGREQUESTS as c_int {
        if cl.cl_pinglist[i as usize].adr.port != 0 {
            count += 1;
        }
    }

    count
}

/// `CL_GetFreePing` — claims a ping slot, recycling the oldest when full.
///
/// Raven: a slot still inside its 500 ms response window is never stolen.
/// Source: `oracle/codemp/client/cl_main.cpp:3375-3425`
pub fn CL_GetFreePing(cl: &mut Client) -> *mut ping_t {
    let mut best: *mut ping_t;
    let mut oldest: c_int;
    let mut time: c_int;

    for i in 0..MAX_PINGREQUESTS as c_int {
        // find free ping slot
        if cl.cl_pinglist[i as usize].adr.port != 0 {
            if cl.cl_pinglist[i as usize].time == 0 {
                if cl.cls.realtime - cl.cl_pinglist[i as usize].start < 500 {
                    // still waiting for response
                    continue;
                }
            } else if cl.cl_pinglist[i as usize].time < 500 {
                // results have not been queried
                continue;
            }
        }

        // clear it
        cl.cl_pinglist[i as usize].adr.port = 0;
        return &mut cl.cl_pinglist[i as usize] as *mut ping_t;
    }

    // use oldest entry
    best = &mut cl.cl_pinglist[0] as *mut ping_t;
    oldest = c_int::MIN;
    for i in 0..MAX_PINGREQUESTS as c_int {
        // scan for oldest
        time = cl.cls.realtime - cl.cl_pinglist[i as usize].start;
        if time > oldest {
            oldest = time;
            best = &mut cl.cl_pinglist[i as usize] as *mut ping_t;
        }
    }

    best
}

/// `CL_ShowIP_f` — prints the machine's network addresses.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3613-3615`
pub fn CL_ShowIP_f(common: &mut Common) {
    Sys_ShowIP(common);
}

/// `CL_MakeMonkeyDoLaundry` — the monkey test's periodic packet corruption.
///
/// Source: `oracle/codemp/client/cl_main.cpp:192-200`
pub fn CL_MakeMonkeyDoLaundry(common: &mut Common, cl: &mut Client) {
    if Sys_MonkeyShouldBeSpanked() != qfalse {
        if cl.cls.framecount & 255 == 0 {
            // Raven's `random()` macro: (rand() & 0x7fff) / (float)0x7fff.
            let draw = common.qrand.rand();
            if (draw & 0x7fff) as f32 / (0x7fff as f32) < 0.1 {
                CL_ChangeReliableCommand(common, cl);
            }
        }
    }
}

/// `CL_Record_f` — opens a demo file and writes the current gamestate into it.
///
/// Raven: the rest of the demo file is copied straight from net messages.
/// Source: `oracle/codemp/client/cl_main.cpp:295-453`
pub fn CL_Record_f(view: &mut EngineHostView, cl: &mut Client) {
    let mut name = [0 as c_char; MAX_OSPATH as usize];
    let mut bufData = [0u8; MAX_MSGLEN as usize];
    let mut buf: msg_t = unsafe { core::mem::zeroed() };
    let mut len: c_int;
    let mut ent: *mut entityState_t;
    let mut nullstate: entityState_t = unsafe { core::mem::zeroed() };

    if Cmd_Argc(view.common) > 2 {
        com_printf(view.common, "record <demoname>\n");
        return;
    }

    if cl.clc.demorecording != qfalse {
        if cl.clc.spDemoRecording == qfalse {
            com_printf(view.common, "Already recording.\n");
        }
        return;
    }

    if cl.cls.state as c_int != connstate_t::CA_ACTIVE as c_int {
        com_printf(view.common, "You must be in a level to record.\n");
        return;
    }

    if Cvar_VariableValue(view.common, "g_synchronousClients") == 0.0 {
        com_printf(
            view.common,
            "The server must have 'g_synchronousClients 1' set for demos\n",
        );
        return;
    }

    if Cmd_Argc(view.common) == 2 {
        let s = Cmd_Argv(view.common, 1).to_string();
        let destsize = cl.demoName.len();
        Q_strncpyz(&mut cl.demoName, &s, destsize);
        let demoName: String = cl
            .demoName
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();
        Com_sprintf(
            name.as_mut_ptr(),
            name.len() as c_int,
            &format!("demos/{}.dm_{}", demoName, PROTOCOL_VERSION),
        );
    } else {
        // scan for a free demo name
        for number in 0..=9999 {
            CL_DemoFilename(number, cl.demoName.as_mut_ptr());
            let demoName: String = cl
                .demoName
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8 as char)
                .collect();
            Com_sprintf(
                name.as_mut_ptr(),
                name.len() as c_int,
                &format!("demos/{}.dm_{}", demoName, PROTOCOL_VERSION),
            );

            let name_str: String = name
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8 as char)
                .collect();
            len = FS_ReadFile(view, &name_str, core::ptr::null_mut());
            if len <= 0 {
                break; // file doesn't exist
            }
        }
    }

    // open the demo file

    let name_str: String = name
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    com_printf(view.common, &format!("recording to {}.\n", name_str));
    cl.clc.demofile = FS_FOpenFileWrite(view.common, &name_str);
    if cl.clc.demofile == 0 {
        com_printf(view.common, "ERROR: couldn't open.\n");
        return;
    }
    cl.clc.demorecording = qtrue;
    if Cvar_VariableValue(view.common, "ui_recordSPDemo") != 0.0 {
        cl.clc.spDemoRecording = qtrue;
    } else {
        cl.clc.spDemoRecording = qfalse;
    }

    let demoName: String = cl
        .demoName
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    let destsize = cl.clc.demoName.len();
    Q_strncpyz(&mut cl.clc.demoName, &demoName, destsize);

    // don't start saving messages until a non-delta compressed message is received
    cl.clc.demowaiting = qtrue;

    // write out the gamestate message
    MSG_Init(view, &mut buf, bufData.as_mut_ptr(), bufData.len() as c_int);
    MSG_Bitstream(&mut buf);

    // NOTE, MRE: all server->client messages now acknowledge
    MSG_WriteLong(view.common, &mut buf, cl.clc.reliableSequence);

    MSG_WriteByte(view.common, &mut buf, svc_gamestate as c_int);
    MSG_WriteLong(view.common, &mut buf, cl.clc.serverCommandSequence);

    // configstrings
    for i in 0..MAX_CONFIGSTRINGS as c_int {
        if cl.cl.gameState.stringOffsets[i as usize] == 0 {
            continue;
        }
        let ofs = cl.cl.gameState.stringOffsets[i as usize] as usize;
        let s: String = cl.cl.gameState.stringData[ofs..]
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();
        MSG_WriteByte(view.common, &mut buf, svc_configstring as c_int);
        MSG_WriteShort(view.common, &mut buf, i);
        MSG_WriteBigString(view.common, &mut buf, &s);
    }

    // baselines
    let size = core::mem::size_of::<entityState_t>();
    Com_Memset(&mut nullstate as *mut entityState_t as *mut (), 0, size);
    for i in 0..MAX_GENTITIES as c_int {
        ent = &mut cl.cl.entityBaselines[i as usize] as *mut entityState_t;
        if unsafe { (*ent).number } == 0 {
            continue;
        }
        MSG_WriteByte(view.common, &mut buf, svc_baseline as c_int);
        MSG_WriteDeltaEntity(view.common, &mut buf, &mut nullstate, ent, qtrue);
    }

    MSG_WriteByte(view.common, &mut buf, svc_EOF as c_int);

    // finished writing the gamestate stuff

    // write the client num
    MSG_WriteLong(view.common, &mut buf, cl.clc.clientNum);
    // write the checksum feed
    MSG_WriteLong(view.common, &mut buf, cl.clc.checksumFeed);

    // RMG stuff
    if cl.clc.rmgHeightMapSize != 0 {
        // Height map
        MSG_WriteShort(
            view.common,
            &mut buf,
            cl.clc.rmgHeightMapSize as u16 as c_int,
        );
        MSG_WriteBits(view.common, &mut buf, 0, 1);
        MSG_WriteData(
            view.common,
            &mut buf,
            cl.clc.rmgHeightMap.as_ptr() as *const (),
            cl.clc.rmgHeightMapSize,
        );

        // Flatten map
        MSG_WriteShort(
            view.common,
            &mut buf,
            cl.clc.rmgHeightMapSize as u16 as c_int,
        );
        MSG_WriteBits(view.common, &mut buf, 0, 1);
        MSG_WriteData(
            view.common,
            &mut buf,
            cl.clc.rmgFlattenMap.as_ptr() as *const (),
            cl.clc.rmgHeightMapSize,
        );

        // Seed
        MSG_WriteLong(view.common, &mut buf, cl.clc.rmgSeed);

        // Automap symbols
        MSG_WriteShort(
            view.common,
            &mut buf,
            cl.clc.rmgAutomapSymbolCount as u16 as c_int,
        );
        for i in 0..cl.clc.rmgAutomapSymbolCount {
            let sym = &cl.clc.rmgAutomapSymbols[i as usize];
            let mType = sym.mType as u8 as c_int;
            let mSide = sym.mSide as u8 as c_int;
            let x = sym.mOrigin[0] as c_int;
            let y = sym.mOrigin[1] as c_int;
            MSG_WriteByte(view.common, &mut buf, mType);
            MSG_WriteByte(view.common, &mut buf, mSide);
            MSG_WriteLong(view.common, &mut buf, x);
            MSG_WriteLong(view.common, &mut buf, y);
        }
    } else {
        MSG_WriteShort(view.common, &mut buf, 0);
    }

    // finished writing the client packet
    MSG_WriteByte(view.common, &mut buf, svc_EOF as c_int);

    // write it to the demo file
    len = (cl.clc.serverMessageSequence - 1).to_le();
    FS_Write(
        view.common,
        &len as *const c_int as *const (),
        4,
        cl.clc.demofile,
    );

    len = buf.cursize.to_le();
    FS_Write(
        view.common,
        &len as *const c_int as *const (),
        4,
        cl.clc.demofile,
    );
    FS_Write(
        view.common,
        buf.data as *const (),
        buf.cursize,
        cl.clc.demofile,
    );

    // the rest of the demo file will be copied from net messages
}

/// `CL_ForwardCommandToServer` — sends an unknown command on to the server.
///
/// Raven: key-up commands and `+` commands are never forwarded.
/// Source: `oracle/codemp/client/cl_main.cpp:913-937`
pub fn CL_ForwardCommandToServer(common: &mut Common, cl: &mut Client, string: *const c_char) {
    let cmd = Cmd_Argv(common, 0).to_string();

    // ignore key up commands
    if cmd.starts_with('-') {
        return;
    }

    if cl.clc.demoplaying != qfalse
        || (cl.cls.state as c_int) < connstate_t::CA_CONNECTED as c_int
        || cmd.starts_with('+')
    {
        com_printf(common, &format!("Unknown command \"{}\"\n", cmd));
        return;
    }

    if Cmd_Argc(common) > 1 {
        CL_AddReliableCommand(cl, string);
    } else {
        CL_AddReliableCommand(cl, cmd.as_ptr() as *const c_char);
    }
}

/// `CL_ForwardToServer_f` — the `cmd` command, forwards its arguments.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1092-1102`
pub fn CL_ForwardToServer_f(common: &mut Common, cl: &mut Client) {
    if cl.cls.state as c_int != connstate_t::CA_ACTIVE as c_int || cl.clc.demoplaying != qfalse {
        com_printf(common, "Not connected to a server.\n");
        return;
    }

    // don't forward the first argument
    if Cmd_Argc(common) > 1 {
        let args = Cmd_Args(common);
        CL_AddReliableCommand(cl, args.as_ptr() as *const c_char);
    }
}

/// `CL_SendPureChecksums` — reports our referenced pk3 checksums to the server.
///
/// Raven: the two leading characters are shifted by 10, turning "Of" into "Yf".
/// Source: `oracle/codemp/client/cl_main.cpp:1271-1289`
pub fn CL_SendPureChecksums(common: &mut Common, cl: &mut Client) {
    let mut cMsg = [0 as c_char; MAX_INFO_VALUE as usize];

    // if we are pure we need to send back a command with our referenced pk3 checksums
    let pChecksums = FS_ReferencedPakPureChecksums(common);
    let pChecksums: String = unsafe { CStr::from_ptr(pChecksums) }
        .to_string_lossy()
        .into_owned();

    // "cp"
    // "Yf"
    Com_sprintf(cMsg.as_mut_ptr(), cMsg.len() as c_int, "Yf ");
    let size = cMsg.len();
    Q_strcat(&mut cMsg, size, &pChecksums);
    for i in 0..2 {
        cMsg[i] += 10;
    }
    CL_AddReliableCommand(cl, cMsg.as_ptr());
}

/// `CL_ResetPureClientAtServer` — tells the server to forget our pure state.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1296-1298`
pub fn CL_ResetPureClientAtServer(cl: &mut Client) {
    let cmd = format!("vdr");
    CL_AddReliableCommand(cl, cmd.as_ptr() as *const c_char);
}

/// `CL_BeginDownload` — starts one file download and publishes it to the UI.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1522-1542`
pub fn CL_BeginDownload(
    view: &mut EngineHostView,
    cl: &mut Client,
    localName: *const c_char,
    remoteName: *const c_char,
) {
    let localName: String = unsafe { CStr::from_ptr(localName) }
        .to_string_lossy()
        .into_owned();
    let remoteName: String = unsafe { CStr::from_ptr(remoteName) }
        .to_string_lossy()
        .into_owned();

    Com_DPrintf(
        view.common,
        &format!(
            "***** CL_BeginDownload *****\nLocalname: {}\nRemotename: {}\n****************************\n",
            localName, remoteName
        ),
    );

    let destsize = cl.clc.downloadName.len();
    Q_strncpyz(&mut cl.clc.downloadName, &localName, destsize);
    let cap = cl.clc.downloadTempName.len() as c_int;
    Com_sprintf(
        cl.clc.downloadTempName.as_mut_ptr(),
        cap,
        &format!("{}.tmp", localName),
    );

    // Set so UI gets access to it
    Cvar_Set(view, "cl_downloadName", &remoteName);
    Cvar_Set(view, "cl_downloadSize", "0");
    Cvar_Set(view, "cl_downloadCount", "0");
    Cvar_SetValue(view, "cl_downloadTime", cl.cls.realtime as f32);

    cl.clc.downloadBlock = 0; // Starting new file
    cl.clc.downloadCount = 0;

    let cmd = format!("download {}", remoteName);
    CL_AddReliableCommand(cl, cmd.as_ptr() as *const c_char);
}

/// `CL_ServersResponsePacket` — parses a master server's packed address list.
///
/// Raven: addresses past the browser list are kept in the extra global list.
/// Source: `oracle/codemp/client/cl_main.cpp:1834-1946`
pub fn CL_ServersResponsePacket(
    common: &mut Common,
    cl: &mut Client,
    from: netadr_t,
    msg: *mut msg_t,
) {
    let mut count: c_int;
    let mut max: c_int;
    let total: c_int;
    let mut addresses: [serverAddress_t; MAX_SERVERSPERPACKET as usize] =
        unsafe { core::mem::zeroed() };
    let mut numservers: c_int;

    com_printf(common, "CL_ServersResponsePacket\n");

    if cl.cls.numglobalservers == -1 {
        // state to detect lack of servers or lack of response
        cl.cls.numglobalservers = 0;
        cl.cls.numGlobalServerAddresses = 0;
    }

    if cl.cls.nummplayerservers == -1 {
        cl.cls.nummplayerservers = 0;
    }

    // parse through server response string
    numservers = 0;
    let mut buffptr: *mut byte = unsafe { (*msg).data };
    let buffend: *mut byte = unsafe { buffptr.offset((*msg).cursize as isize) };
    unsafe {
        while buffptr.offset(1) < buffend {
            // advance to initial token
            loop {
                let ch = *buffptr;
                buffptr = buffptr.offset(1);
                if ch == b'\\' {
                    break;
                }
                if buffptr >= buffend {
                    break;
                }
            }

            if buffptr >= buffend.offset(-6) {
                break;
            }

            // parse out ip
            addresses[numservers as usize].ip[0] = *buffptr;
            buffptr = buffptr.offset(1);
            addresses[numservers as usize].ip[1] = *buffptr;
            buffptr = buffptr.offset(1);
            addresses[numservers as usize].ip[2] = *buffptr;
            buffptr = buffptr.offset(1);
            addresses[numservers as usize].ip[3] = *buffptr;
            buffptr = buffptr.offset(1);

            // parse out port
            addresses[numservers as usize].port = (*buffptr as u16) << 8;
            buffptr = buffptr.offset(1);
            addresses[numservers as usize].port += *buffptr as u16;
            buffptr = buffptr.offset(1);
            addresses[numservers as usize].port = addresses[numservers as usize].port.to_be();

            // syntax check
            if *buffptr != b'\\' {
                break;
            }

            Com_DPrintf(
                common,
                &format!(
                    "server: {} ip: {}.{}.{}.{}:{}\n",
                    numservers,
                    addresses[numservers as usize].ip[0],
                    addresses[numservers as usize].ip[1],
                    addresses[numservers as usize].ip[2],
                    addresses[numservers as usize].ip[3],
                    addresses[numservers as usize].port
                ),
            );

            numservers += 1;
            if numservers >= MAX_SERVERSPERPACKET {
                break;
            }

            // parse out EOT
            if *buffptr.offset(1) == b'E'
                && *buffptr.offset(2) == b'O'
                && *buffptr.offset(3) == b'T'
            {
                break;
            }
        }
    }

    if cl.cls.masterNum == 0 {
        count = cl.cls.numglobalservers;
        max = MAX_GLOBAL_SERVERS as c_int;
    } else {
        count = cl.cls.nummplayerservers;
        max = MAX_OTHER_SERVERS as c_int;
    }

    let mut i: c_int = 0;
    while i < numservers && count < max {
        // build net address
        let server: *mut serverInfo_t = if cl.cls.masterNum == 0 {
            &mut cl.cls.globalServers[count as usize] as *mut serverInfo_t
        } else {
            &mut cl.cls.mplayerServers[count as usize] as *mut serverInfo_t
        };

        CL_InitServerInfo(server, &mut addresses[i as usize] as *mut serverAddress_t);
        // advance to next slot
        count += 1;
        i += 1;
    }

    // if getting the global list
    if cl.cls.masterNum == 0 {
        if cl.cls.numGlobalServerAddresses < MAX_GLOBAL_SERVERS as c_int {
            // if we couldn't store the servers in the main list anymore
            while i < numservers && count >= max {
                // just store the addresses in an additional list
                let slot = cl.cls.numGlobalServerAddresses as usize;
                cl.cls.numGlobalServerAddresses += 1;
                cl.cls.globalServerAddresses[slot].ip[0] = addresses[i as usize].ip[0];
                cl.cls.globalServerAddresses[slot].ip[1] = addresses[i as usize].ip[1];
                cl.cls.globalServerAddresses[slot].ip[2] = addresses[i as usize].ip[2];
                cl.cls.globalServerAddresses[slot].ip[3] = addresses[i as usize].ip[3];
                cl.cls.globalServerAddresses[slot].port = addresses[i as usize].port;
                i += 1;
            }
        }
    }

    if cl.cls.masterNum == 0 {
        cl.cls.numglobalservers = count;
        total = count + cl.cls.numGlobalServerAddresses;
    } else {
        cl.cls.nummplayerservers = count;
        total = count;
    }

    com_printf(
        common,
        &format!("{} servers parsed (total {})\n", numservers, total),
    );
}

/// `CL_CheckUserinfo` — sends a reliable userinfo update when a cvar changed.
///
/// Raven: nothing is queued before the challenge, or while paused, so the
/// reliable command buffer cannot overflow.
/// Source: `oracle/codemp/client/cl_main.cpp:2240-2255`
pub fn CL_CheckUserinfo(common: &mut Common, cl: &mut Client) {
    // don't add reliable commands when not yet connected
    if (cl.cls.state as c_int) < connstate_t::CA_CHALLENGING as c_int {
        return;
    }
    // don't overflow the reliable command buffer when paused
    if common.cvar(common.cl_paused).integer != 0 {
        return;
    }
    // send a reliable userinfo update if needed
    if common.cvar_modifiedFlags & CVAR_USERINFO != 0 {
        common.cvar_modifiedFlags &= !CVAR_USERINFO;
        let cmd = format!("userinfo \"{}\"", Cvar_InfoString(common, CVAR_USERINFO));
        CL_AddReliableCommand(cl, cmd.as_ptr() as *const c_char);
    }
}

/// `CL_SetServerInfoByAddress` — updates every browser row with that address.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2803-2830`
pub fn CL_SetServerInfoByAddress(
    common: &mut Common,
    cl: &mut Client,
    from: netadr_t,
    info: *const c_char,
    ping: c_int,
) {
    for i in 0..MAX_OTHER_SERVERS as c_int {
        let adr = cl.cls.localServers[i as usize].adr;
        if NET_CompareAdr(common, from, adr) != qfalse {
            CL_SetServerInfo(
                &mut cl.cls.localServers[i as usize] as *mut serverInfo_t,
                info,
                ping,
            );
        }
    }

    for i in 0..MAX_OTHER_SERVERS as c_int {
        let adr = cl.cls.mplayerServers[i as usize].adr;
        if NET_CompareAdr(common, from, adr) != qfalse {
            CL_SetServerInfo(
                &mut cl.cls.mplayerServers[i as usize] as *mut serverInfo_t,
                info,
                ping,
            );
        }
    }

    for i in 0..MAX_GLOBAL_SERVERS as c_int {
        let adr = cl.cls.globalServers[i as usize].adr;
        if NET_CompareAdr(common, from, adr) != qfalse {
            CL_SetServerInfo(
                &mut cl.cls.globalServers[i as usize] as *mut serverInfo_t,
                info,
                ping,
            );
        }
    }

    for i in 0..MAX_OTHER_SERVERS as c_int {
        let adr = cl.cls.favoriteServers[i as usize].adr;
        if NET_CompareAdr(common, from, adr) != qfalse {
            CL_SetServerInfo(
                &mut cl.cls.favoriteServers[i as usize] as *mut serverInfo_t,
                info,
                ping,
            );
        }
    }
}

/// `CL_ServerStatus` — the UI's poll for one server's status string.
///
/// Raven: a null address resets every request slot; a null out-string only
/// releases the slot for that address.
/// Source: `oracle/codemp/client/cl_main.cpp:3002-3058`
pub fn CL_ServerStatus(
    view: &mut EngineHostView,
    cl: &mut Client,
    serverAddress: *mut c_char,
    serverStatusString: *mut c_char,
    maxLen: c_int,
) -> c_int {
    let mut to: netadr_t = unsafe { core::mem::zeroed() };

    // if no server address then reset all server status requests
    if serverAddress.is_null() {
        for i in 0..MAX_SERVERSTATUSREQUESTS as c_int {
            cl.cl_serverStatusList[i as usize].address.port = 0;
            cl.cl_serverStatusList[i as usize].retrieved = qtrue;
        }
        return qfalse;
    }
    // get the address
    if NET_StringToAdr(serverAddress, &mut to) == qfalse {
        return qfalse;
    }
    let serverStatus = CL_GetServerStatus(view.common, cl, to);
    // if no server status string then reset the server status request for this address
    if serverStatusString.is_null() {
        unsafe {
            (*serverStatus).retrieved = qtrue;
        }
        return qfalse;
    }

    unsafe {
        // if this server status request has the same address
        let address = (*serverStatus).address;
        if NET_CompareAdr(view.common, to, address) != qfalse {
            // if we recieved an response for this server status request
            if (*serverStatus).pending == qfalse {
                let text: String = (*serverStatus)
                    .string
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u8 as char)
                    .collect();
                let dest = core::slice::from_raw_parts_mut(serverStatusString, maxLen as usize);
                Q_strncpyz(dest, &text, maxLen as usize);
                (*serverStatus).retrieved = qtrue;
                (*serverStatus).startTime = 0;
                return qtrue;
            }
            // resend the request regularly
            else if (*serverStatus).startTime
                < Com_Milliseconds(view) - view.common.cvar(cl.cl_serverStatusResendTime).integer
            {
                (*serverStatus).print = qfalse;
                (*serverStatus).pending = qtrue;
                (*serverStatus).retrieved = qfalse;
                (*serverStatus).time = 0;
                (*serverStatus).startTime = Com_Milliseconds(view);
                NET_OutOfBandPrint(view.common, netsrc_t::NS_CLIENT, to, "getstatus".to_string());
                return qfalse;
            }
        }
        // if retrieved
        else if (*serverStatus).retrieved != qfalse {
            (*serverStatus).address = to;
            (*serverStatus).print = qfalse;
            (*serverStatus).pending = qtrue;
            (*serverStatus).retrieved = qfalse;
            (*serverStatus).startTime = Com_Milliseconds(view);
            (*serverStatus).time = 0;
            NET_OutOfBandPrint(view.common, netsrc_t::NS_CLIENT, to, "getstatus".to_string());
            return qfalse;
        }
    }
    qfalse
}

/// `CL_ServerStatus_f` — the `serverstatus` console command.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3573-3606`
pub fn CL_ServerStatus_f(common: &mut Common, cl: &mut Client) {
    let mut to: netadr_t = unsafe { core::mem::zeroed() };
    let server: String;

    let size = core::mem::size_of::<netadr_t>();
    Com_Memset(&mut to as *mut netadr_t as *mut (), 0, size);

    if Cmd_Argc(common) != 2 {
        if cl.cls.state as c_int != connstate_t::CA_ACTIVE as c_int || cl.clc.demoplaying != qfalse
        {
            com_printf(common, "Not connected to a server.\n");
            com_printf(common, "Usage: serverstatus [server]\n");
            return;
        }
        server = cl
            .cls
            .servername
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();
    } else {
        server = Cmd_Argv(common, 1).to_string();
    }

    if NET_StringToAdr(server.as_ptr() as *const c_char, &mut to) == qfalse {
        return;
    }

    NET_OutOfBandPrint(common, netsrc_t::NS_CLIENT, to, "getstatus".to_string());

    let serverStatus = CL_GetServerStatus(common, cl, to);
    unsafe {
        (*serverStatus).address = to;
        (*serverStatus).print = qtrue;
        (*serverStatus).pending = qtrue;
    }
}

/// `CL_ServerInfoPacket` — accepts an `infoResponse` from a pinged server.
///
/// Raven: "make sure these types are in sync with the netnames strings in the UI".
/// Source: `oracle/codemp/client/cl_main.cpp:2837-2960`
pub fn CL_ServerInfoPacket(
    common: &mut Common,
    cl: &mut Client,
    from: netadr_t,
    msg: *mut msg_t,
) {
    let mut type_: c_int;
    let mut str: &str;
    let prot: c_int;

    let infoString = MSG_ReadString(common, msg);

    // if this isn't the correct protocol version, ignore it
    prot = atoi(&Info_ValueForKey(&infoString, "protocol"));
    if prot != PROTOCOL_VERSION {
        Com_DPrintf(
            common,
            &format!("Different protocol info packet: {}\n", infoString),
        );
        return;
    }

    // iterate servers waiting for ping response
    for i in 0..MAX_PINGREQUESTS as c_int {
        let adr = cl.cl_pinglist[i as usize].adr;
        if adr.port != 0
            && cl.cl_pinglist[i as usize].time == 0
            && NET_CompareAdr(common, from, adr) != qfalse
        {
            // calc ping time
            cl.cl_pinglist[i as usize].time =
                cl.cls.realtime - cl.cl_pinglist[i as usize].start + 1;
            let adrstr = NET_AdrToString(common, from);
            let adrstr: String = unsafe { CStr::from_ptr(adrstr) }
                .to_string_lossy()
                .into_owned();
            Com_DPrintf(
                common,
                &format!(
                    "ping time {}ms from {}\n",
                    cl.cl_pinglist[i as usize].time, adrstr
                ),
            );

            // save of info
            let destsize = cl.cl_pinglist[i as usize].info.len();
            Q_strncpyz(&mut cl.cl_pinglist[i as usize].info, &infoString, destsize);

            // tack on the net type
            // NOTE: make sure these types are in sync with the netnames strings in the UI
            let t = from.r#type as c_int;
            if t == netadrtype_t::NA_BROADCAST as c_int || t == netadrtype_t::NA_IP as c_int {
                str = "udp";
                type_ = 1;
            } else if t == netadrtype_t::NA_IPX as c_int
                || t == netadrtype_t::NA_BROADCAST_IPX as c_int
            {
                str = "ipx";
                type_ = 2;
            } else {
                str = "???";
                type_ = 0;
            }
            let mut slotinfo: String = cl.cl_pinglist[i as usize]
                .info
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8 as char)
                .collect();
            Info_SetValueForKey(&mut slotinfo, "nettype", &format!("{}", type_));
            let destsize = cl.cl_pinglist[i as usize].info.len();
            Q_strncpyz(&mut cl.cl_pinglist[i as usize].info, &slotinfo, destsize);
            let ping = cl.cl_pinglist[i as usize].time;
            CL_SetServerInfoByAddress(common, cl, from, infoString.as_ptr() as *const c_char, ping);

            return;
        }
    }

    // if not just sent a local broadcast or pinging local servers
    if cl.cls.pingUpdateSource != AS_LOCAL {
        return;
    }

    let mut i: c_int = 0;
    while i < MAX_OTHER_SERVERS as c_int {
        // empty slot
        if cl.cls.localServers[i as usize].adr.port == 0 {
            break;
        }

        // avoid duplicate
        let adr = cl.cls.localServers[i as usize].adr;
        if NET_CompareAdr(common, from, adr) != qfalse {
            return;
        }
        i += 1;
    }

    if i == MAX_OTHER_SERVERS as c_int {
        Com_DPrintf(common, "MAX_OTHER_SERVERS hit, dropping infoResponse\n");
        return;
    }

    // add this to the list
    cl.cls.numlocalservers = i + 1;
    cl.cls.localServers[i as usize].adr = from;
    cl.cls.localServers[i as usize].clients = 0;
    cl.cls.localServers[i as usize].hostName[0] = 0;
    cl.cls.localServers[i as usize].mapName[0] = 0;
    cl.cls.localServers[i as usize].maxClients = 0;
    cl.cls.localServers[i as usize].maxPing = 0;
    cl.cls.localServers[i as usize].minPing = 0;
    cl.cls.localServers[i as usize].netType = from.r#type as c_int;
    cl.cls.localServers[i as usize].needPassword = qfalse;
    cl.cls.localServers[i as usize].trueJedi = 0;
    cl.cls.localServers[i as usize].weaponDisable = 0;
    cl.cls.localServers[i as usize].forceDisable = 0;
    cl.cls.localServers[i as usize].ping = -1;
    cl.cls.localServers[i as usize].game[0] = 0;
    cl.cls.localServers[i as usize].gameType = 0;
    //	cls.localServers[i].allowAnonymous = 0;
    //	cls.localServers[i].pure = qfalse;

    let tail = MSG_ReadString(common, msg);
    let mut info = [0 as c_char; MAX_INFO_STRING as usize];
    Q_strncpyz(&mut info, &tail, MAX_INFO_STRING as usize);
    let mut text: String = info
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    if !text.is_empty() {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        let adrstr = NET_AdrToString(common, from);
        let adrstr: String = unsafe { CStr::from_ptr(adrstr) }
            .to_string_lossy()
            .into_owned();
        com_printf(common, &format!("{}: {}", adrstr, text));
    }
}

/// `CL_GetPing` — reports one ping slot's address and round-trip time.
///
/// Raven: a slot that has not answered inside `cl_maxPing` reports zero.
/// Source: `oracle/codemp/client/cl_main.cpp:3263-3299`
pub fn CL_GetPing(
    common: &mut Common,
    cl: &mut Client,
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
    pingtime: *mut c_int,
) {
    let mut time: c_int;
    let mut maxPing: c_int;

    if cl.cl_pinglist[n as usize].adr.port == 0 {
        // empty slot
        unsafe {
            *buf.offset(0) = 0;
            *pingtime = 0;
        }
        return;
    }

    let adr = cl.cl_pinglist[n as usize].adr;
    let str = NET_AdrToString(common, adr);
    let str: String = unsafe { CStr::from_ptr(str) }
        .to_string_lossy()
        .into_owned();
    let dest = unsafe { core::slice::from_raw_parts_mut(buf, buflen as usize) };
    Q_strncpyz(dest, &str, buflen as usize);

    time = cl.cl_pinglist[n as usize].time;
    if time == 0 {
        // check for timeout
        time = cl.cls.realtime - cl.cl_pinglist[n as usize].start;
        maxPing = Cvar_VariableIntegerValue(common, "cl_maxPing");
        if maxPing < 100 {
            maxPing = 100;
        }
        if time < maxPing {
            // not timed out yet
            time = 0;
        }
    }

    let info = cl.cl_pinglist[n as usize].info.as_ptr();
    let ping = cl.cl_pinglist[n as usize].time;
    CL_SetServerInfoByAddress(common, cl, adr, info, ping);

    unsafe {
        *pingtime = time;
    }
}

/// `CL_UpdateServerInfo` — refreshes the browser row behind one ping slot.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3306-3314`
pub fn CL_UpdateServerInfo(common: &mut Common, cl: &mut Client, n: c_int) {
    if cl.cl_pinglist[n as usize].adr.port == 0 {
        return;
    }

    let adr = cl.cl_pinglist[n as usize].adr;
    let info = cl.cl_pinglist[n as usize].info.as_ptr();
    let ping = cl.cl_pinglist[n as usize].time;
    CL_SetServerInfoByAddress(common, cl, adr, info, ping);
}

/// `CL_Ping_f` — pings one named server from the console.
///
/// Source: `oracle/codemp/client/cl_main.cpp:3432-3459`
pub fn CL_Ping_f(common: &mut Common, cl: &mut Client) {
    let mut to: netadr_t = unsafe { core::mem::zeroed() };
    let pingptr: *mut ping_t;
    let server: String;

    if Cmd_Argc(common) != 2 {
        com_printf(common, "usage: ping [server]\n");
        return;
    }

    let size = core::mem::size_of::<netadr_t>();
    Com_Memset(&mut to as *mut netadr_t as *mut (), 0, size);

    server = Cmd_Argv(common, 1).to_string();

    if NET_StringToAdr(server.as_ptr() as *const c_char, &mut to) == qfalse {
        return;
    }

    pingptr = CL_GetFreePing(cl);

    unsafe {
        (*pingptr).adr = to;
        (*pingptr).start = cl.cls.realtime;
        (*pingptr).time = 0;

        let adr = (*pingptr).adr;
        CL_SetServerInfoByAddress(common, cl, adr, core::ptr::null(), 0);
    }

    NET_OutOfBandPrint(common, netsrc_t::NS_CLIENT, to, "getinfo xxx".to_string());
}

/// `CL_UpdateVisiblePings_f` — pings every visible browser row that needs one.
///
/// Raven: a global row that lost its ping packet is replaced from the extra
/// address list, and "the server[i].visible flag stays untouched".
/// Source: `oracle/codemp/client/cl_main.cpp:3466-3566`
pub fn CL_UpdateVisiblePings_f(common: &mut Common, cl: &mut Client, source: c_int) -> qboolean {
    let mut slots: c_int;
    let mut buff = [0 as c_char; MAX_STRING_CHARS as usize];
    let mut pingTime: c_int = 0;
    let mut max: c_int;
    let mut status: qboolean = qfalse;

    if source < 0 || source > AS_FAVORITES {
        return qfalse;
    }

    cl.cls.pingUpdateSource = source;

    slots = CL_GetPingQueueCount(cl);
    if slots < MAX_PINGREQUESTS as c_int {
        let mut server: *mut serverInfo_t = core::ptr::null_mut();

        max = if source == AS_GLOBAL {
            MAX_GLOBAL_SERVERS as c_int
        } else {
            MAX_OTHER_SERVERS as c_int
        };
        if source == AS_LOCAL {
            server = &mut cl.cls.localServers[0] as *mut serverInfo_t;
            max = cl.cls.numlocalservers;
        } else if source == AS_MPLAYER {
            server = &mut cl.cls.mplayerServers[0] as *mut serverInfo_t;
            max = cl.cls.nummplayerservers;
        } else if source == AS_GLOBAL {
            server = &mut cl.cls.globalServers[0] as *mut serverInfo_t;
            max = cl.cls.numglobalservers;
        } else if source == AS_FAVORITES {
            server = &mut cl.cls.favoriteServers[0] as *mut serverInfo_t;
            max = cl.cls.numfavoriteservers;
        }
        for i in 0..max {
            unsafe {
                let row = server.offset(i as isize);
                if (*row).visible != qfalse {
                    if (*row).ping == -1 {
                        let mut j: c_int;

                        if slots >= MAX_PINGREQUESTS as c_int {
                            break;
                        }
                        j = 0;
                        while j < MAX_PINGREQUESTS as c_int {
                            if cl.cl_pinglist[j as usize].adr.port == 0 {
                                j += 1;
                                continue;
                            }
                            let padr = cl.cl_pinglist[j as usize].adr;
                            let radr = (*row).adr;
                            if NET_CompareAdr(common, padr, radr) != qfalse {
                                // already on the list
                                break;
                            }
                            j += 1;
                        }
                        if j >= MAX_PINGREQUESTS as c_int {
                            status = qtrue;
                            j = 0;
                            while j < MAX_PINGREQUESTS as c_int {
                                if cl.cl_pinglist[j as usize].adr.port == 0 {
                                    break;
                                }
                                j += 1;
                            }
                            cl.cl_pinglist[j as usize].adr = (*row).adr;
                            cl.cl_pinglist[j as usize].start = cl.cls.realtime;
                            cl.cl_pinglist[j as usize].time = 0;
                            let padr = cl.cl_pinglist[j as usize].adr;
                            NET_OutOfBandPrint(
                                common,
                                netsrc_t::NS_CLIENT,
                                padr,
                                "getinfo xxx".to_string(),
                            );
                            slots += 1;
                        }
                    }
                    // if the server has a ping higher than cl_maxPing or
                    // the ping packet got lost
                    else if (*row).ping == 0 {
                        // if we are updating global servers
                        if source == AS_GLOBAL {
                            //
                            if cl.cls.numGlobalServerAddresses > 0 {
                                // overwrite this server with one from the additional global servers
                                cl.cls.numGlobalServerAddresses -= 1;
                                let slot = cl.cls.numGlobalServerAddresses as usize;
                                let addr =
                                    &mut cl.cls.globalServerAddresses[slot] as *mut serverAddress_t;
                                CL_InitServerInfo(row, addr);
                                // NOTE: the server[i].visible flag stays untouched
                            }
                        }
                    }
                }
            }
        }
    }

    if slots != 0 {
        status = qtrue;
    }
    for i in 0..MAX_PINGREQUESTS as c_int {
        if cl.cl_pinglist[i as usize].adr.port == 0 {
            continue;
        }
        CL_GetPing(
            common,
            cl,
            i,
            buff.as_mut_ptr(),
            MAX_STRING_CHARS as c_int,
            &mut pingTime,
        );
        if pingTime != 0 {
            CL_ClearPing(cl, i);
            status = qtrue;
        }
    }

    status
}

/// `CL_ShutdownAll` — stops sound, cgame, ui, and the renderer for a restart.
///
/// Raven: the renderer keeps its window and context.
/// Source: `oracle/codemp/client/cl_main.cpp:657-682`
pub fn CL_ShutdownAll(view: &mut EngineHostView, cl: &mut Client) {
    // clear sounds
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_DisableSounds(view.common, snd);
    }
    // shutdown CGame
    CL_ShutdownCGame(view.common, cl);
    // shutdown UI
    CL_ShutdownUI(view.common, cl);

    // shutdown the renderer
    // PORT-NOTE(dec-59.1): the `refexport_t` table is gone, so Raven's test for
    // a bound `re.Shutdown` has no subject and the site calls `RE_Shutdown`
    // directly.
    // SAFETY: view-constructor slot, single-threaded, no other cast of the same
    // slot is live across the call.
    let re = unsafe { re_from_view(view) };
    RE_Shutdown(
        view,
        &re.cvars,
        Arc::make_mut(&mut re.sim.published),
        &mut re.img_state,
        &mut re.font,
        false, // don't destroy window or context
    );

    cl.cls.uiStarted = qfalse;
    cl.cls.cgameStarted = qfalse;
    cl.cls.rendererStarted = qfalse;
    cl.cls.soundRegistered = qfalse;
}

/// `CL_Disconnect` — leaves the current server and wipes the connection state.
///
/// Raven: the disconnect command goes out three times in case one is dropped.
/// Source: `oracle/codemp/client/cl_main.cpp:837-901`
pub fn CL_Disconnect(view: &mut EngineHostView, cl: &mut Client, showMainMenu: qboolean) {
    if view.common.cvar(view.common.com_cl_running).integer == 0 {
        return;
    }

    // shutting down the client so enter full screen ui mode
    Cvar_Set(view, "r_uiFullScreen", "1");

    if cl.clc.demorecording != qfalse {
        CL_StopRecord_f(view.common, cl);
    }

    if cl.clc.download != 0 {
        FS_FCloseFile(view.common, cl.clc.download);
        cl.clc.download = 0;
    }
    cl.clc.downloadTempName[0] = 0;
    cl.clc.downloadName[0] = 0;
    Cvar_Set(view, "cl_downloadName", "");

    if cl.clc.demofile != 0 {
        FS_FCloseFile(view.common, cl.clc.demofile);
        cl.clc.demofile = 0;
    }

    if !cl.uivm.is_null() && showMainMenu != qfalse {
        VM_Call(
            view.common,
            cl.uivm,
            MpUiExport::UI_SET_ACTIVE_MENU as c_int,
            &[UIMENU_NONE as isize],
        );
    }

    SCR_StopCinematic(view, cl);
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    S_ClearSoundBuffer(unsafe { snd_from_view(view) });

    // send a disconnect message to the server
    // send it a few times in case one is dropped
    if cl.cls.state as c_int >= connstate_t::CA_CONNECTED as c_int {
        let cmd = format!("disconnect");
        CL_AddReliableCommand(cl, cmd.as_ptr() as *const c_char);
        CL_WritePacket(view, cl);
        CL_WritePacket(view, cl);
        CL_WritePacket(view, cl);
    }

    CL_ClearState(cl);

    // wipe the client connection
    let size = core::mem::size_of_val(&*cl.clc);
    Com_Memset(&mut *cl.clc as *mut _ as *mut (), 0, size);

    cl.cls.state = connstate_t::CA_DISCONNECTED;

    // not connected to a pure server anymore
    cl.cl_connectedToPureServer = qfalse;
    cl.cl_connectedGAME = 0;
    cl.cl_connectedCGAME = 0;
    cl.cl_connectedUI = 0;
}

/// `CL_Disconnect_f` — the `disconnect` command; unwinds through `Com_Error`.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1111-1117`
pub fn CL_Disconnect_f(view: &mut EngineHostView, cl: &mut Client) {
    SCR_StopCinematic(view, cl);
    Cvar_Set(view, "ui_singlePlayerActive", "0");
    if cl.cls.state as c_int != connstate_t::CA_DISCONNECTED as c_int
        && cl.cls.state as c_int != connstate_t::CA_CINEMATIC as c_int
    {
        com_error(
            errorParm_t::ERR_DISCONNECT,
            "Disconnected from server".to_string(),
        );
    }
}

/// `CL_DemoCompleted` — ends demo playback and returns to the main menu.
///
/// Raven: "This code will bring us back to the main menu after a demo is
/// finished playing instead."
/// Source: `oracle/codemp/client/cl_main.cpp:468-491`
pub fn CL_DemoCompleted(view: &mut EngineHostView, cl: &mut Client) {
    if view.common.cvar(cl.cl_timedemo).integer != 0 {
        let time: c_int;

        time = sys_milliseconds(view.common) - cl.clc.timeDemoStart;
        if time > 0 {
            let frames = cl.clc.timeDemoFrames;
            com_printf(
                view.common,
                &format!(
                    "{} frames, {:.1} seconds: {:.1} fps\n",
                    frames,
                    time as f64 / 1000.0,
                    frames as f64 * 1000.0 / time as f64
                ),
            );
        }
    }

    /*	CL_Disconnect( qtrue );
        CL_NextDemo();
    */

    //rww - The above code seems to just stick you in a no-menu state and you can't do anything there.
    //I'm not sure why it ever worked in TA, but whatever. This code will bring us back to the main menu
    //after a demo is finished playing instead.
    CL_Disconnect_f(view, cl);
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StopAllSounds(view.common, snd);
    }
    VM_Call(
        view.common,
        cl.uivm,
        MpUiExport::UI_SET_ACTIVE_MENU as c_int,
        &[UIMENU_MAIN as isize],
    );

    CL_NextDemo(view);
}

/// `CL_Connect_f` — the `connect` command; resolves and starts the handshake.
///
/// Raven: a local server is always killed first, even when we join localhost.
/// Source: `oracle/codemp/client/cl_main.cpp:1141-1209`
pub fn CL_Connect_f(view: &mut EngineHostView, cl: &mut Client) {
    let server: String;

    if Cvar_VariableValue(view.common, "fs_restrict") == 0.0 && Sys_CheckCD() == qfalse {
        let msg = SE_GetString(view, "CON_TEXT_NEED_CD");
        com_error(errorParm_t::ERR_NEED_CD, msg); //"Game CD not in drive" );
    }

    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "usage: connect [server]\n");
        return;
    }

    Cvar_Set(view, "ui_singlePlayerActive", "0");

    // fire a message off to the motd server
    CL_RequestMotd(view, cl);

    // clear any previous "server full" type messages
    cl.clc.serverMessage[0] = 0;

    server = Cmd_Argv(view.common, 1).to_string();

    if view.common.cvar(view.common.com_sv_running).integer != 0 && server == "localhost" {
        // if running a local server, kill it
        SV_Shutdown(view, "Server quit\n");
    }

    // make sure a local server is killed
    Cvar_Set(view, "sv_killserver", "1");
    SV_Frame(view, 0);

    CL_Disconnect(view, cl, qtrue);
    Con_Close(view.common, cl);

    /* MrE: 2000-09-13: now called in CL_DownloadsComplete
    CL_FlushMemory( );
    */

    let destsize = cl.cls.servername.len();
    Q_strncpyz(&mut cl.cls.servername, &server, destsize);

    let servername: String = cl
        .cls
        .servername
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    if NET_StringToAdr(
        servername.as_ptr() as *const c_char,
        &mut cl.clc.serverAddress,
    ) == qfalse
    {
        com_printf(view.common, "Bad server address\n");
        cl.cls.state = connstate_t::CA_DISCONNECTED;
        return;
    }
    if cl.clc.serverAddress.port == 0 {
        cl.clc.serverAddress.port = (PORT_SERVER as u16).to_be();
    }
    let adr = cl.clc.serverAddress;
    com_printf(
        view.common,
        &format!(
            "{} resolved to {}.{}.{}.{}:{}\n",
            servername,
            adr.ip[0],
            adr.ip[1],
            adr.ip[2],
            adr.ip[3],
            adr.port.to_be()
        ),
    );

    // if we aren't playing on a lan, we need to authenticate
    // with the cd key
    if NET_IsLocalAddress(adr) != qfalse {
        cl.cls.state = connstate_t::CA_CHALLENGING;
    } else {
        cl.cls.state = connstate_t::CA_CONNECTING;
    }

    cl.cls.keyCatchers = 0;
    cl.clc.connectTime = -99999; // CL_CheckForResend() will fire immediately
    cl.clc.connectPacketCount = 0;

    // server connection string
    Cvar_Set(view, "cl_currentServerAddress", &server);
}

/// `CL_DisconnectPacket` — honors an unsolicited server disconnect.
///
/// Raven: a packet inside the last three seconds of traffic might be a spoof,
/// so it is ignored.
/// Source: `oracle/codemp/client/cl_main.cpp:1738-1762`
pub fn CL_DisconnectPacket(view: &mut EngineHostView, cl: &mut Client, from: netadr_t) {
    if (cl.cls.state as c_int) < connstate_t::CA_AUTHORIZING as c_int {
        return;
    }

    // if not from our server, ignore it
    let remote = cl.clc.netchan.remoteAddress;
    if NET_CompareAdr(view.common, from, remote) == qfalse {
        return;
    }

    // if we have received packets within three seconds, ignore it
    // (it might be a malicious spoof)
    if cl.cls.realtime - cl.clc.lastPacketTime < 3000 {
        return;
    }

    // drop the connection (Raven: a connection-dropped dialog is unimplemented)
    com_printf(view.common, "Server disconnected for unknown reason\n");

    CL_Disconnect(view, cl, qtrue);
}

/// `CL_Shutdown` — tears the whole client down and unregisters its commands.
///
/// Raven: `CL_ShutdownRef` runs before `CL_ShutdownAll` so the images get
/// dumped inside `RE_Shutdown`.
/// Source: `oracle/codemp/client/cl_main.cpp:2719-2774`
pub fn CL_Shutdown(view: &mut EngineHostView, cl: &mut Client) {
    //Com_Printf( "----- CL_Shutdown -----\n" );

    if cl.recursive != qfalse {
        print!("recursive CL_Shutdown shutdown\n");
        return;
    }
    cl.recursive = qtrue;

    // Raven deletes `G2VertSpaceClient` here. The heap is dropped by the ghoul2
    // design, so there is nothing to free.

    CL_Disconnect(view, cl, qtrue);

    CL_ShutdownRef(view, cl); //must be before shutdown all so the images get dumped in RE_Shutdown

    // RJ: added the shutdown all to close down the cgame (to free up some memory, such as in the fx system)
    CL_ShutdownAll(view, cl);

    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    S_Shutdown(view, snd);
    //CL_ShutdownUI();

    Cmd_RemoveCommand(view.common, "cmd");
    Cmd_RemoveCommand(view.common, "configstrings");
    Cmd_RemoveCommand(view.common, "userinfo");
    Cmd_RemoveCommand(view.common, "snd_restart");
    Cmd_RemoveCommand(view.common, "vid_restart");
    Cmd_RemoveCommand(view.common, "disconnect");
    Cmd_RemoveCommand(view.common, "record");
    Cmd_RemoveCommand(view.common, "demo");
    Cmd_RemoveCommand(view.common, "cinematic");
    Cmd_RemoveCommand(view.common, "stoprecord");
    Cmd_RemoveCommand(view.common, "connect");
    Cmd_RemoveCommand(view.common, "localservers");
    Cmd_RemoveCommand(view.common, "globalservers");
    Cmd_RemoveCommand(view.common, "rcon");
    Cmd_RemoveCommand(view.common, "ping");
    Cmd_RemoveCommand(view.common, "serverstatus");
    Cmd_RemoveCommand(view.common, "showip");
    Cmd_RemoveCommand(view.common, "model");
    Cmd_RemoveCommand(view.common, "forcepowers");

    Cvar_Set(view, "cl_running", "0");

    cl.recursive = qfalse;

    let size = core::mem::size_of_val(&*cl.cls);
    Com_Memset(&mut *cl.cls as *mut _ as *mut (), 0, size);

    //Com_Printf( "-----------------------\n" );
}

/// `CL_ConnectionlessPacket` — dispatches every out-of-band server reply.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2028-2141`
pub fn CL_ConnectionlessPacket(
    view: &mut EngineHostView,
    cl: &mut Client,
    from: netadr_t,
    msg: *mut msg_t,
) {
    MSG_BeginReadingOOB(msg);
    MSG_ReadLong(view.common, msg); // skip the -1

    let s = MSG_ReadStringLine(view.common, msg);

    Cmd_TokenizeString(view.common, &s);

    let c = Cmd_Argv(view.common, 0).to_string();

    let adrstr = NET_AdrToString(view.common, from);
    let adrstr: String = unsafe { CStr::from_ptr(adrstr) }
        .to_string_lossy()
        .into_owned();
    Com_DPrintf(view.common, &format!("CL packet {}: {}\n", adrstr, c));

    // challenge from the server we are connecting to
    if Q_stricmp(&c, "challengeResponse") == 0 {
        if cl.cls.state as c_int != connstate_t::CA_CONNECTING as c_int {
            com_printf(
                view.common,
                "Unwanted challenge response received.  Ignored.\n",
            );
        } else {
            // start sending challenge repsonse instead of challenge request packets
            cl.clc.challenge = atoi(Cmd_Argv(view.common, 1));
            cl.cls.state = connstate_t::CA_CHALLENGING;
            cl.clc.connectPacketCount = 0;
            cl.clc.connectTime = -99999;

            // take this address as the new server address.  This allows
            // a server proxy to hand off connections to multiple servers
            cl.clc.serverAddress = from;
            let challenge = cl.clc.challenge;
            Com_DPrintf(view.common, &format!("challengeResponse: {}\n", challenge));
        }
        return;
    }

    // server connection
    if Q_stricmp(&c, "connectResponse") == 0 {
        if cl.cls.state as c_int >= connstate_t::CA_CONNECTED as c_int {
            com_printf(view.common, "Dup connect received.  Ignored.\n");
            return;
        }
        if cl.cls.state as c_int != connstate_t::CA_CHALLENGING as c_int {
            com_printf(
                view.common,
                "connectResponse packet while not connecting.  Ignored.\n",
            );
            return;
        }
        let serverAddress = cl.clc.serverAddress;
        if NET_CompareBaseAdr(view.common, from, serverAddress) == qfalse {
            com_printf(
                view.common,
                "connectResponse from a different address.  Ignored.\n",
            );
            let a = NET_AdrToString(view.common, from);
            let a: String = unsafe { CStr::from_ptr(a) }.to_string_lossy().into_owned();
            let b = NET_AdrToString(view.common, serverAddress);
            let b: String = unsafe { CStr::from_ptr(b) }.to_string_lossy().into_owned();
            com_printf(view.common, &format!("{} should have been {}\n", a, b));
            return;
        }
        let qport = Cvar_VariableValue(view.common, "net_qport") as c_int;
        Netchan_Setup(netsrc_t::NS_CLIENT, &mut cl.clc.netchan, from, qport);
        cl.cls.state = connstate_t::CA_CONNECTED;
        cl.clc.lastPacketSentTime = -9999; // send first packet immediately
        return;
    }

    // server responding to an info broadcast
    if Q_stricmp(&c, "infoResponse") == 0 {
        CL_ServerInfoPacket(view.common, cl, from, msg);
        return;
    }

    // server responding to a get playerlist
    if Q_stricmp(&c, "statusResponse") == 0 {
        CL_ServerStatusResponse(view, cl, from, msg);
        return;
    }

    // a disconnect message from the server, which will happen if the server
    // dropped the connection but it is still getting packets from us
    if Q_stricmp(&c, "disconnect") == 0 {
        CL_DisconnectPacket(view, cl, from);
        return;
    }

    // echo request from server
    if Q_stricmp(&c, "echo") == 0 {
        let arg = Cmd_Argv(view.common, 1).to_string();
        NET_OutOfBandPrint(view.common, netsrc_t::NS_CLIENT, from, arg);
        return;
    }

    // cd check
    if Q_stricmp(&c, "keyAuthorize") == 0 {
        // we don't use these now, so dump them on the floor
        return;
    }

    // global MOTD from id
    if Q_stricmp(&c, "motd") == 0 {
        CL_MotdPacket(view, cl, from);
        return;
    }

    // echo request from server
    if Q_stricmp(&c, "print") == 0 {
        let mut sTemp = [0 as c_char; MAX_STRINGED_SV_STRING as usize];

        let body = MSG_ReadString(view.common, msg);
        let body_c: Vec<c_char> = body
            .bytes()
            .map(|b| b as c_char)
            .chain(core::iter::once(0))
            .collect();
        CL_CheckSVStringEdRef(view, sTemp.as_mut_ptr(), body_c.as_ptr());
        let text: String = sTemp
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();
        let destsize = cl.clc.serverMessage.len();
        Q_strncpyz(&mut cl.clc.serverMessage, &text, destsize);
        com_printf(view.common, &format!("{}", text));
        return;
    }

    // echo request from server
    //	if ( !Q_stricmp(c, "getserversResponse\\") ) {
    if Q_strncmp(&c, "getserversResponse", 18) == 0 {
        CL_ServersResponsePacket(view.common, cl, from, msg);
        return;
    }

    Com_DPrintf(view.common, "Unknown connectionless packet command.\n");
}

/// `CL_MapLoading` — puts the client into the connect screen for a local map.
///
/// Raven: an existing localhost connection is kept so the connect screen draws.
/// Source: `oracle/codemp/client/cl_main.cpp:778-811`
pub fn CL_MapLoading(view: &mut EngineHostView, cl: &mut Client) {
    if view.common.cvar(view.common.com_cl_running).integer == 0 {
        return;
    }

    // Set this to localhost.
    Cvar_Set(view, "cl_currentServerAddress", "Localhost");

    Con_Close(view.common, cl);
    cl.cls.keyCatchers = 0;

    let servername: String = cl
        .cls
        .servername
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();

    // if we are already connected to the local host, stay connected
    if cl.cls.state as c_int >= connstate_t::CA_CONNECTED as c_int
        && Q_stricmp(&servername, "localhost") == 0
    {
        cl.cls.state = connstate_t::CA_CONNECTED; // so the connect screen is drawn
        let size = core::mem::size_of_val(&cl.cls.updateInfoString);
        Com_Memset(cl.cls.updateInfoString.as_mut_ptr() as *mut (), 0, size);
        let size = core::mem::size_of_val(&cl.clc.serverMessage);
        Com_Memset(cl.clc.serverMessage.as_mut_ptr() as *mut (), 0, size);
        let size = core::mem::size_of_val(&cl.cl.gameState);
        Com_Memset(&mut cl.cl.gameState as *mut _ as *mut (), 0, size);
        cl.clc.lastPacketSentTime = -9999;
        SCR_UpdateScreen(view, cl);
    } else {
        // clear nextmap so the cinematic shutdown doesn't execute it
        Cvar_Set(view, "nextmap", "");
        CL_Disconnect(view, cl, qtrue);
        let destsize = cl.cls.servername.len();
        Q_strncpyz(&mut cl.cls.servername, "localhost", destsize);
        cl.cls.state = connstate_t::CA_CHALLENGING; // so the connect screen is drawn
        cl.cls.keyCatchers = 0;
        SCR_UpdateScreen(view, cl);
        cl.clc.connectTime = -RETRANSMIT_TIMEOUT;
        NET_StringToAdr(
            "localhost\0".as_ptr() as *const c_char,
            &mut cl.clc.serverAddress,
        );
        // we don't need a challenge on the localhost

        CL_CheckForResend(view.common, cl);
    }
}

/// `CL_Snd_Restart_f` — the `snd_restart` command.
///
/// Raven: `S_Shutdown` already frees the sfx memory and the dynamic music.
/// Source: `oracle/codemp/client/cl_main.cpp:1378-1392`
pub fn CL_Snd_Restart_f(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    S_Shutdown(view, snd);
    S_Init(view, snd);

    //	S_FreeAllSFXMem();			// These two removed by BTO (VV)
    //	S_UnCacheDynamicMusic();	// S_Shutdown() already does this!

    //	CL_Vid_Restart_f();

    snd.s_soundMuted = false; // we can play again

    S_RestartMusic(view, snd);
}

/// `CL_StartHunkUsers` — brings the renderer, sound, and UI back up.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2445-2473`
pub fn CL_StartHunkUsers(view: &mut EngineHostView, cl: &mut Client) {
    if view.common.cvar(view.common.com_cl_running).integer == 0 {
        return;
    }

    if cl.cls.rendererStarted == qfalse {
        cl.cls.rendererStarted = qtrue;
        CL_InitRenderer(view, cl);
    }

    if cl.cls.soundStarted == qfalse {
        cl.cls.soundStarted = qtrue;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_Init(view, snd);
    }

    if cl.cls.soundRegistered == qfalse {
        cl.cls.soundRegistered = qtrue;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_BeginRegistration(view, snd);
    }

    if cl.cls.uiStarted == qfalse {
        cl.cls.uiStarted = qtrue;
        CL_InitUI(view, cl);
    }
}

/// `CL_FlushMemory` — clears the hunk between levels and restarts the client.
///
/// Raven: with a server running, only the client part of the hunk is cleared.
/// Source: `oracle/codemp/client/cl_main.cpp:734-767`
pub fn CL_FlushMemory(view: &mut EngineHostView, cl: &mut Client) {
    // shutdown all the client stuff
    CL_ShutdownAll(view, cl);

    // if not running a server clear the whole hunk
    if view.common.cvar(view.common.com_sv_running).integer == 0 {
        // clear collision map data
        CM_ClearMap(&mut view.cm, &mut view.rmg);
        // clear the whole hunk
        Hunk_Clear(view);

        //clear everything else to avoid fragmentation
    } else {
        // clear all the client data on the hunk
        Hunk_ClearToMark(view.common);
    }

    CL_StartHunkUsers(view, cl);
}

/// `CL_Vid_Restart_f` — the `vid_restart` command; restarts the renderer stack.
///
/// Raven: selecting a mod from the menu only issues a vid_restart, so the net
/// overrides are re-checked here.
/// Source: `oracle/codemp/client/cl_main.cpp:1311-1366`
pub fn CL_Vid_Restart_f(view: &mut EngineHostView, cl: &mut Client) {
    //rww - sort of nasty, but when a user selects a mod
    //from the menu all it does is a vid_restart, so we
    //have to check for new net overrides for the mod then.
    cl.g_nOverrideChecked = false;

    // don't let them loop during the restart
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StopAllSounds(view.common, snd);
    }
    // shutdown the UI
    CL_ShutdownUI(view.common, cl);
    // shutdown the CGame
    CL_ShutdownCGame(view.common, cl);
    // shutdown the renderer and clear the renderer interface
    CL_ShutdownRef(view, cl);
    // client is no longer pure untill new checksums are sent
    CL_ResetPureClientAtServer(cl);
    // clear pak references
    FS_ClearPakReferences(view.common, FS_UI_REF | FS_CGAME_REF);
    // reinitialize the filesystem if the game directory or checksum has changed
    FS_ConditionalRestart(view, cl.clc.checksumFeed);

    cl.cls.rendererStarted = qfalse;
    cl.cls.uiStarted = qfalse;
    cl.cls.cgameStarted = qfalse;
    cl.cls.soundRegistered = qfalse;

    // unpause so the cgame definately gets a snapshot and renders a frame
    Cvar_Set(view, "cl_paused", "0");

    // if not running a server clear the whole hunk
    if view.common.cvar(view.common.com_sv_running).integer == 0 {
        CM_ClearMap(&mut view.cm, &mut view.rmg);
        // clear the whole hunk
        Hunk_Clear(view);
    } else {
        // clear all the client data on the hunk
        Hunk_ClearToMark(view.common);
    }

    // initialize the renderer interface
    CL_InitRef(view);

    // startup all the client stuff
    CL_StartHunkUsers(view, cl);

    // start the cgame if connected
    if cl.cls.state as c_int > connstate_t::CA_CONNECTED as c_int
        && cl.cls.state as c_int != connstate_t::CA_CINEMATIC as c_int
    {
        cl.cls.cgameStarted = qtrue;
        CL_InitCGame(view, cl);
        // send pure checksums
        CL_SendPureChecksums(view.common, cl);
    }
}

/// `CL_DownloadsComplete` — leaves the download stage and loads the map.
///
/// Raven: sending `donedl` requests a new gamestate, so nothing loads yet.
/// Source: `oracle/codemp/client/cl_main.cpp:1460-1509`
pub fn CL_DownloadsComplete(view: &mut EngineHostView, cl: &mut Client) {
    // if we downloaded files we need to restart the file system
    if cl.clc.downloadRestart != qfalse {
        cl.clc.downloadRestart = qfalse;

        FS_Restart(view, cl.clc.checksumFeed); // We possibly downloaded a pak, restart the file system to load it

        // inform the server so we get new gamestate info
        let cmd = format!("donedl");
        CL_AddReliableCommand(cl, cmd.as_ptr() as *const c_char);

        // by sending the donenl command we request a new gamestate
        // so we don't want to load stuff yet
        return;
    }

    // let the client game init and load data
    cl.cls.state = connstate_t::CA_LOADING;

    // Pump the loop, this may change gamestate!
    Com_EventLoop(view);

    // if the gamestate was changed by calling Com_EventLoop
    // then we loaded everything already and we don't want to do it again.
    if cl.cls.state as c_int != connstate_t::CA_LOADING as c_int {
        return;
    }

    // starting to load a map so we get out of full screen ui mode
    Cvar_Set(view, "r_uiFullScreen", "0");

    // flush client memory and start loading stuff
    // this will also (re)load the UI
    // if this is a local client then only the client part of the hunk
    // will be cleared, note that this is done after the hunk mark has been set
    //
    // Demo referee seam (`cl_referee.rs`): `CL_FlushMemory` restarts the sound
    // stack and the renderer and ui stacks. The sound stack landed with gh#24
    // and gh#25; the platform shell (gh#22) has not. The headless rig keeps the
    // stack it booted with, and this gate goes away when that lane lands.
    if !ref_headless(cl) {
        CL_FlushMemory(view, cl);
    }

    // initialize the CGame
    cl.cls.cgameStarted = qtrue;
    CL_InitCGame(view, cl);

    // set pure checksums
    CL_SendPureChecksums(view.common, cl);

    CL_WritePacket(view, cl);
    CL_WritePacket(view, cl);
    CL_WritePacket(view, cl);
}

/// `CL_NextDownload` — starts the next queued download, or finishes the stage.
///
/// Raven: the list format is `@remotename@localname@remotename@localname`.
/// Source: `oracle/codemp/client/cl_main.cpp:1551-1589`
pub fn CL_NextDownload(view: &mut EngineHostView, cl: &mut Client) {
    // We are looking to start a download here
    if cl.clc.downloadList[0] != 0 {
        let list: String = cl
            .clc
            .downloadList
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();

        // format is:
        //  @remotename@localname@remotename@localname, etc.

        let body = if list.starts_with('@') {
            &list[1..]
        } else {
            &list[..]
        };

        let first = match body.find('@') {
            None => {
                CL_DownloadsComplete(view, cl);
                return;
            }
            Some(x) => x,
        };

        let remoteName = body[..first].to_string();
        let rest = &body[first + 1..];
        let (localName, tail) = match rest.find('@') {
            Some(x) => (rest[..x].to_string(), rest[x + 1..].to_string()),
            None => (rest.to_string(), String::new()),
        };

        let localNameC: Vec<c_char> = localName
            .bytes()
            .map(|b| b as c_char)
            .chain(core::iter::once(0))
            .collect();
        let remoteNameC: Vec<c_char> = remoteName
            .bytes()
            .map(|b| b as c_char)
            .chain(core::iter::once(0))
            .collect();
        CL_BeginDownload(view, cl, localNameC.as_ptr(), remoteNameC.as_ptr());

        cl.clc.downloadRestart = qtrue;

        // move over the rest
        let destsize = cl.clc.downloadList.len();
        Q_strncpyz(&mut cl.clc.downloadList, &tail, destsize);

        return;
    }

    CL_DownloadsComplete(view, cl);
}

/// `CL_InitDownloads` — compares our paks with the server's before loading.
///
/// Raven: with autodownload off we still warn about the referenced files we do
/// not have.
/// Source: `oracle/codemp/client/cl_main.cpp:1601-1632`
pub fn CL_InitDownloads(view: &mut EngineHostView, cl: &mut Client) {
    let mut missingfiles = [0 as c_char; 1024];

    if view.common.cvar(cl.cl_allowDownload).integer == 0 {
        // autodownload is disabled on the client
        // but it's possible that some referenced files on the server are missing
        if FS_ComparePaks(
            view.common,
            missingfiles.as_mut_ptr(),
            missingfiles.len() as c_int,
            qfalse,
        ) != qfalse
        {
            // NOTE TTimo I would rather have that printed as a modal message box
            //   but at this point while joining the game we don't know wether we will successfully join or not
            let missing: String = missingfiles
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8 as char)
                .collect();
            com_printf(
                view.common,
                &format!(
                    "\nWARNING: You are missing some files referenced by the server:\n{}You might not be able to join the game\nGo to the setting menu to turn on autodownload, or get the file elsewhere\n\n",
                    missing
                ),
            );
        }
    } else {
        let capacity = cl.clc.downloadList.len() as c_int;
        let listptr = cl.clc.downloadList.as_mut_ptr();
        if FS_ComparePaks(view.common, listptr, capacity, qtrue) != qfalse {
            let list: String = cl
                .clc
                .downloadList
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8 as char)
                .collect();
            com_printf(view.common, &format!("Need paks: {}\n", list));

            if cl.clc.downloadList[0] != 0 {
                // if autodownloading is not enabled on the server
                cl.cls.state = connstate_t::CA_CONNECTED;
                CL_NextDownload(view, cl);
                return;
            }
        }
    }
    CL_DownloadsComplete(view, cl);
}

/// `CL_ReadDemoMessage` — reads and parses one message out of the demo file.
///
/// Source: `oracle/codemp/client/cl_main.cpp:498-544`
pub fn CL_ReadDemoMessage(view: &mut EngineHostView, cl: &mut Client) {
    let mut r: c_int;
    let mut buf: msg_t = unsafe { core::mem::zeroed() };
    let mut bufData = [0u8; MAX_MSGLEN as usize];
    let mut s: c_int = 0;

    if cl.clc.demofile == 0 {
        CL_DemoCompleted(view, cl);
        return;
    }

    // get the sequence number
    r = FS_Read(
        view.common,
        &mut s as *mut c_int as *mut (),
        4,
        cl.clc.demofile,
    );
    if r != 4 {
        CL_DemoCompleted(view, cl);
        return;
    }
    cl.clc.serverMessageSequence = s.to_le();

    // init the message
    MSG_Init(view, &mut buf, bufData.as_mut_ptr(), bufData.len() as c_int);

    // get the length
    r = FS_Read(
        view.common,
        &mut buf.cursize as *mut i32 as *mut (),
        4,
        cl.clc.demofile,
    );
    if r != 4 {
        CL_DemoCompleted(view, cl);
        return;
    }
    buf.cursize = buf.cursize.to_le();
    if buf.cursize == -1 {
        CL_DemoCompleted(view, cl);
        return;
    }
    if buf.cursize > buf.maxsize {
        com_error(
            errorParm_t::ERR_DROP,
            "CL_ReadDemoMessage: demoMsglen > MAX_MSGLEN".to_string(),
        );
    }
    r = FS_Read(
        view.common,
        buf.data as *mut (),
        buf.cursize,
        cl.clc.demofile,
    );
    if r != buf.cursize {
        com_printf(view.common, "Demo file was truncated.\n");
        CL_DemoCompleted(view, cl);
        return;
    }

    cl.clc.lastPacketTime = cl.cls.realtime;
    buf.readcount = 0;
    CL_ParseServerMessage(view, cl, &mut buf);
}

/// `CL_PacketEvent` — the client's inbound packet entry point.
///
/// Raven: a demo message may only be saved after the frame is parsed.
/// Source: `oracle/codemp/client/cl_main.cpp:2151-2204`
pub fn CL_PacketEvent(
    view: &mut EngineHostView,
    cl: &mut Client,
    from: netadr_t,
    msg: *mut msg_t,
) {
    let headerBytes: c_int;

    cl.clc.lastPacketTime = cl.cls.realtime;

    if unsafe { (*msg).cursize } >= 4 && unsafe { *((*msg).data as *const c_int) } == -1 {
        CL_ConnectionlessPacket(view, cl, from, msg);
        return;
    }

    if (cl.cls.state as c_int) < connstate_t::CA_CONNECTED as c_int {
        return; // can't be a valid sequenced packet
    }

    if unsafe { (*msg).cursize } < 4 {
        let adrstr = NET_AdrToString(view.common, from);
        let adrstr: String = unsafe { CStr::from_ptr(adrstr) }
            .to_string_lossy()
            .into_owned();
        com_printf(view.common, &format!("{}: Runt packet\n", adrstr));
        return;
    }

    //
    // packet from server
    //
    let remote = cl.clc.netchan.remoteAddress;
    if NET_CompareAdr(view.common, from, remote) == qfalse {
        let adrstr = NET_AdrToString(view.common, from);
        let adrstr: String = unsafe { CStr::from_ptr(adrstr) }
            .to_string_lossy()
            .into_owned();
        Com_DPrintf(
            view.common,
            &format!("{}:sequenced packet without connection\n", adrstr),
        );
        // Raven asks here whether to send a client disconnect. It does not.
        return;
    }

    // The netchan lives inside `cl`, and the callee takes `cl` as well, so the
    // site hands the channel over as a raw pointer.
    let chan = &mut cl.clc.netchan as *mut netchan_t;
    if CL_Netchan_Process(view.common, cl, chan, msg) == qfalse {
        return; // out of order, duplicated, etc
    }

    // the header is different lengths for reliable and unreliable messages
    headerBytes = unsafe { (*msg).readcount };

    // track the last message received so it can be returned in
    // client messages, allowing the server to detect a dropped
    // gamestate
    cl.clc.serverMessageSequence = unsafe { *((*msg).data as *const c_int) }.to_le();

    cl.clc.lastPacketTime = cl.cls.realtime;
    CL_ParseServerMessage(view, cl, msg);

    //
    // we don't know if it is ok to save a demo message until
    // after we have parsed the frame
    //
    if cl.clc.demorecording != qfalse && cl.clc.demowaiting == qfalse {
        CL_WriteDemoMessage(view.common, cl, msg, headerBytes);
    }
}

/// `CL_PlayDemo_f` — the `demo` command; opens a demo and primes playback.
///
/// Raven: the first snapshot is skipped this frame so the gamestate load does
/// not cause a time skip.
/// Source: `oracle/codemp/client/cl_main.cpp:554-608`
pub fn CL_PlayDemo_f(view: &mut EngineHostView, cl: &mut Client) {
    let mut name = [0 as c_char; MAX_OSPATH as usize];
    let mut extension = [0 as c_char; 32];

    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "playdemo <demoname>\n");
        return;
    }

    // make sure a local server is killed
    Cvar_Set(view, "sv_killserver", "1");

    CL_Disconnect(view, cl, qtrue);

    /* MrE: 2000-09-13: now called in CL_DownloadsComplete
    CL_FlushMemory( );
    */

    // open the demo file
    let arg = Cmd_Argv(view.common, 1).to_string();
    Com_sprintf(
        extension.as_mut_ptr(),
        extension.len() as c_int,
        &format!(".dm_{}", PROTOCOL_VERSION),
    );
    let ext: String = extension
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    let tail = if arg.len() >= ext.len() {
        arg[arg.len() - ext.len()..].to_string()
    } else {
        arg.clone()
    };
    if Q_stricmp(&tail, &ext) == 0 {
        Com_sprintf(
            name.as_mut_ptr(),
            name.len() as c_int,
            &format!("demos/{}", arg),
        );
    } else {
        Com_sprintf(
            name.as_mut_ptr(),
            name.len() as c_int,
            &format!("demos/{}.dm_{}", arg, PROTOCOL_VERSION),
        );
    }

    let name_str: String = name
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect();
    FS_FOpenFileRead(view, &name_str, &mut cl.clc.demofile, true);
    if cl.clc.demofile == 0 {
        if Q_stricmp(&arg, "(null)") == 0 {
            let msg = SE_GetString(view, "CON_TEXT_NO_DEMO_SELECTED");
            com_error(errorParm_t::ERR_DROP, msg);
        } else {
            com_error(errorParm_t::ERR_DROP, format!("couldn't open {}", name_str));
        }
        return;
    }
    let arg1 = Cmd_Argv(view.common, 1).to_string();
    let destsize = cl.clc.demoName.len();
    Q_strncpyz(&mut cl.clc.demoName, &arg1, destsize);

    Con_Close(view.common, cl);

    cl.cls.state = connstate_t::CA_CONNECTED;
    cl.clc.demoplaying = qtrue;
    let destsize = cl.cls.servername.len();
    Q_strncpyz(&mut cl.cls.servername, &arg1, destsize);

    // read demo messages until connected
    while cl.cls.state as c_int >= connstate_t::CA_CONNECTED as c_int
        && (cl.cls.state as c_int) < connstate_t::CA_PRIMED as c_int
    {
        CL_ReadDemoMessage(view, cl);
    }
    // don't get the first snapshot this frame, to prevent the long
    // time from the gamestate load from messing causing a time skip
    cl.clc.firstDemoFrameSkipped = qfalse;
}

/// `CL_Frame` — one client tick: input, timeout, send, screen, sound.
///
/// Raven: `SE_CheckForLanguageUpdates` costs nothing unless the language
/// changed, and then it reloads the strings.
/// Source: `oracle/codemp/client/cl_main.cpp:2268-2374`
pub fn CL_Frame(view: &mut EngineHostView, cl: &mut Client, msec: c_int) {
    let mut msec = msec;

    if view.common.cvar(view.common.com_cl_running).integer == 0 {
        return;
    }

    // Raven `SE_CheckForLanguageUpdates` has no view-level wrapper, so the site
    // lifts the package out of `Common` for the call the way `SE_GetString`
    // does, then puts it back.
    let mut pkg = take(&mut view.common.stringed);
    se_check_for_language_updates(&mut pkg, &mut *view); // will take zero time to execute unless language changes, then will reload strings.
                                                         //	of course this still doesn't work for menus...
    view.common.stringed = pkg;

    if cl.cls.state as c_int == connstate_t::CA_DISCONNECTED as c_int
        && cl.cls.keyCatchers & KEYCATCH_UI == 0
        && view.common.cvar(view.common.com_sv_running).integer == 0
    {
        // if disconnected, bring up the menu
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let snd = unsafe { snd_from_view(view) };
            S_StopAllSounds(view.common, snd);
        }
        VM_Call(
            view.common,
            cl.uivm,
            MpUiExport::UI_SET_ACTIVE_MENU as c_int,
            &[UIMENU_MAIN as isize],
        );
    }

    // if recording an avi, lock to a fixed fps
    if view.common.cvar(cl.cl_avidemo).integer != 0 && msec != 0 {
        // save the current screen
        if cl.cls.state as c_int == connstate_t::CA_ACTIVE as c_int
            || view.common.cvar(cl.cl_forceavidemo).integer != 0
        {
            if view.common.cvar(cl.cl_avidemo).integer > 0 {
                Cbuf_ExecuteText(view, cbufExec_t::EXEC_NOW as c_int, "screenshot silent\n");
            } else {
                Cbuf_ExecuteText(
                    view,
                    cbufExec_t::EXEC_NOW as c_int,
                    "screenshot_tga silent\n",
                );
            }
        }
        // fixed time for next frame'
        msec = ((1000 / view.common.cvar(cl.cl_avidemo).integer.abs()) as f32
            * view.common.cvar(view.common.com_timescale).value) as c_int;
        if msec == 0 {
            msec = 1;
        }
    }

    CL_MakeMonkeyDoLaundry(view.common, cl);

    // save the msec before checking pause
    cl.cls.realFrametime = msec;

    // decide the simulation time
    cl.cls.frametime = msec;
    if view.common.cvar(cl.cl_framerate).integer != 0 {
        cl.avgFrametime += msec as f32;
        if cl.frameCount & 0x1f == 0 {
            let mess = format!(
                "Frame rate={}\n\n",
                1000.0f32 * (1.0 / (cl.avgFrametime / 32.0f32))
            );
            //		OutputDebugString(mess);
            com_printf(view.common, &mess);
            cl.avgFrametime = 0.0f32;
        }
        cl.frameCount += 1;
    }

    cl.cls.realtime += cl.cls.frametime;

    if view.common.cvar(cl.cl_timegraph).integer != 0 {
        let value = cl.cls.realFrametime as f32 * 0.25;
        SCR_DebugGraph(cl, value, 0);
    }

    // see if we need to update any userinfo
    CL_CheckUserinfo(view.common, cl);

    // if we haven't gotten a packet in a long time,
    // drop the connection
    CL_CheckTimeout(view, cl);

    // send intentions now
    CL_SendCmd(view, cl);

    // resend a connection request if necessary
    CL_CheckForResend(view.common, cl);

    // decide on the serverTime to render
    CL_SetCGameTime(view, cl);

    // update the screen
    SCR_UpdateScreen(view, cl);

    // update audio
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    S_Update(view, snd);

    // advance local effects for next frame
    SCR_RunCinematic(view, cl);

    Con_RunConsole(view.common, cl);

    // Raven resets `G2VertSpaceServer` here for the game-side ghoul2 vertex
    // transforms. The heap is dropped by the ghoul2 design, so there is nothing
    // to reset.

    cl.cls.framecount += 1;
}

/// `CL_Init` — registers every client cvar and command, then starts the client.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2549-2710`
pub fn CL_Init(view: &mut EngineHostView, cl: &mut Client) {
    //	Com_Printf( "----- Client Initialization -----\n" );

    Con_Init(view, cl);

    CL_ClearState(cl);

    cl.cls.state = connstate_t::CA_DISCONNECTED; // no longer CA_UNINITIALIZED

    cl.cls.realtime = 0;

    CL_InitInput(view, cl);

    //
    // register our variables
    //
    cl.cl_noprint = Some(Cvar_Get(view, "cl_noprint", "0", 0));
    cl.cl_motd = Some(Cvar_Get(view, "cl_motd", "1", 0));

    cl.cl_timeout = Some(Cvar_Get(view, "cl_timeout", "200", 0));

    cl.cl_timeNudge = Some(Cvar_Get(view, "cl_timeNudge", "0", CVAR_TEMP));
    cl.cl_shownet = Some(Cvar_Get(view, "cl_shownet", "0", CVAR_TEMP));
    cl.cl_showSend = Some(Cvar_Get(view, "cl_showSend", "0", CVAR_TEMP));
    cl.cl_showTimeDelta = Some(Cvar_Get(view, "cl_showTimeDelta", "0", CVAR_TEMP));
    cl.cl_freezeDemo = Some(Cvar_Get(view, "cl_freezeDemo", "0", CVAR_TEMP));
    cl.rcon_client_password = Some(Cvar_Get(view, "rconPassword", "", CVAR_TEMP));
    cl.cl_activeAction = Some(Cvar_Get(view, "activeAction", "", CVAR_TEMP));

    cl.cl_timedemo = Some(Cvar_Get(view, "timedemo", "0", 0));
    cl.cl_avidemo = Some(Cvar_Get(view, "cl_avidemo", "0", 0));
    cl.cl_forceavidemo = Some(Cvar_Get(view, "cl_forceavidemo", "0", 0));

    cl.rconAddress = Some(Cvar_Get(view, "rconAddress", "", 0));

    cl.cl_yawspeed = Some(Cvar_Get(view, "cl_yawspeed", "140", CVAR_ARCHIVE));
    cl.cl_pitchspeed = Some(Cvar_Get(view, "cl_pitchspeed", "140", CVAR_ARCHIVE));
    cl.cl_anglespeedkey = Some(Cvar_Get(view, "cl_anglespeedkey", "1.5", CVAR_ARCHIVE));

    cl.cl_maxpackets = Some(Cvar_Get(view, "cl_maxpackets", "30", CVAR_ARCHIVE));
    cl.cl_packetdup = Some(Cvar_Get(view, "cl_packetdup", "1", CVAR_ARCHIVE));

    cl.cl_run = Some(Cvar_Get(view, "cl_run", "1", CVAR_ARCHIVE));
    cl.cl_sensitivity = Some(Cvar_Get(view, "sensitivity", "5", CVAR_ARCHIVE));
    cl.cl_mouseAccel = Some(Cvar_Get(view, "cl_mouseAccel", "0", CVAR_ARCHIVE));
    cl.cl_freelook = Some(Cvar_Get(view, "cl_freelook", "1", CVAR_ARCHIVE));

    cl.cl_showMouseRate = Some(Cvar_Get(view, "cl_showmouserate", "0", 0));
    cl.cl_framerate = Some(Cvar_Get(view, "cl_framerate", "0", CVAR_TEMP));
    cl.cl_allowDownload = Some(Cvar_Get(view, "cl_allowDownload", "0", CVAR_ARCHIVE));
    cl.cl_allowAltEnter = Some(Cvar_Get(view, "cl_allowAltEnter", "0", CVAR_ARCHIVE));

    cl.cl_autolodscale = Some(Cvar_Get(view, "cl_autolodscale", "1", CVAR_ARCHIVE));

    cl.cl_conXOffset = Some(Cvar_Get(view, "cl_conXOffset", "0", 0));
    cl.cl_inGameVideo = Some(Cvar_Get(view, "r_inGameVideo", "1", CVAR_ARCHIVE));

    cl.cl_serverStatusResendTime = Some(Cvar_Get(view, "cl_serverStatusResendTime", "750", 0));

    // init autoswitch so the ui will have it correctly even
    // if the cgame hasn't been started
    Cvar_Get(view, "cg_autoswitch", "1", CVAR_ARCHIVE);

    cl.m_pitchVeh = Some(Cvar_Get(view, "m_pitchVeh", "0.022", CVAR_ARCHIVE));
    cl.m_pitch = Some(Cvar_Get(view, "m_pitch", "0.022", CVAR_ARCHIVE));
    cl.m_yaw = Some(Cvar_Get(view, "m_yaw", "0.022", CVAR_ARCHIVE));
    cl.m_forward = Some(Cvar_Get(view, "m_forward", "0.25", CVAR_ARCHIVE));
    cl.m_side = Some(Cvar_Get(view, "m_side", "0.25", CVAR_ARCHIVE));
    cl.m_filter = Some(Cvar_Get(view, "m_filter", "0", CVAR_ARCHIVE));

    cl.cl_motdString = Some(Cvar_Get(view, "cl_motdString", "", CVAR_ROM));

    Cvar_Get(view, "cl_maxPing", "800", CVAR_ARCHIVE);

    // userinfo
    Cvar_Get(view, "name", "Padawan", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "rate", "4000", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "snaps", "20", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "model", "kyle/default", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(
        view,
        "forcepowers",
        "7-1-032330000000001333",
        CVAR_USERINFO | CVAR_ARCHIVE,
    );
    //	Cvar_Get ("g_redTeam", "Empire", CVAR_SERVERINFO | CVAR_ARCHIVE);
    //	Cvar_Get ("g_blueTeam", "Rebellion", CVAR_SERVERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "color1", "4", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "color2", "4", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "handicap", "100", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "teamtask", "0", CVAR_USERINFO);
    Cvar_Get(view, "sex", "male", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "password", "", CVAR_USERINFO);
    Cvar_Get(view, "cg_predictItems", "1", CVAR_USERINFO | CVAR_ARCHIVE);

    //default sabers
    Cvar_Get(view, "saber1", "single_1", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(view, "saber2", "none", CVAR_USERINFO | CVAR_ARCHIVE);

    //skin color
    Cvar_Get(view, "char_color_red", "255", CVAR_USERINFO | CVAR_ARCHIVE);
    Cvar_Get(
        view,
        "char_color_green",
        "255",
        CVAR_USERINFO | CVAR_ARCHIVE,
    );
    Cvar_Get(view, "char_color_blue", "255", CVAR_USERINFO | CVAR_ARCHIVE);

    // cgame might not be initialized before menu is used
    Cvar_Get(view, "cg_viewsize", "100", CVAR_ARCHIVE);

    //
    // register our commands
    //
    Cmd_AddCommand(view, "cmd", Some(CL_ForwardToServer_f_cmd));
    Cmd_AddCommand(view, "globalservers", Some(CL_GlobalServers_f_cmd));
    Cmd_AddCommand(view, "record", Some(CL_Record_f_cmd));
    Cmd_AddCommand(view, "demo", Some(CL_PlayDemo_f_cmd));
    Cmd_AddCommand(view, "stoprecord", Some(CL_StopRecord_f_cmd));
    Cmd_AddCommand(view, "configstrings", Some(CL_Configstrings_f_cmd));
    Cmd_AddCommand(view, "clientinfo", Some(CL_Clientinfo_f_cmd));
    Cmd_AddCommand(view, "snd_restart", Some(CL_Snd_Restart_f_cmd));
    Cmd_AddCommand(view, "vid_restart", Some(CL_Vid_Restart_f_cmd));
    Cmd_AddCommand(view, "disconnect", Some(CL_Disconnect_f_cmd));
    Cmd_AddCommand(view, "cinematic", Some(CL_PlayCinematic_f_cmd));
    Cmd_AddCommand(view, "connect", Some(CL_Connect_f_cmd));
    Cmd_AddCommand(view, "reconnect", Some(CL_Reconnect_f_cmd));
    Cmd_AddCommand(view, "localservers", Some(CL_LocalServers_f_cmd));
    Cmd_AddCommand(view, "rcon", Some(CL_Rcon_f_cmd));
    Cmd_AddCommand(view, "ping", Some(CL_Ping_f_cmd));
    Cmd_AddCommand(view, "serverstatus", Some(CL_ServerStatus_f_cmd));
    Cmd_AddCommand(view, "showip", Some(CL_ShowIP_f_cmd));
    Cmd_AddCommand(view, "fs_openedList", Some(CL_OpenedPK3List_f_cmd));
    Cmd_AddCommand(view, "fs_referencedList", Some(CL_ReferencedPK3List_f_cmd));
    Cmd_AddCommand(view, "model", Some(CL_SetModel_f_cmd));
    Cmd_AddCommand(view, "forcepowers", Some(CL_SetForcePowers_f_cmd));

    CL_InitRef(view);

    SCR_Init(view, cl);

    Cbuf_Execute(view);

    Cvar_Set(view, "cl_running", "1");

    // Raven allocates `G2VertSpaceClient = new CMiniHeap(...)` here for the
    // cgame-side model vertex transforms (cl_main.cpp:2703). `CMiniHeap` is
    // dropped by the ghoul2 design, the same disposition `SV_Init` records for
    // `G2VertSpaceServer`, so this allocation drops too.

    //	Com_Printf( "----- Client Initialization Complete -----\n" );
}

// The `CmdFunction` adapters `CL_Init` registers above. `CmdFunction` is
// `fn(&mut EngineHostView)`, and each `CL_*_f` handler takes its own receivers,
// so a forwarder casts the view's `cl` slot back and calls the handler
// (`sv_ccmds.rs`'s `SV_Map_f_cmd` idiom). They sit together beside the one
// function that registers them, exactly as the server's forwarders sit beside
// `SV_AddOperatorCommands`.
//
// SAFETY (every `cl_from_view` below): view-constructor slot, single-threaded,
// no other cast of the same slot live across the handler call.

fn CL_ForwardToServer_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_ForwardToServer_f(view.common, cl)
}

fn CL_GlobalServers_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_GlobalServers_f(view.common, cl)
}

fn CL_Record_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Record_f(view, cl)
}

fn CL_PlayDemo_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_PlayDemo_f(view, cl)
}

fn CL_StopRecord_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_StopRecord_f(view.common, cl)
}

fn CL_Configstrings_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Configstrings_f(view.common, cl)
}

fn CL_Clientinfo_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Clientinfo_f(view.common, cl)
}

fn CL_Snd_Restart_f_cmd(view: &mut EngineHostView) {
    CL_Snd_Restart_f(view)
}

fn CL_Vid_Restart_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Vid_Restart_f(view, cl)
}

fn CL_Disconnect_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Disconnect_f(view, cl)
}

fn CL_PlayCinematic_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_PlayCinematic_f(view, cl)
}

fn CL_Connect_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Connect_f(view, cl)
}

fn CL_Reconnect_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Reconnect_f(view, cl)
}

fn CL_LocalServers_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_LocalServers_f(view.common, cl)
}

fn CL_Rcon_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Rcon_f(view.common, cl)
}

fn CL_Ping_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_Ping_f(view.common, cl)
}

fn CL_ServerStatus_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    CL_ServerStatus_f(view.common, cl)
}

fn CL_ShowIP_f_cmd(view: &mut EngineHostView) {
    CL_ShowIP_f(view.common)
}

fn CL_OpenedPK3List_f_cmd(view: &mut EngineHostView) {
    CL_OpenedPK3List_f(view.common)
}

fn CL_ReferencedPK3List_f_cmd(view: &mut EngineHostView) {
    CL_ReferencedPK3List_f(view.common)
}

fn CL_SetModel_f_cmd(view: &mut EngineHostView) {
    CL_SetModel_f(view)
}

fn CL_SetForcePowers_f_cmd(_view: &mut EngineHostView) {
    CL_SetForcePowers_f()
}
