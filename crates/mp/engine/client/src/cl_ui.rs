//! `cl_ui.cpp` — the UI VM host: the `UI_*`/`LAN_*` trap arms the `ui` module
//! calls into, and the `uivm` lifecycle (`CL_InitUI`/`CL_ShutdownUI`).
//!
//! Source: `oracle/codemp/client/cl_ui.cpp`

use core::ffi::{c_char, c_int};
use std::sync::Arc;

use mp_abi::ui::exports::MpUiExport;
use mp_abi::ui::imports::MpUiImport;
use mp_abi::ui::public::ui_client_state_t::uiClientState_t;
use mp_abi::ui::public::UI_API_VERSION;
use mp_engine_ghoul2::api_bolts::g2api_get_bolt_matrix;
use mp_engine_ghoul2::api_bones::g2api_set_bone_angles;
use mp_engine_ghoul2::api_models::{g2api_clean_ghoul2_models, g2api_init_ghoul2_model};
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_qcommon::cmd_common::{Cbuf_ExecuteText, Cmd_Argc, Cmd_ArgvBuffer};
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{
    Com_DPrintf, Com_Memcpy, Com_Memset, Com_RealTime, Q_acos, Q_asin,
};
use mp_engine_qcommon::cvar_fns::{
    Cvar_Get, Cvar_InfoStringBuffer, Cvar_Register, Cvar_Reset, Cvar_Set, Cvar_SetValue,
    Cvar_Update, Cvar_VariableStringBuffer, Cvar_VariableValue,
};
use mp_engine_qcommon::files_common::{
    FS_FCloseFile, FS_FOpenFileRead, FS_Read, FS_SV_FOpenFileRead, FS_Write,
};
use mp_engine_qcommon::files_pc::{
    FS_FOpenFileByMode, FS_GetFileList, FS_Read2, FS_SV_FOpenFileWrite,
};
use mp_engine_qcommon::net_chan::{NET_AdrToString, NET_CompareAdr, NET_StringToAdr};
use mp_engine_qcommon::qcommon::shared_traps_t::sharedTraps_t;
use mp_engine_qcommon::qcommon::vm_interpret_t::vmInterpret_t;
use mp_engine_qcommon::stringed::api::{se_get_language_name, se_get_num_languages};
use mp_engine_qcommon::stringed::SE_GetString;
use mp_engine_qcommon::timing::sys_milliseconds;
use mp_engine_qcommon::vm::ui_syscall_trampoline_words;
use mp_engine_qcommon::vm_fns::{VM_ArgPtrWord, VM_Call, VM_Create, VM_Free};
use mp_engine_qcommon::z_memman_pc::{Hunk_MemoryRemaining, Z_Free};
use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
use mp_qshared::shared::connstate_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::file_mode::fsMode_t;
use mp_qshared::shared::shared_ik_move_params::sharedIKMoveParams_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use mp_renderer::hook_install::{re_from_view, rm_from_view};
use mp_renderer::tr_cmds::{RE_SetColor, RE_StretchPic};
use mp_renderer::tr_font::{
    AnyLanguage_ReadCharFromString, GetLanguageEnum, Language_IsAsian, Language_UsesSpaces,
    RE_Font_DrawString, RE_Font_HeightPixels, RE_Font_StrLenChars, RE_Font_StrLenPixels,
    RE_RegisterFont,
};
use mp_renderer::tr_image::RE_RegisterSkin;
use mp_renderer::tr_model::frontend::{r_lerp_tag, r_model_bounds, RE_RegisterModel};
use mp_renderer::tr_scene::{
    RE_AddLightToScene, RE_AddPolyToScene, RE_AddRefEntityToScene, RE_ClearScene, RE_RenderScene,
};
use mp_renderer::tr_shader::{RE_RegisterShaderNoMip, RE_ShaderNameFromIndex, R_RemapShader};
use native_math::eorientations::Eorientations;
use native_math::qmath::{AngleVectors, MatrixMultiply, PerpendicularVectorMP};
use native_math::vector::vec3_t;
use native_types::{fileHandle_t, mdxaBone_t, qhandle_t};
use native_string::info::Info_SetValueForKey;
use native_string::q_strncpyz::Q_strncpyz;
use native_string::{latin1_to_string, string_to_latin1};

use crate::cl_keys::{Key_KeynumToString, Key_SetBinding};
use crate::client::client_static_t::{MAX_GLOBAL_SERVERS, MAX_OTHER_SERVERS};
use crate::client::server_info_t::serverInfo_t;
use crate::client_host::{bot_from_view, client_legacy_syscall, sv_from_view};
use crate::client_host::snd_from_view;
use crate::snd_dma::{S_RegisterSound, S_StartLocalSound, S_StopBackgroundTrack};
use crate::snd_dma::S_StartBackgroundTrack;
use crate::Client;

/// Raven's `AS_LOCAL`/`AS_MPLAYER`/`AS_GLOBAL`/`AS_FAVORITES` server-source
/// selector has no rosetta row yet.
///
/// PORT-NOTE(consts): numbering matches Raven's `enum { AS_LOCAL, AS_MPLAYER,
/// AS_GLOBAL, AS_FAVORITES }` declaration order.
/// Source: `oracle/codemp/qcommon/qcommon.h`
const AS_LOCAL: c_int = 0;
const AS_MPLAYER: c_int = 1;
const AS_GLOBAL: c_int = 2;
const AS_FAVORITES: c_int = 3;

/// Raven `KEYCATCH_UI` bit flag. No rosetta row exists yet.
///
/// PORT-NOTE(consts): transcribed from Raven's `#define KEYCATCH_UI 0x0002`.
/// Source: `oracle/codemp/client/keys.h`
const KEYCATCH_UI: c_int = 0x0002;

/// Raven's `SORT_HOST`/`SORT_MAP`/`SORT_CLIENTS`/`SORT_GAME`/`SORT_PING` sort
/// keys have no rosetta row yet.
///
/// PORT-NOTE(consts): numbering matches Raven's declaration order.
/// Source: `oracle/codemp/ui/ui_shared.h`
const SORT_HOST: c_int = 0;
const SORT_MAP: c_int = 1;
const SORT_CLIENTS: c_int = 2;
const SORT_GAME: c_int = 3;
const SORT_PING: c_int = 4;

/// Reads a NUL-terminated seam `c_char*` as a Latin-1 `String` (the #13
/// string-campaign discipline) for `Q_strncpyz`'s `&str` source parameter.
///
/// # Safety
/// `p` must point at a NUL-terminated buffer, exactly like the C string it replaces.
unsafe fn cstr_to_string(p: *const c_char) -> String {
    latin1_to_string(core::ffi::CStr::from_ptr(p).to_bytes())
}

/// Borrows a module-space C string as its Latin-1 bytes, the shape the `RE_Font_*`
/// signatures take. The bytes stay in module memory for the whole dispatch.
fn cstr_bytes<'a>(p: *const c_char) -> &'a [u8] {
    // SAFETY: the module passed a NUL-terminated string across the seam.
    unsafe { core::ffi::CStr::from_ptr(p).to_bytes() }
}

/// Reads a module-space `const float *rgba` as the port's nullable-color model.
/// Raven passes NULL for "keep the current colour".
fn rgba_arg(p: *const f32) -> Option<[f32; 4]> {
    if p.is_null() {
        None
    } else {
        // SAFETY: the module passed a four-float colour across the seam.
        Some(unsafe { *(p as *const [f32; 4]) })
    }
}

/// Reborrow one `CGhoul2Info` out of the handle's arena without keeping the
/// `Ghoul2System` borrow, so the same call can still pass `g2` as its own
/// receiver. This is the arena twin of the view's slot-cast discipline.
fn g2_info<'a>(g2: &mut Ghoul2System, ghoul2: &CGhoul2Info_v, index: c_int) -> &'a mut CGhoul2Info {
    let p = ghoul2.get_mut(g2, index) as *mut CGhoul2Info;
    // SAFETY: the arena slot outlives the dispatch, and no other borrow of the
    // same slot is live (single-threaded synchronous traps).
    unsafe { &mut *p }
}

/// The shared body of the three `UI_G2_GETBOLT*` arms, which differ only in the
/// two mode flags they set first.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:1252-1262`
fn get_bolt_matrix_arm(view: &mut EngineHostView, g2: &mut Ghoul2System, args: *mut isize) -> bool {
    // SAFETY: the handle, the model list, and the matrix out-param are all
    // module-space (porting-rules §D11).
    unsafe {
        let common: *const Common = view.common;
        let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
        let bolt_matrix = &mut *(VM_ArgPtrWord(&*common, *args.offset(4)) as *mut mdxaBone_t);
        g2api_get_bolt_matrix(
            g2,
            view,
            ghoul2,
            *args.offset(2) as c_int,
            *args.offset(3) as c_int,
            *(VM_ArgPtrWord(&*common, *args.offset(5)) as *const vec3_t),
            *(VM_ArgPtrWord(&*common, *args.offset(6)) as *const vec3_t),
            *args.offset(7) as c_int,
            core::slice::from_raw_parts(
                VM_ArgPtrWord(&*common, *args.offset(8)) as *const qhandle_t,
                0,
            ),
            *(VM_ArgPtrWord(&*common, *args.offset(9)) as *const vec3_t),
            bolt_matrix,
        )
    }
}

/// Reads a fixed `[c_char; N]` field as a Latin-1 `String`.
fn field_to_string(buf: &[c_char]) -> String {
    let bytes: &[u8] = unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) };
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    latin1_to_string(&bytes[..len])
}

/// Raven `static void GetClientState( uiClientState_t *state )` — fills the
/// UI-visible connection snapshot from the live client state.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:41-48`
pub fn GetClientState(cl: &mut Client, state: *mut uiClientState_t) {
    // SAFETY: `state` is the VM's seam out-param pointer (porting-rules §D11).
    unsafe {
        (*state).connectPacketCount = cl.clc.connectPacketCount;
        (*state).connState = cl.cls.state;
        Q_strncpyz(
            &mut (*state).servername,
            &field_to_string(&cl.cls.servername),
            (*state).servername.len(),
        );
        Q_strncpyz(
            &mut (*state).updateInfoString,
            &field_to_string(&cl.cls.updateInfoString),
            (*state).updateInfoString.len(),
        );
        Q_strncpyz(
            &mut (*state).messageString,
            &field_to_string(&cl.clc.serverMessage),
            (*state).messageString.len(),
        );
        (*state).clientNum = cl.cl.snap.ps.clientNum;
    }
}

/// Raven `void LAN_LoadCachedServers( )` — loads the cached server list off disk.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:55-77`
pub fn LAN_LoadCachedServers(common: &mut Common, cl: &mut Client) {
    cl.cls.numglobalservers = 0;
    cl.cls.nummplayerservers = 0;
    cl.cls.numfavoriteservers = 0;
    cl.cls.numGlobalServerAddresses = 0;

    let mut file_in: native_types::fileHandle_t = 0;
    if FS_SV_FOpenFileRead(common, "servercache.dat", &mut file_in) != 0 {
        let int_size = core::mem::size_of::<c_int>() as c_int;
        // SAFETY: seam file reads into owned struct fields (porting-rules §D11).
        unsafe {
            FS_Read(
                common,
                &mut cl.cls.numglobalservers as *mut _ as *mut (),
                int_size,
                file_in,
            );
            FS_Read(
                common,
                &mut cl.cls.nummplayerservers as *mut _ as *mut (),
                int_size,
                file_in,
            );
            FS_Read(
                common,
                &mut cl.cls.numfavoriteservers as *mut _ as *mut (),
                int_size,
                file_in,
            );
            let mut size: c_int = 0;
            FS_Read(common, &mut size as *mut _ as *mut (), int_size, file_in);
            let expect = (core::mem::size_of_val(&cl.cls.globalServers)
                + core::mem::size_of_val(&cl.cls.favoriteServers)
                + core::mem::size_of_val(&cl.cls.mplayerServers)) as c_int;
            if size == expect {
                FS_Read(
                    common,
                    cl.cls.globalServers.as_mut_ptr() as *mut (),
                    core::mem::size_of_val(&cl.cls.globalServers) as c_int,
                    file_in,
                );
                FS_Read(
                    common,
                    cl.cls.mplayerServers.as_mut_ptr() as *mut (),
                    core::mem::size_of_val(&cl.cls.mplayerServers) as c_int,
                    file_in,
                );
                FS_Read(
                    common,
                    cl.cls.favoriteServers.as_mut_ptr() as *mut (),
                    core::mem::size_of_val(&cl.cls.favoriteServers) as c_int,
                    file_in,
                );
            } else {
                cl.cls.numglobalservers = 0;
                cl.cls.nummplayerservers = 0;
                cl.cls.numfavoriteservers = 0;
                cl.cls.numGlobalServerAddresses = 0;
            }
        }
        FS_FCloseFile(common, file_in);
    }
}

/// Raven `void LAN_SaveServersToCache( )` — writes the cached server list to disk.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:84-98`
pub fn LAN_SaveServersToCache(common: &mut Common, cl: &mut Client) {
    let file_out = FS_SV_FOpenFileWrite(common, "servercache.dat");
    let int_size = core::mem::size_of::<c_int>() as c_int;
    // SAFETY: seam file writes from owned struct fields (porting-rules §D11).
    unsafe {
        FS_Write(
            common,
            &cl.cls.numglobalservers as *const _ as *const (),
            int_size,
            file_out,
        );
        FS_Write(
            common,
            &cl.cls.nummplayerservers as *const _ as *const (),
            int_size,
            file_out,
        );
        FS_Write(
            common,
            &cl.cls.numfavoriteservers as *const _ as *const (),
            int_size,
            file_out,
        );
        let size = (core::mem::size_of_val(&cl.cls.globalServers)
            + core::mem::size_of_val(&cl.cls.favoriteServers)
            + core::mem::size_of_val(&cl.cls.mplayerServers)) as c_int;
        FS_Write(common, &size as *const _ as *const (), int_size, file_out);
        FS_Write(
            common,
            cl.cls.globalServers.as_ptr() as *const (),
            core::mem::size_of_val(&cl.cls.globalServers) as c_int,
            file_out,
        );
        FS_Write(
            common,
            cl.cls.mplayerServers.as_ptr() as *const (),
            core::mem::size_of_val(&cl.cls.mplayerServers) as c_int,
            file_out,
        );
        FS_Write(
            common,
            cl.cls.favoriteServers.as_ptr() as *const (),
            core::mem::size_of_val(&cl.cls.favoriteServers) as c_int,
            file_out,
        );
    }
    FS_FCloseFile(common, file_out);
}

/// Raven `static void LAN_ResetPings(int source)` — clears the ping field on
/// every server in a source list.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:106-134`
pub fn LAN_ResetPings(cl: &mut Client, source: c_int) {
    let servers: Option<&mut [serverInfo_t]> = match source {
        AS_LOCAL => Some(&mut cl.cls.localServers[..]),
        AS_MPLAYER => Some(&mut cl.cls.mplayerServers[..]),
        AS_GLOBAL => Some(&mut cl.cls.globalServers[..]),
        AS_FAVORITES => Some(&mut cl.cls.favoriteServers[..]),
        _ => None,
    };
    if let Some(servers) = servers {
        for server in servers.iter_mut() {
            server.ping = -1;
        }
    }
}

/// Raven `static int LAN_AddServer(int source, const char *name, const char
/// *address)` — inserts a server into a source list if not already present.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:141-193`
pub fn LAN_AddServer(
    common: &mut Common,
    cl: &mut Client,
    source: c_int,
    name: *const c_char,
    address: *const c_char,
) -> c_int {
    let mut max = MAX_OTHER_SERVERS as c_int;
    let (count, servers): (&mut c_int, &mut [serverInfo_t]) = match source {
        AS_LOCAL => (&mut cl.cls.numlocalservers, &mut cl.cls.localServers[..]),
        AS_MPLAYER => (
            &mut cl.cls.nummplayerservers,
            &mut cl.cls.mplayerServers[..],
        ),
        AS_GLOBAL => {
            max = MAX_GLOBAL_SERVERS as c_int;
            (&mut cl.cls.numglobalservers, &mut cl.cls.globalServers[..])
        }
        AS_FAVORITES => (
            &mut cl.cls.numfavoriteservers,
            &mut cl.cls.favoriteServers[..],
        ),
        _ => return -1,
    };
    if *count >= max {
        return -1;
    }
    let mut adr: netadr_t = unsafe { core::mem::zeroed() };
    // SAFETY: `address` is the VM's seam string pointer (porting-rules §D11).
    unsafe {
        NET_StringToAdr(address, &mut adr);
    }
    if adr.r#type == netadrtype_t::NA_BAD {
        return -1;
    }
    let mut i = 0;
    while i < *count {
        if NET_CompareAdr(common, servers[i as usize].adr, adr) != 0 {
            break;
        }
        i += 1;
    }
    if i >= *count {
        servers[*count as usize].adr = adr;
        // SAFETY: `name` is the VM's seam string pointer (porting-rules §D11).
        let name_str = unsafe { cstr_to_string(name) };
        let len = servers[*count as usize].hostName.len();
        Q_strncpyz(&mut servers[*count as usize].hostName, &name_str, len);
        servers[*count as usize].visible = qtrue;
        *count += 1;
        return 1;
    }
    0
}

/// Raven `static void LAN_RemoveServer(int source, const char *addr)` —
/// removes a server from a source list and shifts the tail down.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:200-237`
pub fn LAN_RemoveServer(common: &mut Common, cl: &mut Client, source: c_int, addr: *const c_char) {
    let (count, servers): (&mut c_int, &mut [serverInfo_t]) = match source {
        AS_LOCAL => (&mut cl.cls.numlocalservers, &mut cl.cls.localServers[..]),
        AS_MPLAYER => (
            &mut cl.cls.nummplayerservers,
            &mut cl.cls.mplayerServers[..],
        ),
        AS_GLOBAL => (&mut cl.cls.numglobalservers, &mut cl.cls.globalServers[..]),
        AS_FAVORITES => (
            &mut cl.cls.numfavoriteservers,
            &mut cl.cls.favoriteServers[..],
        ),
        _ => return,
    };
    let mut comp: netadr_t = unsafe { core::mem::zeroed() };
    // SAFETY: `addr` is the VM's seam string pointer (porting-rules §D11).
    unsafe {
        NET_StringToAdr(addr, &mut comp);
    }
    let mut i = 0;
    while i < *count {
        if servers[i as usize].adr.r#type == netadrtype_t::NA_BAD
            || NET_CompareAdr(common, comp, servers[i as usize].adr) != 0
        {
            let mut j = i;
            while j < *count - 1 {
                servers.swap(j as usize, (j + 1) as usize);
                j += 1;
            }
            *count -= 1;
            break;
        }
        i += 1;
    }
}

/// Raven `static int LAN_GetServerCount( int source )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:245-261`
pub fn LAN_GetServerCount(cl: &mut Client, source: c_int) -> c_int {
    match source {
        AS_LOCAL => cl.cls.numlocalservers,
        AS_MPLAYER => cl.cls.nummplayerservers,
        AS_GLOBAL => cl.cls.numglobalservers,
        AS_FAVORITES => cl.cls.numfavoriteservers,
        _ => 0,
    }
}

/// Raven `static void LAN_GetServerAddressString( int source, int n, char
/// *buf, int buflen )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:268-296`
pub fn LAN_GetServerAddressString(
    common: &mut Common,
    cl: &mut Client,
    source: c_int,
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
) {
    let adr = match source {
        AS_LOCAL if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(cl.cls.localServers[n as usize].adr)
        }
        AS_MPLAYER if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(cl.cls.mplayerServers[n as usize].adr)
        }
        AS_GLOBAL if n >= 0 && n < MAX_GLOBAL_SERVERS as c_int => {
            Some(cl.cls.globalServers[n as usize].adr)
        }
        AS_FAVORITES if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(cl.cls.favoriteServers[n as usize].adr)
        }
        _ => None,
    };
    if let Some(adr) = adr {
        let s = NET_AdrToString(common, adr);
        // SAFETY: `s` is a `Common`-owned static scratch C string; `buf` is
        // the VM's seam out-buffer (porting-rules §D11).
        unsafe {
            let text = cstr_to_string(s);
            Q_strncpyz(
                core::slice::from_raw_parts_mut(buf, buflen as usize),
                &text,
                buflen as usize,
            );
        }
        return;
    }
    // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
    unsafe {
        *buf = 0;
    }
}

/// Raven `static void LAN_GetServerInfo( int source, int n, char *buf, int
/// buflen )` — packs a server's info as an info string for the UI.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:303-358`
pub fn LAN_GetServerInfo(
    common: &mut Common,
    cl: &mut Client,
    source: c_int,
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
) {
    let server: Option<&serverInfo_t> = match source {
        AS_LOCAL if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(&cl.cls.localServers[n as usize])
        }
        AS_MPLAYER if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(&cl.cls.mplayerServers[n as usize])
        }
        AS_GLOBAL if n >= 0 && n < MAX_GLOBAL_SERVERS as c_int => {
            Some(&cl.cls.globalServers[n as usize])
        }
        AS_FAVORITES if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(&cl.cls.favoriteServers[n as usize])
        }
        _ => None,
    };
    if let (Some(server), false) = (server, buf.is_null()) {
        // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
        unsafe {
            *buf = 0;
        }
        let mut info = String::new();
        Info_SetValueForKey(&mut info, "hostname", &field_to_string(&server.hostName));
        Info_SetValueForKey(&mut info, "mapname", &field_to_string(&server.mapName));
        Info_SetValueForKey(&mut info, "clients", &server.clients.to_string());
        Info_SetValueForKey(&mut info, "sv_maxclients", &server.maxClients.to_string());
        Info_SetValueForKey(&mut info, "ping", &server.ping.to_string());
        Info_SetValueForKey(&mut info, "minping", &server.minPing.to_string());
        Info_SetValueForKey(&mut info, "maxping", &server.maxPing.to_string());
        Info_SetValueForKey(&mut info, "nettype", &server.netType.to_string());
        Info_SetValueForKey(&mut info, "needpass", &server.needPassword.to_string());
        Info_SetValueForKey(&mut info, "truejedi", &server.trueJedi.to_string());
        Info_SetValueForKey(&mut info, "wdisable", &server.weaponDisable.to_string());
        Info_SetValueForKey(&mut info, "fdisable", &server.forceDisable.to_string());
        Info_SetValueForKey(&mut info, "game", &field_to_string(&server.game));
        Info_SetValueForKey(&mut info, "gametype", &server.gameType.to_string());
        let addr_str = unsafe { cstr_to_string(NET_AdrToString(common, server.adr)) };
        Info_SetValueForKey(&mut info, "addr", &addr_str);
        Q_strncpyz(
            unsafe { core::slice::from_raw_parts_mut(buf, buflen as usize) },
            &info,
            buflen as usize,
        );
    } else if !buf.is_null() {
        // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
        unsafe {
            *buf = 0;
        }
    }
}

/// Raven `static int LAN_GetServerPing( int source, int n )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:365-393`
pub fn LAN_GetServerPing(cl: &mut Client, source: c_int, n: c_int) -> c_int {
    let server: Option<&serverInfo_t> = match source {
        AS_LOCAL if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(&cl.cls.localServers[n as usize])
        }
        AS_MPLAYER if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(&cl.cls.mplayerServers[n as usize])
        }
        AS_GLOBAL if n >= 0 && n < MAX_GLOBAL_SERVERS as c_int => {
            Some(&cl.cls.globalServers[n as usize])
        }
        AS_FAVORITES if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            Some(&cl.cls.favoriteServers[n as usize])
        }
        _ => None,
    };
    server.map(|s| s.ping).unwrap_or(-1)
}

/// Raven `static serverInfo_t *LAN_GetServerPtr( int source, int n )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:400-424`
pub fn LAN_GetServerPtr(cl: &mut Client, source: c_int, n: c_int) -> *mut serverInfo_t {
    match source {
        AS_LOCAL if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            &mut cl.cls.localServers[n as usize] as *mut serverInfo_t
        }
        AS_MPLAYER if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            &mut cl.cls.mplayerServers[n as usize] as *mut serverInfo_t
        }
        AS_GLOBAL if n >= 0 && n < MAX_GLOBAL_SERVERS as c_int => {
            &mut cl.cls.globalServers[n as usize] as *mut serverInfo_t
        }
        AS_FAVORITES if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            &mut cl.cls.favoriteServers[n as usize] as *mut serverInfo_t
        }
        _ => core::ptr::null_mut(),
    }
}

/// Raven `static void LAN_MarkServerVisible(int source, int n, qboolean
/// visible )` — `n == -1` marks the whole source list.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:536-585`
pub fn LAN_MarkServerVisible(cl: &mut Client, source: c_int, n: c_int, visible: qboolean) {
    if n == -1 {
        let servers: Option<&mut [serverInfo_t]> = match source {
            AS_LOCAL => Some(&mut cl.cls.localServers[..]),
            AS_MPLAYER => Some(&mut cl.cls.mplayerServers[..]),
            AS_GLOBAL => Some(&mut cl.cls.globalServers[..]),
            AS_FAVORITES => Some(&mut cl.cls.favoriteServers[..]),
            _ => None,
        };
        if let Some(servers) = servers {
            for server in servers.iter_mut() {
                server.visible = visible;
            }
        }
        return;
    }
    match source {
        AS_LOCAL if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            cl.cls.localServers[n as usize].visible = visible;
        }
        AS_MPLAYER if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            cl.cls.mplayerServers[n as usize].visible = visible;
        }
        AS_GLOBAL if n >= 0 && n < MAX_GLOBAL_SERVERS as c_int => {
            cl.cls.globalServers[n as usize].visible = visible;
        }
        AS_FAVORITES if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            cl.cls.favoriteServers[n as usize].visible = visible;
        }
        _ => {}
    }
}

/// Raven `static int LAN_ServerIsVisible(int source, int n )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:593-617`
pub fn LAN_ServerIsVisible(cl: &mut Client, source: c_int, n: c_int) -> c_int {
    match source {
        AS_LOCAL if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            cl.cls.localServers[n as usize].visible
        }
        AS_MPLAYER if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            cl.cls.mplayerServers[n as usize].visible
        }
        AS_GLOBAL if n >= 0 && n < MAX_GLOBAL_SERVERS as c_int => {
            cl.cls.globalServers[n as usize].visible
        }
        AS_FAVORITES if n >= 0 && n < MAX_OTHER_SERVERS as c_int => {
            cl.cls.favoriteServers[n as usize].visible
        }
        _ => qfalse,
    }
}

/// Raven `static void CL_GetGlconfig( glconfig_t *config )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:642-644`
pub fn CL_GetGlconfig(cl: &mut Client, config: *mut glconfig_t) {
    // `glconfig_t` is an ABI-frozen `#[repr(C)]` block with no `Copy`, so the
    // seam copy is the raw structure copy Raven's `*config = cls.glconfig` is.
    // SAFETY: `config` is the VM's seam out-param pointer (porting-rules §D11).
    unsafe {
        core::ptr::copy_nonoverlapping(&cl.cls.glconfig as *const glconfig_t, config, 1);
    }
}

/// Raven `static void GetClipboardData( char *buf, int buflen )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:651-664`
pub fn GetClipboardData(common: &mut Common, buf: *mut c_char, buflen: c_int) {
    let cbd = native_platform::Sys_GetClipboardData();
    // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
    unsafe {
        if cbd.is_null() {
            *buf = 0;
            return;
        }
        let text = cstr_to_string(cbd);
        Q_strncpyz(
            core::slice::from_raw_parts_mut(buf, buflen as usize),
            &text,
            buflen as usize,
        );
    }
    Z_Free(common, cbd as *mut ());
}

/// Raven `int Key_GetCatcher( void )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:708-710`
pub fn Key_GetCatcher(cl: &mut Client) -> c_int {
    cl.cls.keyCatchers
}

/// Raven `void Key_SetCatcher( int catcher )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:717-719`
pub fn Key_SetCatcher(cl: &mut Client, catcher: c_int) {
    cl.cls.keyCatchers = catcher;
}

/// Raven `static int GetConfigString(int index, char *buf, int size)`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:769-787`
pub fn GetConfigString(cl: &mut Client, index: c_int, buf: *mut c_char, size: c_int) -> c_int {
    if index < 0 || index >= mp_qshared::shared::game_state::MAX_CONFIGSTRINGS as c_int {
        return qfalse;
    }
    let offset = cl.cl.gameState.stringOffsets[index as usize];
    if offset == 0 {
        if size != 0 {
            // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
            unsafe {
                *buf = 0;
            }
        }
        return qfalse;
    }
    let bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(
            cl.cl.gameState.stringData.as_ptr().add(offset as usize) as *const u8,
            cl.cl.gameState.stringData.len() - offset as usize,
        )
    };
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let text = latin1_to_string(&bytes[..len]);
    // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
    unsafe {
        Q_strncpyz(
            core::slice::from_raw_parts_mut(buf, size as usize),
            &text,
            size as usize,
        );
    }
    qtrue
}

/// Raven `static int FloatAsInt( float f )` — reinterprets a float's bit
/// pattern as an int (the VM's word-transport convention for float args).
///
/// Source: `oracle/codemp/client/cl_ui.cpp:794-800`
pub fn FloatAsInt(f: f32) -> c_int {
    f.to_bits() as c_int
}

/// Raven `void CL_ShutdownUI( void )` — tears down the `ui` VM.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:1444-1454`
pub fn CL_ShutdownUI(common: &mut Common, cl: &mut Client) {
    cl.cls.keyCatchers &= !KEYCATCH_UI;
    cl.cls.uiStarted = qfalse;
    if cl.uivm.is_null() {
        return;
    }
    VM_Call(common, cl.uivm, MpUiExport::UI_SHUTDOWN as c_int, &[]);
    VM_Call(common, cl.uivm, MpUiExport::UI_MENU_RESET as c_int, &[]);
    VM_Free(common, cl.uivm);
    cl.uivm = core::ptr::null_mut();
}

/// Raven `qboolean UI_usesUniqueCDKey()`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:1498-1504`
pub fn UI_usesUniqueCDKey(common: &mut Common, cl: &mut Client) -> qboolean {
    if !cl.uivm.is_null() {
        (VM_Call(common, cl.uivm, MpUiExport::UI_HASUNIQUECDKEY as c_int, &[]) == qtrue as isize)
            as qboolean
    } else {
        qfalse
    }
}

/// Raven `qboolean UI_GameCommand( void )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:1513-1519`
pub fn UI_GameCommand(common: &mut Common, cl: &mut Client) -> qboolean {
    if cl.uivm.is_null() {
        return qfalse;
    }
    VM_Call(
        common,
        cl.uivm,
        MpUiExport::UI_CONSOLE_COMMAND as c_int,
        &[cl.cls.realtime as isize],
    ) as qboolean
}

/// Raven `static int LAN_CompareServers( int source, int sortKey, int
/// sortDir, int s1, int s2 )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:431-493`
pub fn LAN_CompareServers(
    cl: &mut Client,
    source: c_int,
    sortKey: c_int,
    sortDir: c_int,
    s1: c_int,
    s2: c_int,
) -> c_int {
    let server1 = LAN_GetServerPtr(cl, source, s1);
    let server2 = LAN_GetServerPtr(cl, source, s2);
    if server1.is_null() || server2.is_null() {
        return 0;
    }
    // SAFETY: both pointers were just validated non-null by `LAN_GetServerPtr`,
    // which only returns valid in-bounds server slots.
    let (server1, server2) = unsafe { (&*server1, &*server2) };
    let mut res = 0;
    match sortKey {
        SORT_HOST => {
            res = native_string::q_string::Q_stricmp(
                &field_to_string(&server1.hostName),
                &field_to_string(&server2.hostName),
            )
        }
        SORT_MAP => {
            res = native_string::q_string::Q_stricmp(
                &field_to_string(&server1.mapName),
                &field_to_string(&server2.mapName),
            )
        }
        SORT_CLIENTS => {
            res = if server1.clients < server2.clients {
                -1
            } else if server1.clients > server2.clients {
                1
            } else {
                0
            };
        }
        SORT_GAME => {
            res = if server1.gameType < server2.gameType {
                -1
            } else if server1.gameType > server2.gameType {
                1
            } else {
                0
            };
        }
        SORT_PING => {
            res = if server1.ping < server2.ping {
                -1
            } else if server1.ping > server2.ping {
                1
            } else {
                0
            };
        }
        _ => {}
    }
    if sortDir != 0 {
        if res < 0 {
            return 1;
        }
        if res > 0 {
            return -1;
        }
        return 0;
    }
    res
}

/// Raven `static int LAN_GetPingQueueCount( void )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:500-502`
pub fn LAN_GetPingQueueCount(cl: &mut Client) -> c_int {
    crate::cl_main::CL_GetPingQueueCount(cl)
}

/// Raven `static void LAN_ClearPing( int n )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:509-511`
pub fn LAN_ClearPing(cl: &mut Client, n: c_int) {
    crate::cl_main::CL_ClearPing(cl, n);
}

/// Raven `static void LAN_GetPingInfo( int n, char *buf, int buflen )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:527-529`
pub fn LAN_GetPingInfo(cl: &mut Client, n: c_int, buf: *mut c_char, buflen: c_int) {
    crate::cl_main::CL_GetPingInfo(cl, n, buf, buflen);
}

/// Raven `static void Key_GetBindingBuf( int keynum, char *buf, int buflen )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:691-701`
pub fn Key_GetBindingBuf(cl: &mut Client, keynum: c_int, buf: *mut c_char, buflen: c_int) {
    let value = crate::cl_keys::Key_GetBinding(cl, keynum);
    if !value.is_null() {
        // SAFETY: `value` is a `kg`-owned binding-table C string; `buf` is
        // the VM's seam out-buffer (porting-rules §D11).
        unsafe {
            let text = cstr_to_string(value);
            Q_strncpyz(
                core::slice::from_raw_parts_mut(buf, buflen as usize),
                &text,
                buflen as usize,
            );
        }
    } else {
        // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
        unsafe {
            *buf = 0;
        }
    }
}

/// Raven `int LAN_GetServerStatus( char *serverAddress, char *serverStatus,
/// int maxLen )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:633-635`
pub fn LAN_GetServerStatus(
    view: &mut EngineHostView,
    cl: &mut Client,
    serverAddress: *mut c_char,
    serverStatus: *mut c_char,
    maxLen: c_int,
) -> c_int {
    crate::cl_main::CL_ServerStatus(view, cl, serverAddress, serverStatus, maxLen)
}

/// Raven `void Key_KeynumToStringBuf( int keynum, char *buf, int buflen )` —
/// prefers a Stringed-localized friendly key name when one exists.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:674-683`
pub fn Key_KeynumToStringBuf(
    view: &mut EngineHostView,
    cl: &mut Client,
    keynum: c_int,
    buf: *mut c_char,
    buflen: c_int,
) {
    // SAFETY: `Key_KeynumToString` answers with a NUL-terminated name buffer.
    let ps_key_name = unsafe { cstr_to_string(Key_KeynumToString(cl, keynum)) };
    let ps_key_name_friendly = SE_GetString(view, &format!("KEYNAMES_KEYNAME_{ps_key_name}"));
    let chosen = if !ps_key_name_friendly.is_empty() {
        ps_key_name_friendly
    } else {
        ps_key_name
    };
    // SAFETY: `buf` is the VM's seam out-buffer (porting-rules §D11).
    unsafe {
        Q_strncpyz(
            core::slice::from_raw_parts_mut(buf, buflen as usize),
            &chosen,
            buflen as usize,
        );
    }
}

/// Raven `static void LAN_GetPing( int n, char *buf, int buflen, int
/// *pingtime )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:518-520`
pub fn LAN_GetPing(
    common: &mut Common,
    cl: &mut Client,
    n: c_int,
    buf: *mut c_char,
    buflen: c_int,
    pingtime: *mut c_int,
) {
    crate::cl_main::CL_GetPing(common, cl, n, buf, buflen, pingtime);
}

/// Raven `qboolean LAN_UpdateVisiblePings(int source )`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:624-626`
pub fn LAN_UpdateVisiblePings(common: &mut Common, cl: &mut Client, source: c_int) -> qboolean {
    crate::cl_main::CL_UpdateVisiblePings_f(common, cl, source)
}

/// Raven `int CL_UISystemCalls( int *args )` — the `ui` module's syscall
/// dispatcher (`VMA`/`VMF` macros).
///
/// Raven's `botlib_export` is a file-scope global, carried on `Client` per the
/// carrier rule until the merge lane threads a real slot.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:813-1437`
#[allow(clippy::too_many_arguments, unused_variables)]
pub fn CL_UISystemCalls(
    view: &mut EngineHostView,
    cl: &mut Client,
    g2: &mut Ghoul2System,
    args: *mut isize,
) -> c_int {
    // SAFETY: `args` is the trampoline's raw syscall word array (seam
    // pointer, porting-rules §D11); every arm reads only the words its trap
    // number defines, exactly as Raven's `int *args` does. The word is full
    // width, because a 64-bit module hands the engine a 64-bit pointer. A value
    // argument narrows to `c_int` at its read site, which is Raven's own width.
    unsafe fn vma(common: &Common, args: *mut isize, n: isize) -> *mut () {
        VM_ArgPtrWord(common, *args.offset(n))
    }
    unsafe fn vmf(args: *mut isize, n: isize) -> f32 {
        // A float travels in the low half of the word.
        f32::from_bits(*args.offset(n) as u32)
    }

    let trap = unsafe { *args.offset(0) as c_int };

    if trap == sharedTraps_t::TRAP_MEMSET as c_int {
        unsafe {
            Com_Memset(
                vma(view.common, args, 1),
                *args.offset(2) as c_int,
                *args.offset(3) as usize,
            )
        };
        0
    } else if trap == sharedTraps_t::TRAP_MEMCPY as c_int {
        unsafe {
            Com_Memcpy(
                vma(view.common, args, 1),
                vma(view.common, args, 2) as *const (),
                *args.offset(3) as usize,
            )
        };
        0
    } else if trap == sharedTraps_t::TRAP_STRNCPY as c_int {
        unsafe {
            let dst = vma(view.common, args, 1) as *mut c_char;
            let src = vma(view.common, args, 2) as *const c_char;
            libc::strncpy(dst, src, *args.offset(3) as usize);
            dst as c_int
        }
    } else if trap == sharedTraps_t::TRAP_SIN as c_int {
        unsafe { FloatAsInt(vmf(args, 1).sin()) }
    } else if trap == sharedTraps_t::TRAP_COS as c_int {
        unsafe { FloatAsInt(vmf(args, 1).cos()) }
    } else if trap == sharedTraps_t::TRAP_ATAN2 as c_int {
        unsafe { FloatAsInt(vmf(args, 1).atan2(vmf(args, 2))) }
    } else if trap == sharedTraps_t::TRAP_SQRT as c_int {
        unsafe { FloatAsInt(vmf(args, 1).sqrt()) }
    } else if trap == sharedTraps_t::TRAP_MATRIXMULTIPLY as c_int {
        unsafe {
            MatrixMultiply(
                &*(vma(view.common, args, 1) as *const [[f32; 3]; 3]),
                &*(vma(view.common, args, 2) as *const [[f32; 3]; 3]),
                &mut *(vma(view.common, args, 3) as *mut [[f32; 3]; 3]),
            );
        }
        0
    } else if trap == sharedTraps_t::TRAP_ANGLEVECTORS as c_int {
        unsafe {
            AngleVectors(
                *(vma(view.common, args, 1) as *const vec3_t),
                (vma(view.common, args, 2) as *mut vec3_t).as_mut(),
                (vma(view.common, args, 3) as *mut vec3_t).as_mut(),
                (vma(view.common, args, 4) as *mut vec3_t).as_mut(),
            );
        }
        0
    } else if trap == sharedTraps_t::TRAP_PERPENDICULARVECTOR as c_int {
        unsafe {
            PerpendicularVectorMP(
                &mut *(vma(view.common, args, 1) as *mut vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
            );
        }
        0
    } else if trap == sharedTraps_t::TRAP_FLOOR as c_int {
        unsafe { FloatAsInt(vmf(args, 1).floor()) }
    } else if trap == sharedTraps_t::TRAP_CEIL as c_int {
        unsafe { FloatAsInt(vmf(args, 1).ceil()) }
    } else if trap == sharedTraps_t::TRAP_TESTPRINTINT as c_int
        || trap == sharedTraps_t::TRAP_TESTPRINTFLOAT as c_int
    {
        0
    } else if trap == sharedTraps_t::TRAP_ACOS as c_int {
        unsafe { FloatAsInt(Q_acos(vmf(args, 1))) }
    } else if trap == sharedTraps_t::TRAP_ASIN as c_int {
        unsafe { FloatAsInt(Q_asin(vmf(args, 1))) }
    } else if trap == MpUiImport::UI_ERROR as c_int {
        // SAFETY: `vma` resolves the VM's seam string-arg word (porting-rules §D11).
        com_error(errorParm_t::ERR_DROP, unsafe {
            cstr_to_string(vma(view.common, args, 1) as *const c_char)
        });
    } else if trap == MpUiImport::UI_PRINT as c_int {
        // SAFETY: see above.
        com_printf(view.common, &unsafe {
            cstr_to_string(vma(view.common, args, 1) as *const c_char)
        });
        0
    } else if trap == MpUiImport::UI_MILLISECONDS as c_int {
        sys_milliseconds(view.common)
    } else if trap == MpUiImport::UI_CVAR_REGISTER as c_int {
        unsafe {
            Cvar_Register(
                view,
                vma(view.common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
                &cstr_to_string(vma(view.common, args, 2) as *const c_char),
                &cstr_to_string(vma(view.common, args, 3) as *const c_char),
                *args.offset(4) as c_int,
            );
        }
        0
    } else if trap == MpUiImport::UI_CVAR_UPDATE as c_int {
        unsafe {
            Cvar_Update(
                view.common,
                vma(view.common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
            )
        };
        0
    } else if trap == MpUiImport::UI_CVAR_SET as c_int {
        unsafe {
            Cvar_Set(
                view,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
                &cstr_to_string(vma(view.common, args, 2) as *const c_char),
            );
        }
        0
    } else if trap == MpUiImport::UI_CVAR_VARIABLEVALUE as c_int {
        unsafe {
            FloatAsInt(Cvar_VariableValue(
                view.common,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
            ))
        }
    } else if trap == MpUiImport::UI_CVAR_VARIABLESTRINGBUFFER as c_int {
        unsafe {
            Cvar_VariableStringBuffer(
                view.common,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
        }
        0
    } else if trap == MpUiImport::UI_CVAR_SETVALUE as c_int {
        unsafe {
            Cvar_SetValue(
                view,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
                vmf(args, 2),
            )
        };
        0
    } else if trap == MpUiImport::UI_CVAR_RESET as c_int {
        unsafe {
            Cvar_Reset(
                view,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
            )
        };
        0
    } else if trap == MpUiImport::UI_CVAR_CREATE as c_int {
        unsafe {
            Cvar_Get(
                view,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
                &cstr_to_string(vma(view.common, args, 2) as *const c_char),
                *args.offset(3) as c_int,
            );
        }
        0
    } else if trap == MpUiImport::UI_CVAR_INFOSTRINGBUFFER as c_int {
        unsafe {
            Cvar_InfoStringBuffer(
                view.common,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
        }
        0
    } else if trap == MpUiImport::UI_ARGC as c_int {
        Cmd_Argc(view.common)
    } else if trap == MpUiImport::UI_ARGV as c_int {
        unsafe {
            Cmd_ArgvBuffer(
                view.common,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_CMD_EXECUTETEXT as c_int {
        unsafe {
            Cbuf_ExecuteText(
                view,
                *args.offset(1) as c_int,
                &cstr_to_string(vma(view.common, args, 2) as *const c_char),
            )
        };
        0
    } else if trap == MpUiImport::UI_FS_FOPENFILE as c_int {
        unsafe {
            FS_FOpenFileByMode(
                view,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
                vma(view.common, args, 2) as *mut native_types::fileHandle_t,
                core::mem::transmute::<c_int, fsMode_t>(*args.offset(3) as c_int),
            )
        }
    } else if trap == MpUiImport::UI_FS_READ as c_int {
        unsafe {
            FS_Read2(
                view.common,
                vma(view.common, args, 1),
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_FS_WRITE as c_int {
        unsafe {
            FS_Write(
                view.common,
                vma(view.common, args, 1) as *const (),
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_FS_FCLOSEFILE as c_int {
        unsafe { FS_FCloseFile(view.common, *args.offset(1) as c_int) };
        0
    } else if trap == MpUiImport::UI_FS_GETFILELIST as c_int {
        unsafe {
            FS_GetFileList(
                view,
                &cstr_to_string(vma(view.common, args, 1) as *const c_char),
                &cstr_to_string(vma(view.common, args, 2) as *const c_char),
                vma(view.common, args, 3) as *mut c_char,
                *args.offset(4) as c_int,
            )
        }
    } else if trap == MpUiImport::UI_R_REGISTERMODEL as c_int {
        // Renderer reach (DEC-59.1): the `RE_*` frontend fns take their
        // declared receivers straight off the `re`/`rm` slots this view carries.
        unsafe {
            let name = cstr_to_string(vma(view.common, args, 1) as *const c_char);
            let re = re_from_view(view);
            let rm = rm_from_view(view);
            RE_RegisterModel(
                &mut re.qs,
                &mut re.world_load,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
                &mut re.world_effects,
                &name,
            )
        }
    } else if trap == MpUiImport::UI_R_REGISTERSKIN as c_int {
        unsafe {
            let name = cstr_to_string(vma(view.common, args, 1) as *const c_char);
            let re = re_from_view(view);
            let rm = rm_from_view(view);
            RE_RegisterSkin(
                &mut re.qs,
                &mut re.world_load,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
                &name,
            )
        }
    } else if trap == MpUiImport::UI_R_REGISTERSHADERNOMIP as c_int {
        unsafe {
            let name = cstr_to_string(vma(view.common, args, 1) as *const c_char);
            let re = re_from_view(view);
            let rm = rm_from_view(view);
            RE_RegisterShaderNoMip(
                &name,
                &mut re.qs,
                &mut re.world_load,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
            )
        }
    } else if trap == MpUiImport::UI_R_SHADERNAMEFROMINDEX as c_int {
        unsafe {
            let game_mem = vma(view.common, args, 1) as *mut c_char;
            let re = re_from_view(view);
            let ret_mem = RE_ShaderNameFromIndex(&re.sim.published, *args.offset(2) as c_int);
            if !ret_mem.is_empty() {
                let s = string_to_latin1(ret_mem);
                core::ptr::copy_nonoverlapping(s.as_ptr(), game_mem as *mut u8, s.len());
                *game_mem.add(s.len()) = 0;
            } else {
                *game_mem = 0;
            }
        }
        0
    } else if trap == MpUiImport::UI_R_CLEARSCENE as c_int {
        let re = unsafe { re_from_view(view) };
        RE_ClearScene(&mut re.frame_data, &mut re.scene);
        0
    } else if trap == MpUiImport::UI_R_ADDREFENTITYTOSCENE as c_int {
        unsafe {
            let ent = &*(vma(view.common, args, 1) as *const _);
            let re = re_from_view(view);
            RE_AddRefEntityToScene(&mut re.frame_data, &re.sim.published, &mut re.scene, ent)
        };
        0
    } else if trap == MpUiImport::UI_R_ADDPOLYTOSCENE as c_int {
        unsafe {
            let hshader = *args.offset(1) as c_int;
            let num_verts = *args.offset(2) as usize;
            let verts =
                core::slice::from_raw_parts(vma(view.common, args, 3) as *const _, num_verts);
            let re = re_from_view(view);
            RE_AddPolyToScene(
                &mut re.frame_data,
                &re.sim.published,
                view.common,
                hshader,
                verts,
                num_verts,
                1,
            )
        };
        0
    } else if trap == MpUiImport::UI_R_ADDLIGHTTOSCENE as c_int {
        unsafe {
            let org = *(vma(view.common, args, 1) as *const vec3_t);
            let re = re_from_view(view);
            RE_AddLightToScene(
                &mut re.frame_data,
                &re.sim.published,
                org,
                vmf(args, 2),
                vmf(args, 3),
                vmf(args, 4),
                vmf(args, 5),
            )
        };
        0
    } else if trap == MpUiImport::UI_R_RENDERSCENE as c_int {
        // SAFETY: `args` is the trampoline's 16-word frame (porting-rules §D11).
        let fd = unsafe { vma(view.common, args, 1) } as *const refdef_t;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `refdef_t` (porting-rules §D11).
        RE_RenderScene(
            unsafe { &*fd },
            &mut re.frame_data,
            &re.sim.published,
            &re.cvars,
            &mut re.scene,
            view.common,
            &re.sim.light_styles,
        );
        0
    } else if trap == MpUiImport::UI_R_SETCOLOR as c_int {
        unsafe {
            let ptr = vma(view.common, args, 1) as *const f32;
            let rgba = if ptr.is_null() {
                None
            } else {
                Some(*(ptr as *const [f32; 4]))
            };
            let re = re_from_view(view);
            RE_SetColor(&mut re.frame_data, rgba)
        };
        0
    } else if trap == MpUiImport::UI_R_DRAWSTRETCHPIC as c_int {
        unsafe {
            let (x, y, w, h, s1, t1, s2, t2) = (
                vmf(args, 1),
                vmf(args, 2),
                vmf(args, 3),
                vmf(args, 4),
                vmf(args, 5),
                vmf(args, 6),
                vmf(args, 7),
                vmf(args, 8),
            );
            let h_shader = *args.offset(9) as c_int;
            let re = re_from_view(view);
            RE_StretchPic(
                &mut re.frame_data,
                &re.sim.published,
                view.common,
                x,
                y,
                w,
                h,
                s1,
                t1,
                s2,
                t2,
                h_shader,
            )
        };
        0
    } else if trap == MpUiImport::UI_R_MODELBOUNDS as c_int {
        unsafe {
            let handle = *args.offset(1) as c_int;
            let mins_ptr = vma(view.common, args, 2) as *mut f32;
            let maxs_ptr = vma(view.common, args, 3) as *mut f32;
            let rm = rm_from_view(view);
            let re = re_from_view(view);
            let (mins, maxs) = r_model_bounds(rm, &re.sim.published, handle);
            core::ptr::copy_nonoverlapping(mins.as_ptr(), mins_ptr, 3);
            core::ptr::copy_nonoverlapping(maxs.as_ptr(), maxs_ptr, 3);
        };
        0
    } else if trap == MpUiImport::UI_UPDATESCREEN as c_int {
        crate::cl_scrn::SCR_UpdateScreen(view, cl);
        0
    } else if trap == MpUiImport::UI_CM_LERPTAG as c_int {
        unsafe {
            let tag_ptr = vma(view.common, args, 1) as *mut _;
            let handle = *args.offset(2) as c_int;
            let start_frame = *args.offset(3) as c_int;
            let end_frame = *args.offset(4) as c_int;
            let frac = vmf(args, 5);
            let tag_name = cstr_to_string(vma(view.common, args, 6) as *const c_char);
            let rm = rm_from_view(view);
            r_lerp_tag(
                rm,
                &mut *tag_ptr,
                handle,
                start_frame,
                end_frame,
                frac,
                &tag_name,
            )
        };
        0
    } else if trap == MpUiImport::UI_S_REGISTERSOUND as c_int {
        let name = unsafe { cstr_to_string(vma(view.common, args, 1) as *const c_char) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_RegisterSound(view, snd, &name)
    } else if trap == MpUiImport::UI_S_STARTLOCALSOUND as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        unsafe {
            S_StartLocalSound(
                view,
                snd,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_KEY_KEYNUMTOSTRINGBUF as c_int {
        unsafe {
            Key_KeynumToStringBuf(
                view,
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_KEY_GETBINDINGBUF as c_int {
        unsafe {
            Key_GetBindingBuf(
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_KEY_SETBINDING as c_int {
        unsafe {
            Key_SetBinding(
                view,
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *const c_char,
            )
        };
        0
    } else if trap == MpUiImport::UI_KEY_ISDOWN as c_int {
        unsafe { crate::cl_keys::Key_IsDown(cl, *args.offset(1) as c_int) }
    } else if trap == MpUiImport::UI_KEY_GETOVERSTRIKEMODE as c_int {
        crate::cl_keys::Key_GetOverstrikeMode(cl)
    } else if trap == MpUiImport::UI_KEY_SETOVERSTRIKEMODE as c_int {
        unsafe { crate::cl_keys::Key_SetOverstrikeMode(cl, *args.offset(1) as c_int) };
        0
    } else if trap == MpUiImport::UI_KEY_CLEARSTATES as c_int {
        crate::cl_keys::Key_ClearStates(view, cl);
        0
    } else if trap == MpUiImport::UI_KEY_GETCATCHER as c_int {
        Key_GetCatcher(cl)
    } else if trap == MpUiImport::UI_KEY_SETCATCHER as c_int {
        unsafe { Key_SetCatcher(cl, *args.offset(1) as c_int) };
        0
    } else if trap == MpUiImport::UI_GETCLIPBOARDDATA as c_int {
        unsafe {
            GetClipboardData(
                view.common,
                vma(view.common, args, 1) as *mut c_char,
                *args.offset(2) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_GETCLIENTSTATE as c_int {
        unsafe { GetClientState(cl, vma(view.common, args, 1) as *mut uiClientState_t) };
        0
    } else if trap == MpUiImport::UI_GETGLCONFIG as c_int {
        unsafe {
            CL_GetGlconfig(cl, vma(view.common, args, 1) as *mut glconfig_t)
        };
        0
    } else if trap == MpUiImport::UI_GETCONFIGSTRING as c_int {
        unsafe {
            GetConfigString(
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            )
        }
    } else if trap == MpUiImport::UI_LAN_LOADCACHEDSERVERS as c_int {
        LAN_LoadCachedServers(view.common, cl);
        0
    } else if trap == MpUiImport::UI_LAN_SAVECACHEDSERVERS as c_int {
        LAN_SaveServersToCache(view.common, cl);
        0
    } else if trap == MpUiImport::UI_LAN_ADDSERVER as c_int {
        unsafe {
            LAN_AddServer(
                view.common,
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *const c_char,
                vma(view.common, args, 3) as *const c_char,
            )
        }
    } else if trap == MpUiImport::UI_LAN_REMOVESERVER as c_int {
        unsafe {
            LAN_RemoveServer(
                view.common,
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *const c_char,
            )
        };
        0
    } else if trap == MpUiImport::UI_LAN_GETPINGQUEUECOUNT as c_int {
        LAN_GetPingQueueCount(cl)
    } else if trap == MpUiImport::UI_LAN_CLEARPING as c_int {
        unsafe { LAN_ClearPing(cl, *args.offset(1) as c_int) };
        0
    } else if trap == MpUiImport::UI_LAN_GETPING as c_int {
        unsafe {
            LAN_GetPing(
                view.common,
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
                vma(view.common, args, 4) as *mut c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_LAN_GETPINGINFO as c_int {
        unsafe {
            LAN_GetPingInfo(
                cl,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_LAN_GETSERVERCOUNT as c_int {
        unsafe { LAN_GetServerCount(cl, *args.offset(1) as c_int) }
    } else if trap == MpUiImport::UI_LAN_GETSERVERADDRESSSTRING as c_int {
        unsafe {
            LAN_GetServerAddressString(
                view.common,
                cl,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *mut c_char,
                *args.offset(4) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_LAN_GETSERVERINFO as c_int {
        unsafe {
            LAN_GetServerInfo(
                view.common,
                cl,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *mut c_char,
                *args.offset(4) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_LAN_GETSERVERPING as c_int {
        unsafe { LAN_GetServerPing(cl, *args.offset(1) as c_int, *args.offset(2) as c_int) }
    } else if trap == MpUiImport::UI_LAN_MARKSERVERVISIBLE as c_int {
        unsafe {
            LAN_MarkServerVisible(
                cl,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_LAN_SERVERISVISIBLE as c_int {
        unsafe { LAN_ServerIsVisible(cl, *args.offset(1) as c_int, *args.offset(2) as c_int) }
    } else if trap == MpUiImport::UI_LAN_UPDATEVISIBLEPINGS as c_int {
        unsafe { LAN_UpdateVisiblePings(view.common, cl, *args.offset(1) as c_int) }
    } else if trap == MpUiImport::UI_LAN_RESETPINGS as c_int {
        unsafe { LAN_ResetPings(cl, *args.offset(1) as c_int) };
        0
    } else if trap == MpUiImport::UI_LAN_SERVERSTATUS as c_int {
        unsafe {
            LAN_GetServerStatus(
                view,
                cl,
                vma(view.common, args, 1) as *mut c_char,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            )
        }
    } else if trap == MpUiImport::UI_LAN_COMPARESERVERS as c_int {
        unsafe {
            LAN_CompareServers(
                cl,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
            )
        }
    } else if trap == MpUiImport::UI_MEMORY_REMAINING as c_int {
        Hunk_MemoryRemaining(view.common)
    } else if trap == MpUiImport::UI_R_REGISTERFONT as c_int {
        let name = unsafe { cstr_to_string(vma(view.common, args, 1) as *const c_char) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let rm = unsafe { rm_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        RE_RegisterFont(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.sky,
            &mut re.font,
            language,
            mod_count,
            &name,
        )
    } else if trap == MpUiImport::UI_R_FONT_STRLENPIXELS as c_int {
        let text = cstr_bytes(unsafe { vma(view.common, args, 1) } as *const c_char);
        let (handle, scale) = unsafe { (*args.offset(2) as c_int, vmf(args, 3)) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let rm = unsafe { rm_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        RE_Font_StrLenPixels(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.sky,
            &mut re.font,
            language,
            mod_count,
            text,
            handle,
            scale,
        )
    } else if trap == MpUiImport::UI_R_FONT_STRLENCHARS as c_int {
        let text = cstr_bytes(unsafe { vma(view.common, args, 1) } as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        RE_Font_StrLenChars(&re.font, language, text)
    } else if trap == MpUiImport::UI_R_FONT_STRHEIGHTPIXELS as c_int {
        let (handle, scale) = unsafe { (*args.offset(1) as c_int, vmf(args, 2)) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let rm = unsafe { rm_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        RE_Font_HeightPixels(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.sky,
            &mut re.font,
            language,
            mod_count,
            handle,
            scale,
        )
    } else if trap == MpUiImport::UI_R_FONT_DRAWSTRING as c_int {
        let text = cstr_bytes(unsafe { vma(view.common, args, 3) } as *const c_char);
        let rgba = rgba_arg(unsafe { vma(view.common, args, 4) } as *const f32);
        let (ox, oy, handle, max_pixel_width, scale) = unsafe {
            (
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                vmf(args, 7),
            )
        };
        let millis = sys_milliseconds(view.common);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let rm = unsafe { rm_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        RE_Font_DrawString(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.sky,
            &mut re.font,
            language,
            mod_count,
            &mut re.frame_data,
            ox,
            oy,
            text,
            rgba,
            handle,
            max_pixel_width,
            scale,
            millis,
        );
        0
    } else if trap == MpUiImport::UI_LANGUAGE_ISASIAN as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        Language_IsAsian(language) as c_int
    } else if trap == MpUiImport::UI_LANGUAGE_USESSPACES as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        Language_UsesSpaces(language) as c_int
    } else if trap == MpUiImport::UI_ANYLANGUAGE_READCHARFROMSTRING as c_int {
        let text = cstr_bytes(unsafe { vma(view.common, args, 1) } as *const c_char);
        let advance_out = unsafe { vma(view.common, args, 2) } as *mut c_int;
        let punctuation_out = unsafe { vma(view.common, args, 3) } as *mut qboolean;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let (uiLetter, advance, trailing) =
            AnyLanguage_ReadCharFromString(&re.font, language, text, !punctuation_out.is_null());
        // SAFETY: both are the module's seam out-params (porting-rules §D11).
        unsafe {
            if !advance_out.is_null() {
                *advance_out = advance;
            }
            if !punctuation_out.is_null() {
                *punctuation_out = trailing.unwrap_or(false) as qboolean;
            }
        }
        uiLetter as c_int
    } else if trap == MpUiImport::UI_PC_ADD_GLOBAL_DEFINE as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let sv = unsafe { sv_from_view(view) };
        // SAFETY: the seam pointer is the module's string (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_AddGlobalDefine.unwrap())(
                vma(view.common, args, 1) as *mut c_char
            )
        }
    } else if trap == MpUiImport::UI_PC_LOAD_SOURCE as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointer is the module's string (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_LoadSourceHandle.unwrap())(
                bot,
                vma(view.common, args, 1) as *const c_char,
            )
        }
    } else if trap == MpUiImport::UI_PC_FREE_SOURCE as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: `args` is the trampoline's 16-word frame (porting-rules §D11).
        unsafe { ((*sv.botlib_export).PC_FreeSourceHandle.unwrap())(bot, *args.offset(1) as c_int) }
    } else if trap == MpUiImport::UI_PC_READ_TOKEN as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointers are the module's out-params (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_ReadTokenHandle.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut _,
            )
        }
    } else if trap == MpUiImport::UI_PC_SOURCE_FILE_AND_LINE as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointers are the module's out-params (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_SourceFileAndLine.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                vma(view.common, args, 3) as *mut c_int,
            )
        }
    } else if trap == MpUiImport::UI_PC_LOAD_GLOBAL_DEFINES as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointer is the module's string (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_LoadGlobalDefines.unwrap())(
                bot,
                vma(view.common, args, 1) as *const c_char,
            )
        }
    } else if trap == MpUiImport::UI_PC_REMOVE_ALL_GLOBAL_DEFINES as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: `sv.botlib_export` is the table `SV_BotInitBotLib` installed.
        unsafe { ((*sv.botlib_export).PC_RemoveAllGlobalDefines.unwrap())(bot) };
        0
    } else if trap == MpUiImport::UI_S_STOPBACKGROUNDTRACK as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StopBackgroundTrack(view.common, snd);
        0
    } else if trap == MpUiImport::UI_S_STARTBACKGROUNDTRACK as c_int {
        let intro = unsafe { cstr_to_string(vma(view.common, args, 1) as *const c_char) };
        let loop_track = unsafe { cstr_to_string(vma(view.common, args, 2) as *const c_char) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StartBackgroundTrack(view, snd, &intro, &loop_track, false);
        0
    } else if trap == MpUiImport::UI_REAL_TIME as c_int {
        unsafe {
            Com_RealTime(
                vma(view.common, args, 1) as *mut mp_qshared::common::mp::qcommon::qtime::qtime_t
            )
        }
    } else if trap == MpUiImport::UI_CIN_PLAYCINEMATIC as c_int {
        Com_DPrintf(view.common, "UI_CIN_PlayCinematic\n");
        unsafe {
            crate::cl_cin::CIN_PlayCinematic(
                view,
                cl,
                vma(view.common, args, 1) as *const c_char,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
            )
        }
    } else if trap == MpUiImport::UI_CIN_STOPCINEMATIC as c_int {
        unsafe { crate::cl_cin::CIN_StopCinematic(view, cl, *args.offset(1) as c_int) }
    } else if trap == MpUiImport::UI_CIN_RUNCINEMATIC as c_int {
        unsafe { crate::cl_cin::CIN_RunCinematic(view, cl, *args.offset(1) as c_int) }
    } else if trap == MpUiImport::UI_CIN_DRAWCINEMATIC as c_int {
        unsafe { crate::cl_cin::CIN_DrawCinematic(view, cl, *args.offset(1) as c_int) };
        0
    } else if trap == MpUiImport::UI_CIN_SETEXTENTS as c_int {
        unsafe {
            crate::cl_cin::CIN_SetExtents(
                cl,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_R_REMAP_SHADER as c_int {
        unsafe {
            let shader_name = cstr_to_string(vma(view.common, args, 1) as *const c_char);
            let new_shader_name = cstr_to_string(vma(view.common, args, 2) as *const c_char);
            let time_offset = cstr_to_string(vma(view.common, args, 3) as *const c_char);
            let re = re_from_view(view);
            let rm = rm_from_view(view);
            R_RemapShader(
                &shader_name,
                &new_shader_name,
                Some(&time_offset),
                &mut re.qs,
                &mut re.world_load,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
            )
        };
        0
    } else if trap == MpUiImport::UI_SP_GETNUMLANGUAGES as c_int {
        let mut pkg = std::mem::take(&mut view.common.stringed);
        let n = se_get_num_languages(&mut pkg, view);
        view.common.stringed = pkg;
        n
    } else if trap == MpUiImport::UI_SP_GETLANGUAGENAME as c_int {
        unsafe {
            let hold_name = vma(view.common, args, 2) as *mut c_char;
            let language_name =
                se_get_language_name(&view.common.stringed, *args.offset(1) as c_int).to_string();
            Q_strncpyz(
                core::slice::from_raw_parts_mut(hold_name, 128),
                &language_name,
                128,
            );
        }
        0
    } else if trap == MpUiImport::UI_SP_GETSTRINGTEXTSTRING as c_int {
        unsafe {
            let reference = cstr_to_string(vma(view.common, args, 1) as *const c_char);
            let out_buf = vma(view.common, args, 2) as *mut c_char;
            let buflen = *args.offset(3) as usize;
            let text = SE_GetString(view, &reference);
            Q_strncpyz(
                core::slice::from_raw_parts_mut(out_buf, buflen),
                &text,
                buflen,
            );
        }
        qtrue
    } else if trap == MpUiImport::UI_G2_LISTSURFACES as c_int {
        // The `G2API_*` arms below reach the engine host through `view`, which
        // implements `EngineHost` directly (DEC-59.1).
        unsafe {
            mp_engine_ghoul2::api_surfaces::g2api_list_surfaces(
                g2,
                view,
                &mut *(*args.offset(1) as *mut mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info),
            )
        };
        0
    } else if trap == MpUiImport::UI_G2_LISTBONES as c_int {
        unsafe {
            mp_engine_ghoul2::api_bones::g2api_list_bones(
                g2,
                view,
                &mut *(*args.offset(1) as *mut mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info),
                *args.offset(2) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_G2_HAVEWEGHOULMODELS as c_int {
        unsafe {
            mp_engine_ghoul2::api_models::g2api_have_we_ghoul2_models(
                g2,
                &*(*args.offset(1) as *const CGhoul2Info_v),
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_SETMODELS as c_int {
        unsafe {
            mp_engine_ghoul2::api_models::g2api_set_ghoul2_model_indexes(
                g2,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                core::slice::from_raw_parts(vma(view.common, args, 2) as *const _, 0),
                core::slice::from_raw_parts(vma(view.common, args, 3) as *const _, 0),
            )
        };
        0
    } else if trap == MpUiImport::UI_G2_GETBOLT as c_int {
        get_bolt_matrix_arm(view, g2, args) as c_int
    } else if trap == MpUiImport::UI_G2_GETBOLT_NOREC as c_int {
        g2.gbm_no_reconstruct = true;
        get_bolt_matrix_arm(view, g2, args) as c_int
    } else if trap == MpUiImport::UI_G2_GETBOLT_NOREC_NOROT as c_int {
        g2.gbm_no_reconstruct = true;
        g2.gbm_use_sp_method = true;
        get_bolt_matrix_arm(view, g2, args) as c_int
    } else if trap == MpUiImport::UI_G2_INITGHOUL2MODEL as c_int {
        let file_name = unsafe { cstr_to_string(vma(view.common, args, 2) as *const c_char) };
        // SAFETY: `VMA(1)` is the module's `CGhoul2Info_v *` slot (§D11).
        // Raven `if (!(*ghoul2Ptr)) *ghoul2Ptr = new CGhoul2Info_v;` - the
        // handle object's `new`/`delete` is the engine's job: the engine owns
        // the `Box`, the module holds the raw pointer, freed at UI_G2_CLEANMODELS.
        let ghoul2 = unsafe {
            let pp = vma(view.common, args, 1) as *mut *mut CGhoul2Info_v;
            if (*pp).is_null() {
                *pp = Box::into_raw(Box::new(CGhoul2Info_v { mItem: 0 }));
            }
            &mut **pp
        };
        // SAFETY: `args` is the trampoline's 16-word frame (§D11).
        let (model_index, custom_skin, custom_shader, model_flags, lod_bias) = unsafe {
            (
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                *args.offset(7) as c_int,
            )
        };
        g2api_init_ghoul2_model(
            g2,
            view,
            ghoul2,
            &file_name,
            model_index,
            custom_skin,
            custom_shader,
            model_flags,
            lod_bias,
        )
    } else if trap == MpUiImport::UI_G2_COLLISIONDETECT as c_int
        || trap == MpUiImport::UI_G2_COLLISIONDETECTCACHE as c_int
    {
        // Raven: "not supported for ui" — both arms return 0.
        0
    } else if trap == MpUiImport::UI_G2_ANGLEOVERRIDE as c_int {
        // SAFETY: every pointer here is module-space (porting-rules §D11), and
        // `args` is the trampoline's 16-word frame.
        unsafe {
            let bone_name = cstr_to_string(vma(view.common, args, 3) as *const c_char);
            let angles = *(vma(view.common, args, 4) as *const vec3_t);
            let model_list =
                core::slice::from_raw_parts(vma(view.common, args, 9) as *const qhandle_t, 0);
            g2api_set_bone_angles(
                g2,
                view,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                &bone_name,
                angles,
                *args.offset(5) as c_int,
                core::mem::transmute::<c_int, Eorientations>(*args.offset(6) as c_int),
                core::mem::transmute::<c_int, Eorientations>(*args.offset(7) as c_int),
                core::mem::transmute::<c_int, Eorientations>(*args.offset(8) as c_int),
                model_list,
                *args.offset(10) as c_int,
                *args.offset(11) as c_int,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_CLEANMODELS as c_int {
        // SAFETY: `VMA(1)` is the module's `CGhoul2Info_v *` slot (§D11).
        // Raven guards the null pointee and then deletes and nulls the handle
        // (`G2_API.cpp:496-564`); the engine drops the `Box` it made at init.
        unsafe {
            let pp = vma(view.common, args, 1) as *mut *mut CGhoul2Info_v;
            if !(*pp).is_null() {
                g2api_clean_ghoul2_models(g2, &mut **pp);
                drop(Box::from_raw(*pp));
                *pp = core::ptr::null_mut();
            }
        }
        0
    } else if trap == MpUiImport::UI_G2_PLAYANIM as c_int {
        unsafe {
            mp_engine_ghoul2::api_bones::g2api_set_bone_anim(
                g2,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                &cstr_to_string(vma(view.common, args, 3) as *const c_char),
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                vmf(args, 7),
                *args.offset(8) as c_int,
                vmf(args, 9),
                *args.offset(10) as c_int,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_GETBONEANIM as c_int {
        unsafe {
            let model_index = *args.offset(10) as c_int;
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let ghl_info = g2_info(g2, ghoul2, model_index);
            let bone_name = cstr_to_string(vma(view.common, args, 2) as *const c_char);
            let model_list = core::slice::from_raw_parts(vma(view.common, args, 9) as *const _, 0);
            match mp_engine_ghoul2::api_bones::g2api_get_bone_anim(
                g2,
                view,
                ghl_info,
                &bone_name,
                *args.offset(3) as c_int,
                model_list,
            ) {
                Some((current_frame, start_frame, end_frame, flags, anim_speed)) => {
                    *(vma(view.common, args, 4) as *mut f32) = current_frame;
                    *(vma(view.common, args, 5) as *mut c_int) = start_frame;
                    *(vma(view.common, args, 6) as *mut c_int) = end_frame;
                    *(vma(view.common, args, 7) as *mut c_int) = flags;
                    *(vma(view.common, args, 8) as *mut f32) = anim_speed;
                    1
                }
                None => 0,
            }
        }
    } else if trap == MpUiImport::UI_G2_GETBONEFRAME as c_int {
        unsafe {
            let model_index = *args.offset(6) as c_int;
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let ghl_info = g2_info(g2, ghoul2, model_index);
            let bone_name = cstr_to_string(vma(view.common, args, 2) as *const c_char);
            let model_list = core::slice::from_raw_parts(vma(view.common, args, 5) as *const _, 0);
            match mp_engine_ghoul2::api_bones::g2api_get_bone_anim(
                g2,
                view,
                ghl_info,
                &bone_name,
                *args.offset(3) as c_int,
                model_list,
            ) {
                Some((current_frame, ..)) => {
                    *(vma(view.common, args, 4) as *mut f32) = current_frame;
                    1
                }
                None => 0,
            }
        }
    } else if trap == MpUiImport::UI_G2_GETGLANAME as c_int {
        unsafe {
            let point = vma(view.common, args, 3) as *mut c_char;
            if let Some(local) = mp_engine_ghoul2::api_saveload::g2api_get_gla_name(
                g2,
                view,
                &*(*args.offset(1) as *const CGhoul2Info_v),
                *args.offset(2) as c_int,
            ) {
                let s = string_to_latin1(&local);
                core::ptr::copy_nonoverlapping(s.as_ptr(), point as *mut u8, s.len());
                *point.add(s.len()) = 0;
            }
        }
        0
    } else if trap == MpUiImport::UI_G2_COPYGHOUL2INSTANCE as c_int {
        unsafe {
            mp_engine_ghoul2::api_models::g2api_copy_ghoul2_instance(
                g2,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                &mut *(*args.offset(2) as *mut CGhoul2Info_v),
                *args.offset(3) as c_int,
            )
        }
    } else if trap == MpUiImport::UI_G2_COPYSPECIFICGHOUL2MODEL as c_int {
        unsafe {
            mp_engine_ghoul2::api_models::g2api_copy_specific_g2_model(
                g2,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                &mut *(*args.offset(3) as *mut CGhoul2Info_v),
                *args.offset(4) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_G2_DUPLICATEGHOUL2INSTANCE as c_int {
        // Raven returns on a live destination (assert dropped, NDEBUG) and
        // allocates on a null one (`G2_API.cpp:2330-2340`). The engine owns
        // the `Box`, the module holds the raw pointer.
        unsafe {
            let pp = vma(view.common, args, 2) as *mut *mut CGhoul2Info_v;
            if (*pp).is_null() {
                *pp = Box::into_raw(Box::new(CGhoul2Info_v { mItem: 0 }));
                mp_engine_ghoul2::api_models::g2api_duplicate_ghoul2_instance(
                    g2,
                    &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                    &mut **pp,
                );
            }
        };
        0
    } else if trap == MpUiImport::UI_G2_HASGHOUL2MODELONINDEX as c_int {
        // §19: Raven derefs the null pointee; the gone-instance sanity answer
        // is qfalse, so the null pointee takes it too.
        let pp = unsafe { vma(view.common, args, 1) as *mut *mut CGhoul2Info_v };
        if unsafe { (*pp).is_null() } {
            0
        } else {
            unsafe {
                mp_engine_ghoul2::api_models::g2api_has_ghoul2_model_on_index(
                    g2,
                    &**pp,
                    *args.offset(2) as c_int,
                ) as c_int
            }
        }
    } else if trap == MpUiImport::UI_G2_REMOVEGHOUL2MODEL as c_int {
        // §19: same null-pointee answer as the HASGHOUL2MODELONINDEX arm.
        let pp = unsafe { vma(view.common, args, 1) as *mut *mut CGhoul2Info_v };
        if unsafe { (*pp).is_null() } {
            0
        } else {
            unsafe {
                mp_engine_ghoul2::api_models::g2api_remove_ghoul2_model(
                    g2,
                    &mut **pp,
                    *args.offset(2) as c_int,
                ) as c_int
            }
        }
    } else if trap == MpUiImport::UI_G2_ADDBOLT as c_int {
        unsafe {
            let bone_name = cstr_to_string(vma(view.common, args, 3) as *const c_char);
            mp_engine_ghoul2::api_bolts::g2api_add_bolt(
                g2,
                view,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                &bone_name,
            )
        }
    } else if trap == MpUiImport::UI_G2_SETBOLTON as c_int {
        unsafe {
            mp_engine_ghoul2::api_bolts::g2api_set_bolt_info(
                g2,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_G2_SETROOTSURFACE as c_int {
        unsafe {
            let surface_name = cstr_to_string(vma(view.common, args, 3) as *const c_char);
            mp_engine_ghoul2::api_surfaces::g2api_set_root_surface(
                g2,
                view,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                &surface_name,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_SETSURFACEONOFF as c_int {
        unsafe {
            let surface_name = cstr_to_string(vma(view.common, args, 2) as *const c_char);
            mp_engine_ghoul2::api_surfaces::g2api_set_surface_on_off(
                g2,
                view,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                &surface_name,
                *args.offset(3) as c_int,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_SETNEWORIGIN as c_int {
        unsafe {
            mp_engine_ghoul2::api_bolts::g2api_set_new_origin(
                g2,
                view,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_GETTIME as c_int {
        mp_engine_ghoul2::api_collision::g2api_get_time(g2, 0)
    } else if trap == MpUiImport::UI_G2_SETTIME as c_int {
        unsafe {
            mp_engine_ghoul2::api_collision::g2api_set_time(
                g2,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            )
        };
        0
    } else if trap == MpUiImport::UI_G2_SETRAGDOLL as c_int
        || trap == MpUiImport::UI_G2_ANIMATEG2MODELS as c_int
    {
        // Raven: "not supported for ui" — both arms return 0.
        0
    } else if trap == MpUiImport::UI_G2_SETBONEIKSTATE as c_int {
        unsafe {
            // A NULL bone name is Raven's own contract: it tells
            // `G2_SetBoneIKState` to initialize the IK state on this instance.
            // Source: `oracle/codemp/ghoul2/G2_bones.cpp:4674-4676`
            let bone_name_ptr = vma(view.common, args, 3) as *const c_char;
            let bone_name = if bone_name_ptr.is_null() {
                None
            } else {
                Some(cstr_to_string(bone_name_ptr))
            };
            let params = (vma(view.common, args, 5) as *mut sharedSetBoneIKStateParams_t).as_mut();
            mp_engine_ghoul2::api_ragdoll::g2api_set_bone_ik_state(
                g2,
                view,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                bone_name.as_deref(),
                *args.offset(4) as c_int,
                params,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_IKMOVE as c_int {
        unsafe {
            let params = vma(view.common, args, 3) as *mut sharedIKMoveParams_t;
            mp_engine_ghoul2::api_ragdoll::g2api_ik_move(
                g2,
                view,
                &mut *(*args.offset(1) as *mut CGhoul2Info_v),
                *args.offset(2) as c_int,
                &mut *params,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_GETSURFACENAME as c_int {
        unsafe {
            let point = vma(view.common, args, 4) as *mut c_char;
            let model_index = *args.offset(3) as c_int;
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let ghl_info = g2_info(g2, ghoul2, model_index);
            let local = mp_engine_ghoul2::api_surfaces::g2api_get_surface_name(
                g2,
                view,
                ghl_info,
                *args.offset(2) as c_int,
            );
            if !local.is_empty() {
                let s = string_to_latin1(&local);
                core::ptr::copy_nonoverlapping(s.as_ptr(), point as *mut u8, s.len());
                *point.add(s.len()) = 0;
            }
        }
        0
    } else if trap == MpUiImport::UI_G2_SETSKIN as c_int {
        unsafe {
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let model_index = *args.offset(2) as c_int;
            let ghl_info = g2_info(g2, ghoul2, model_index);
            mp_engine_ghoul2::api_models::g2api_set_skin(
                g2,
                view,
                ghl_info,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
            ) as c_int
        }
    } else if trap == MpUiImport::UI_G2_ATTACHG2MODEL as c_int {
        unsafe {
            let g2_from = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let g2_to = &mut *(*args.offset(3) as *mut CGhoul2Info_v);
            mp_engine_ghoul2::api_bolts::g2api_attach_g2_model(
                g2,
                view,
                g2_from,
                *args.offset(2) as c_int,
                g2_to,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
            ) as c_int
        }
    } else {
        com_error(errorParm_t::ERR_DROP, format!("Bad UI system trap: {trap}"));
    }
}

/// Raven `void CL_InitUI( void )` — creates and initializes the `ui` VM.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:1462-1496`
pub fn CL_InitUI(view: &mut EngineHostView, cl: &mut Client) {
    let interpret = if cl.cl_connectedToPureServer != 0 {
        // Raven's `#if 0`-disabled `interpret = VMI_COMPILED;` branch never
        // runs; the live arm loads the module type the server used.
        unsafe { core::mem::transmute::<c_int, vmInterpret_t>(cl.cl_connectedUI) }
    } else {
        unsafe {
            core::mem::transmute::<c_int, vmInterpret_t>(
                Cvar_VariableValue(view.common, "vm_ui") as c_int
            )
        }
    };
    // The ui twin of the cgame pure-reference open (`CL_InitCGame`): the pak
    // that holds `uix86.dll` gains FS_UI_REF, which the pure reply lists second.
    {
        let mut h: fileHandle_t = 0;
        FS_FOpenFileRead(view, "uix86.dll", &mut h, false);
        if h != 0 {
            FS_FCloseFile(view.common, h);
        }
    }
    cl.uivm = VM_Create(view, "ui", Some(CL_UISystemCalls_trampoline), interpret);
    if cl.uivm.is_null() {
        com_error(errorParm_t::ERR_FATAL, "VM_Create on UI failed".to_string());
    }

    let v = VM_Call(
        view.common,
        cl.uivm,
        MpUiExport::UI_GETAPIVERSION as c_int,
        &[],
    );
    if v != UI_API_VERSION as isize {
        com_error(
            errorParm_t::ERR_DROP,
            format!("User Interface is version {v}, expected {UI_API_VERSION}"),
        );
        cl.cls.uiStarted = qfalse;
    } else {
        // rww - changed to <= CA_ACTIVE, because that is the state when we
        // did a vid_restart ingame (was just < CA_ACTIVE before, resulting in
        // ingame menus getting wiped and not reloaded on vid restart from
        // ingame menu).
        let ingame = cl.cls.state as c_int >= connstate_t::CA_AUTHORIZING as c_int
            && cl.cls.state as c_int <= connstate_t::CA_ACTIVE as c_int;
        VM_Call(
            view.common,
            cl.uivm,
            MpUiExport::UI_INIT as c_int,
            &[ingame as isize],
        );
    }
}

/// The `int (*)(int*)` C-ABI adapter handed to `VM_Create` as `systemCalls`
/// (`vm.cpp:471-472`, stored `vm->systemCall`). On the SEAM-D11 native path the
/// module reaches the engine through `ui_syscall_trampoline` → the armed ui
/// slot, so `vm->systemCall` (the legacy `VM_DllSyscall` target,
/// `vm.cpp:363-380`) is vestigial; this adapter widens the legacy contiguous
/// int arg block to the trampoline's `isize` words and forwards to the same
/// armed slot for parity if ever invoked. The `common`/`cl`/`g2` receivers come
/// from the boot-armed `ClientDispatchCtx` note, which `ui_system_calls_shim`
/// reads (DEC-55.1) — the twin of the server's `sv_game_system_call`.
///
/// Source: `oracle/codemp/client/cl_ui.cpp:813`
extern "C" fn CL_UISystemCalls_trampoline(args: *mut c_int) -> c_int {
    // SAFETY: the legacy `VM_DllSyscall` convention passes a contiguous 16-int
    // arg block (`args[i] = va_arg(...)`, vm.cpp:366).
    unsafe { client_legacy_syscall(args, ui_syscall_trampoline_words) }
}
