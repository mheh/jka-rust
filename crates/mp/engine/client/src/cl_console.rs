//! `cl_console.cpp` — the client console: scrollback buffer, notify lines,
//! and the drop-down/solid console draw path.
//!
//! Source: `oracle/codemp/client/cl_console.cpp`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};
use std::sync::Arc;

use libc::{strcat, strlen};

use mp_abi::cgame::exports::MpCgameExport;
use mp_bg::public::pmtype::pmtype_t;
use mp_engine_icarus::q3_interface::S_COLOR_RED;
use mp_engine_qcommon::cmd_common::{Cmd_Argc, Cmd_Argv};
use mp_engine_qcommon::cmd_pc::Cmd_AddCommand;
use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::common_fns::Com_Memcpy;
use mp_engine_qcommon::cvar_fns::Cvar_Get;
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileWrite, FS_Write};
use mp_engine_qcommon::stringed::api::{SE_GetString, SE_GetString2};
use mp_engine_qcommon::timing::sys_milliseconds;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_game::g_cmds::SAY_TEAM;
use mp_game::g_team::{COLOR_RED, COLOR_WHITE};
use mp_qshared::shared::connstate_t;
use mp_qshared::shared::limits::MAX_CLIENTS;
use mp_qshared::shared::q_color::Q_IsColorString;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use mp_renderer::hook_install::{re_from_view, rm_from_view};
use mp_renderer::tr_cmds::{RE_SetColor, RE_StretchPic};
use mp_renderer::tr_font::{
    GetLanguageEnum, Language_IsAsian, RE_Font_DrawString, RE_Font_HeightPixels, RE_RegisterFont,
};
use native_types::fileHandle_t;

use crate::cl_keys::{Field_BigDraw, Field_Clear, Field_Draw};
use crate::cl_main::CL_StartDemoLoop;
use crate::cl_scrn::{SCR_DrawBigString, SCR_DrawPic, SCR_DrawSmallChar};
use crate::client::console_t::NUM_CON_TIMES;
use crate::client_host::{cl_from_view, Client};
use crate::keys::key_globals_s::COMMAND_HISTORY;

/// Raven `#define KEYCATCH_CONSOLE 0x0001`.
// PORT-NOTE(consts): the packet lists KEYCATCH_* as unresolved, but the const
// already lives in `mp_qshared::shared::keycatch`; imported below instead of
// redefined.
use mp_qshared::shared::keycatch::{
    KEYCATCH_CGAME, KEYCATCH_CONSOLE, KEYCATCH_MESSAGE, KEYCATCH_UI,
};

/// `CON_TEXT_DUMP_USAGE`.
// PORT-NOTE(consts): no rosetta row yet for the usage-string key; the raw
// Raven literal is transcribed verbatim rather than invented.
const CON_TEXT_DUMP_USAGE: &str = "CON_TEXT_DUMP_USAGE";

/// `DEFAULT_CONSOLE_WIDTH`.
// PORT-NOTE(consts): unresolved in the rosetta; Raven's `q_shared.h` value (78)
// transcribed directly.
const DEFAULT_CONSOLE_WIDTH: c_int = 78;

/// `SMALLCHAR_WIDTH`/`SMALLCHAR_HEIGHT`/`BIGCHAR_WIDTH`/`BIGCHAR_HEIGHT`.
// PORT-NOTE(consts): unresolved in the rosetta for this crate; Raven's
// `cl_screen.h` values transcribed directly (BIGCHAR mirrors SMALLCHAR * 2).
const SMALLCHAR_WIDTH: c_int = 8;
const SMALLCHAR_HEIGHT: c_int = 16;
const BIGCHAR_WIDTH: c_int = 16;
const BIGCHAR_HEIGHT: c_int = 16;

/// `SCREEN_WIDTH`/`SCREEN_HEIGHT`.
// PORT-NOTE(consts): unresolved in the rosetta for this crate; Raven's
// `q_shared.h` design-resolution values transcribed directly.
const SCREEN_WIDTH: c_int = 640;
const SCREEN_HEIGHT: c_int = 480;

/// `Q3_VERSION`.
// PORT-NOTE(consts): unresolved in the rosetta; the build-version string is
// engine-generated, not a fixed literal, so this is a placeholder pending its
// rosetta row.
const Q3_VERSION: &str = "Q3_VERSION";

/// Raven `ColorIndex` macro — `((c) & 7)`.
///
/// Source: `oracle/codemp/game/q_shared.h`
fn ColorIndex(c: c_int) -> c_int {
    c & 7
}

/// Raven `Con_Dump_f` — writes the console scrollback to a file.
///
/// Source: `oracle/codemp/client/cl_console.cpp:142-194`
pub fn Con_Dump_f(view: &mut EngineHostView, cl: &mut Client) {
    // The receivers below reach `view.common` per DEC-59.1 (SE_GetString takes
    // the whole view; every other call needs only the `common` field).
    if Cmd_Argc(view.common) != 2 {
        let usage = SE_GetString(view, CON_TEXT_DUMP_USAGE);
        com_printf(view.common, &usage);
        return;
    }

    // The file name is read once, so the print and the open each hold the only
    // borrow of `view.common`.
    let file_name = Cmd_Argv(view.common, 1).to_string();
    com_printf(
        view.common,
        &format!("Dumped console text to {}.\n", file_name),
    );

    let f: fileHandle_t = FS_FOpenFileWrite(view.common, &file_name);
    if f == 0 {
        com_printf(
            view.common,
            &format!("{}ERROR: couldn't open.\n", S_COLOR_RED),
        );
        return;
    }

    let con = &mut cl.con;
    let mut buffer = [0u8; 1024];

    // skip empty lines
    let mut l = con.current - con.totallines + 1;
    while l <= con.current {
        let line_off = (l % con.totallines) * con.linewidth;
        let mut x = 0;
        while x < con.linewidth {
            if (con.text[(line_off + x) as usize] & 0xff) != b' ' as i16 {
                break;
            }
            x += 1;
        }
        if x != con.linewidth {
            break;
        }
        l += 1;
    }

    // write the remaining lines
    buffer[con.linewidth as usize] = 0;
    while l <= con.current {
        let line_off = (l % con.totallines) * con.linewidth;
        for i in 0..con.linewidth {
            buffer[i as usize] = (con.text[(line_off + i) as usize] & 0xff) as u8;
        }
        let mut x = con.linewidth - 1;
        while x >= 0 {
            if buffer[x as usize] == b' ' {
                buffer[x as usize] = 0;
            } else {
                break;
            }
            x -= 1;
        }
        unsafe {
            strcat(
                buffer.as_mut_ptr() as *mut c_char,
                b"\n\0".as_ptr() as *const c_char,
            );
            FS_Write(
                view.common,
                buffer.as_ptr() as *const (),
                strlen(buffer.as_ptr() as *const c_char) as c_int,
                f,
            );
        }
        l += 1;
    }

    FS_FCloseFile(view.common, f);
}

/// The `CmdFunction` adapter `Con_Init` registers for `Con_Dump_f`.
fn Con_Dump_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Con_Dump_f(view, cl)
}

/// Raven `Con_ClearNotify` — zeroes the notify-line timestamps.
///
/// Source: `oracle/codemp/client/cl_console.cpp:202-208`
pub fn Con_ClearNotify(cl: &mut Client) {
    for i in 0..NUM_CON_TIMES {
        cl.con.times[i] = 0;
    }
}

/// Raven `Con_Linefeed` — advances the console cursor to a new line.
///
/// Source: `oracle/codemp/client/cl_console.cpp:325-345`
pub fn Con_Linefeed(cl: &mut Client, silent: qboolean) {
    // mark time for transparent overlay
    if cl.con.current >= 0 && silent == qfalse {
        let idx = (cl.con.current % NUM_CON_TIMES as i32) as usize;
        cl.con.times[idx] = cl.cls.realtime;
    } else {
        let idx = (cl.con.current % NUM_CON_TIMES as i32) as usize;
        cl.con.times[idx] = 0;
    }

    cl.con.x = 0;
    if cl.con.display == cl.con.current {
        cl.con.display += 1;
    }
    cl.con.current += 1;
    for i in 0..cl.con.linewidth {
        let idx = ((cl.con.current % cl.con.totallines) * cl.con.linewidth + i) as usize;
        cl.con.text[idx] = ((ColorIndex(COLOR_WHITE) << 8) | (b' ' as i32)) as i16;
    }
}

/// Raven `Con_RunConsole` — scrolls the console height towards its target.
///
/// Source: `oracle/codemp/client/cl_console.cpp:771-793`
pub fn Con_RunConsole(common: &mut Common, cl: &mut Client) {
    if cl.cls.keyCatchers & KEYCATCH_CONSOLE != 0 {
        cl.con.finalFrac = 0.5;
    } else {
        cl.con.finalFrac = 0.0;
    }

    if cl.con.finalFrac < cl.con.displayFrac {
        cl.con.displayFrac -=
            common.cvar(cl.con_conspeed).value * (cl.cls.realFrametime as f32 * 0.001f32);
        if cl.con.finalFrac > cl.con.displayFrac {
            cl.con.displayFrac = cl.con.finalFrac;
        }
    } else if cl.con.finalFrac > cl.con.displayFrac {
        cl.con.displayFrac +=
            common.cvar(cl.con_conspeed).value * (cl.cls.realFrametime as f32 * 0.001f32);
        if cl.con.finalFrac < cl.con.displayFrac {
            cl.con.displayFrac = cl.con.finalFrac;
        }
    }
}

/// Raven `Con_PageUp` — scrolls the scrollback view up two lines.
///
/// Source: `oracle/codemp/client/cl_console.cpp:796-801`
pub fn Con_PageUp(cl: &mut Client) {
    cl.con.display -= 2;
    if cl.con.current - cl.con.display >= cl.con.totallines {
        cl.con.display = cl.con.current - cl.con.totallines + 1;
    }
}

/// Raven `Con_PageDown` — scrolls the scrollback view down two lines.
///
/// Source: `oracle/codemp/client/cl_console.cpp:803-808`
pub fn Con_PageDown(cl: &mut Client) {
    cl.con.display += 2;
    if cl.con.display > cl.con.current {
        cl.con.display = cl.con.current;
    }
}

/// Raven `Con_Top` — scrolls the scrollback view to the oldest line.
///
/// Source: `oracle/codemp/client/cl_console.cpp:810-815`
pub fn Con_Top(cl: &mut Client) {
    cl.con.display = cl.con.totallines;
    if cl.con.current - cl.con.display >= cl.con.totallines {
        cl.con.display = cl.con.current - cl.con.totallines + 1;
    }
}

/// Raven `Con_Bottom` — scrolls the scrollback view to the newest line.
///
/// Source: `oracle/codemp/client/cl_console.cpp:817-819`
pub fn Con_Bottom(cl: &mut Client) {
    cl.con.display = cl.con.current;
}

/// Raven `Con_ToggleConsole_f` — the `toggleconsole` command handler.
///
/// Source: `oracle/codemp/client/cl_console.cpp:29-41`
pub fn Con_ToggleConsole_f(common: &mut Common, cl: &mut Client) {
    // closing a full screen console restarts the demo loop
    if cl.cls.state == connstate_t::CA_DISCONNECTED && cl.cls.keyCatchers == KEYCATCH_CONSOLE {
        CL_StartDemoLoop(common, cl);
        return;
    }

    Field_Clear(&mut cl.kg.g_consoleField);
    cl.kg.g_consoleField.widthInChars = cl.g_console_field_width;

    Con_ClearNotify(cl);
    cl.cls.keyCatchers ^= KEYCATCH_CONSOLE;
}

/// The `CmdFunction` adapter `Con_Init` registers, which casts the view's `cl`
/// slot back to the handler's real receiver (`sv_ccmds.rs` forwarder idiom).
fn Con_ToggleConsole_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Con_ToggleConsole_f(view.common, cl)
}

/// Raven `Con_MessageMode_f` — the `messagemode` (yell) command handler.
///
/// Source: `oracle/codemp/client/cl_console.cpp:49-56`
pub fn Con_MessageMode_f(cl: &mut Client) {
    cl.chat_playerNum = -1;
    cl.chat_team = qfalse;
    Field_Clear(&mut cl.chatField);
    cl.chatField.widthInChars = 30;

    cl.cls.keyCatchers ^= KEYCATCH_MESSAGE;
}

/// The `CmdFunction` adapter `Con_Init` registers for `Con_MessageMode_f`.
fn Con_MessageMode_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Con_MessageMode_f(cl)
}

/// Raven `Con_MessageMode2_f` — the `messagemode2` (team chat) command handler.
///
/// Source: `oracle/codemp/client/cl_console.cpp:63-69`
pub fn Con_MessageMode2_f(cl: &mut Client) {
    cl.chat_playerNum = -1;
    cl.chat_team = qtrue;
    Field_Clear(&mut cl.chatField);
    cl.chatField.widthInChars = 25;
    cl.cls.keyCatchers ^= KEYCATCH_MESSAGE;
}

/// The `CmdFunction` adapter `Con_Init` registers for `Con_MessageMode2_f`.
fn Con_MessageMode2_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Con_MessageMode2_f(cl)
}

/// Raven `Con_MessageMode3_f` — the `messagemode3` (target chat) command handler.
///
/// Source: `oracle/codemp/client/cl_console.cpp:76-93`
pub fn Con_MessageMode3_f(common: &mut Common, cl: &mut Client) {
    if cl.cgvm.is_null() {
        debug_assert!(false, "null cgvm");
        return;
    }

    cl.chat_playerNum = VM_Call(
        common,
        cl.cgvm,
        MpCgameExport::CG_CROSSHAIR_PLAYER as c_int,
        &[],
    ) as c_int;
    if cl.chat_playerNum < 0 || cl.chat_playerNum >= MAX_CLIENTS as c_int {
        cl.chat_playerNum = -1;
        return;
    }
    cl.chat_team = qfalse;
    Field_Clear(&mut cl.chatField);
    cl.chatField.widthInChars = 30;
    cl.cls.keyCatchers ^= KEYCATCH_MESSAGE;
}

/// The `CmdFunction` adapter `Con_Init` registers for `Con_MessageMode3_f`.
fn Con_MessageMode3_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Con_MessageMode3_f(view.common, cl)
}

/// Raven `Con_MessageMode4_f` — the `messagemode4` (attacker chat) command handler.
///
/// Source: `oracle/codemp/client/cl_console.cpp:100-117`
pub fn Con_MessageMode4_f(common: &mut Common, cl: &mut Client) {
    if cl.cgvm.is_null() {
        debug_assert!(false, "null cgvm");
        return;
    }

    cl.chat_playerNum = VM_Call(
        common,
        cl.cgvm,
        MpCgameExport::CG_LAST_ATTACKER as c_int,
        &[],
    ) as c_int;
    if cl.chat_playerNum < 0 || cl.chat_playerNum >= MAX_CLIENTS as c_int {
        cl.chat_playerNum = -1;
        return;
    }
    cl.chat_team = qfalse;
    Field_Clear(&mut cl.chatField);
    cl.chatField.widthInChars = 30;
    cl.cls.keyCatchers ^= KEYCATCH_MESSAGE;
}

/// The `CmdFunction` adapter `Con_Init` registers for `Con_MessageMode4_f`.
fn Con_MessageMode4_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Con_MessageMode4_f(view.common, cl)
}

/// Raven `Con_Clear_f` — the `clear` command handler.
///
/// Source: `oracle/codemp/client/cl_console.cpp:124-132`
pub fn Con_Clear_f(cl: &mut Client) {
    for i in 0..crate::client::console_t::CON_TEXTSIZE {
        cl.con.text[i] = ((ColorIndex(COLOR_WHITE) << 8) | (b' ' as i32)) as i16;
    }

    Con_Bottom(cl);
}

/// The `CmdFunction` adapter `Con_Init` registers for `Con_Clear_f`.
fn Con_Clear_f_cmd(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Con_Clear_f(cl)
}

/// Raven `Con_CheckResize` — resizes the scrollback buffer for the current
/// video width, reflowing existing lines into the new line width.
///
/// Source: `oracle/codemp/client/cl_console.cpp:219-284`
pub fn Con_CheckResize(cl: &mut Client) {
    // §19: `tbuf` is a Raven local Raven fills before reading back (via
    // `Com_Memcpy` from `con.text`), so the zero-init below is not
    // load-bearing, but it keeps the local well-defined either way.
    let mut tbuf = [0i16; crate::client::console_t::CON_TEXTSIZE];

    let width = (cl.cls.glconfig.vidWidth / SMALLCHAR_WIDTH) - 2;

    if width == cl.con.linewidth {
        return;
    }

    if width < 1 {
        // video hasn't been initialized yet
        cl.con.xadjust = 1.0;
        cl.con.yadjust = 1.0;
        let width = DEFAULT_CONSOLE_WIDTH;
        cl.con.linewidth = width;
        cl.con.totallines = crate::client::console_t::CON_TEXTSIZE as i32 / cl.con.linewidth;
        for i in 0..crate::client::console_t::CON_TEXTSIZE {
            cl.con.text[i] = ((ColorIndex(COLOR_WHITE) << 8) | (b' ' as i32)) as i16;
        }
    } else {
        // on wide screens, we will center the text
        cl.con.xadjust = 640.0f32 / cl.cls.glconfig.vidWidth as f32;
        cl.con.yadjust = 480.0f32 / cl.cls.glconfig.vidHeight as f32;

        let oldwidth = cl.con.linewidth;
        cl.con.linewidth = width;
        let oldtotallines = cl.con.totallines;
        cl.con.totallines = crate::client::console_t::CON_TEXTSIZE as i32 / cl.con.linewidth;
        let mut numlines = oldtotallines;

        if cl.con.totallines < numlines {
            numlines = cl.con.totallines;
        }

        let mut numchars = oldwidth;

        if cl.con.linewidth < numchars {
            numchars = cl.con.linewidth;
        }

        unsafe {
            Com_Memcpy(
                tbuf.as_mut_ptr() as *mut (),
                cl.con.text.as_ptr() as *const (),
                crate::client::console_t::CON_TEXTSIZE * core::mem::size_of::<i16>(),
            );
        }
        for i in 0..crate::client::console_t::CON_TEXTSIZE {
            cl.con.text[i] = ((ColorIndex(COLOR_WHITE) << 8) | (b' ' as i32)) as i16;
        }

        for i in 0..numlines {
            for j in 0..numchars {
                let dst = ((cl.con.totallines - 1 - i) * cl.con.linewidth + j) as usize;
                let src = (((cl.con.current - i + oldtotallines) % oldtotallines) * oldwidth + j)
                    as usize;
                cl.con.text[dst] = tbuf[src];
            }
        }

        Con_ClearNotify(cl);
    }

    cl.con.current = cl.con.totallines - 1;
    cl.con.display = cl.con.current;
}

/// Raven `Con_Close` — hides the console and clears its notify state.
///
/// Source: `oracle/codemp/client/cl_console.cpp:822-831`
pub fn Con_Close(common: &mut Common, cl: &mut Client) {
    // `com_cl_running` is a `Common` file-scope cvar handle, not a `Client`
    // field (CLIENT CARRIER RULE).
    if common.cvar(common.com_cl_running).integer == 0 {
        return;
    }
    Field_Clear(&mut cl.kg.g_consoleField);
    Con_ClearNotify(cl);
    cl.cls.keyCatchers &= !KEYCATCH_CONSOLE;
    cl.con.finalFrac = 0.0;
    cl.con.displayFrac = 0.0;
}

/// Raven `Con_Init` — registers console cvars, edit fields, and console
/// commands.
///
/// Source: `oracle/codemp/client/cl_console.cpp:292-317`
pub fn Con_Init(view: &mut EngineHostView, cl: &mut Client) {
    cl.con_notifytime = Some(Cvar_Get(view, "con_notifytime", "3", 0));
    cl.con_conspeed = Some(Cvar_Get(view, "scr_conspeed", "3", 0));

    Field_Clear(&mut cl.kg.g_consoleField);
    cl.kg.g_consoleField.widthInChars = cl.g_console_field_width;
    for i in 0..COMMAND_HISTORY {
        Field_Clear(&mut cl.kg.historyEditLines[i]);
        cl.kg.historyEditLines[i].widthInChars = cl.g_console_field_width;
    }

    // No console on Xbox is not modeled here (`_XBOX` never defined for this port).
    Cmd_AddCommand(view, "toggleconsole", Some(Con_ToggleConsole_f_cmd));
    Cmd_AddCommand(view, "messagemode", Some(Con_MessageMode_f_cmd));
    Cmd_AddCommand(view, "messagemode2", Some(Con_MessageMode2_f_cmd));
    Cmd_AddCommand(view, "messagemode3", Some(Con_MessageMode3_f_cmd));
    Cmd_AddCommand(view, "messagemode4", Some(Con_MessageMode4_f_cmd));
    Cmd_AddCommand(view, "clear", Some(Con_Clear_f_cmd));
    Cmd_AddCommand(view, "condump", Some(Con_Dump_f_cmd));

    // Initialize values on first print
    cl.con.initialized = qfalse;
}

/// Raven `CL_ConsolePrint` — appends colored text to the console scrollback,
/// word-wrapping at `con.linewidth`.
///
/// Source: `oracle/codemp/client/cl_console.cpp:356-433`
pub fn CL_ConsolePrint(common: &mut Common, cl: &mut Client, txt: *const c_char, silent: qboolean) {
    if !cl.cl_noprint.is_none() && common.cvar(cl.cl_noprint).integer != 0 {
        return;
    }

    if cl.con.initialized == qfalse {
        cl.con.color[0] = 1.0;
        cl.con.color[1] = 1.0;
        cl.con.color[2] = 1.0;
        cl.con.color[3] = 1.0;
        cl.con.linewidth = -1;
        Con_CheckResize(cl);
        cl.con.initialized = qtrue;
    }

    let mut color = ColorIndex(COLOR_WHITE);

    unsafe {
        let mut p = txt;
        while *p != 0 {
            let c = *p as u8 as c_int;

            if Q_IsColorString(core::slice::from_raw_parts(p as *const u8, 2)) {
                color = ColorIndex(*p.offset(1) as c_int);
                p = p.offset(2);
                continue;
            }

            // count word length
            let mut l = 0;
            while l < cl.con.linewidth {
                if *p.offset(l as isize) as u8 as c_int <= b' ' as c_int {
                    break;
                }
                l += 1;
            }

            // word wrap
            if l != cl.con.linewidth && (cl.con.x + l >= cl.con.linewidth) {
                Con_Linefeed(cl, silent);
            }

            p = p.offset(1);

            match c as u8 {
                b'\n' => {
                    Con_Linefeed(cl, silent);
                }
                b'\r' => {
                    cl.con.x = 0;
                }
                _ => {
                    // display character and advance
                    let y = cl.con.current % cl.con.totallines;
                    let idx = (y * cl.con.linewidth + cl.con.x) as usize;
                    cl.con.text[idx] = ((color << 8) | c) as i16;
                    cl.con.x += 1;
                    if cl.con.x >= cl.con.linewidth {
                        Con_Linefeed(cl, silent);
                        cl.con.x = 0;
                    }
                }
            }
        }
    }

    // mark time for transparent overlay
    if cl.con.current >= 0 && silent == qfalse {
        let idx = (cl.con.current % NUM_CON_TIMES as i32) as usize;
        cl.con.times[idx] = cl.cls.realtime;
    } else {
        let idx = (cl.con.current % NUM_CON_TIMES as i32) as usize;
        cl.con.times[idx] = 0;
    }
}

/// Raven `Con_DrawInput` — draws the console edit line and cursor.
///
/// Source: `oracle/codemp/client/cl_console.cpp:452-467`
pub fn Con_DrawInput(view: &mut EngineHostView, cl: &mut Client) {
    if cl.cls.state != connstate_t::CA_DISCONNECTED && (cl.cls.keyCatchers & KEYCATCH_CONSOLE) == 0
    {
        return;
    }

    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let re = unsafe { re_from_view(view) };
    let eLanguage = GetLanguageEnum(view.common, &mut re.font);
    let asian_scale = if Language_IsAsian(eLanguage) {
        1.5f32
    } else {
        2.0f32
    };
    let y = cl.con.vislines - (SMALLCHAR_HEIGHT as f32 * asian_scale) as c_int;

    RE_SetColor(&mut re.frame_data, Some(cl.con.color));

    SCR_DrawSmallChar(
        view,
        cl,
        (cl.con.xadjust + 1.0 * SMALLCHAR_WIDTH as f32) as c_int,
        y,
        b']' as c_int,
    );

    // The field pointer is taken first, so the `cl` receiver stays free for the call.
    let edit: *mut _ = &mut cl.kg.g_consoleField;
    let x = (cl.con.xadjust + 2.0 * SMALLCHAR_WIDTH as f32) as c_int;
    Field_Draw(
        view,
        cl,
        edit,
        x,
        y,
        SCREEN_WIDTH - 3 * SMALLCHAR_WIDTH,
        qtrue,
    );
}

/// Raven `Con_DrawNotify` — draws the transparent notify overlay above the
/// game view, and the active chat-message edit line.
///
/// Source: `oracle/codemp/client/cl_console.cpp:479-592`
pub fn Con_DrawNotify(view: &mut EngineHostView, cl: &mut Client) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let re = unsafe { re_from_view(view) };
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };

    let mut current_color = 7;
    RE_SetColor(&mut re.frame_data, g_color_table_ptr(current_color));

    let mut v = 0;
    let mut i = cl.con.current - NUM_CON_TIMES as i32 + 1;
    while i <= cl.con.current {
        if i < 0 {
            i += 1;
            continue;
        }
        let time_idx = (i % NUM_CON_TIMES as i32) as usize;
        let mut time = cl.con.times[time_idx];
        if time == 0 {
            i += 1;
            continue;
        }
        time = cl.cls.realtime - time;
        if time as f32 > view.common.cvar(cl.con_notifytime).value * 1000.0 {
            i += 1;
            continue;
        }
        let text_off = (i % cl.con.totallines) * cl.con.linewidth;

        if cl.cl.snap.ps.pm_type != pmtype_t::PM_INTERMISSION as c_int
            && (cl.cls.keyCatchers & (KEYCATCH_UI | KEYCATCH_CGAME)) != 0
        {
            i += 1;
            continue;
        }

        if cl.cl_conXOffset.is_none() {
            cl.cl_conXOffset = Some(Cvar_Get(view, "cl_conXOffset", "0", 0));
        }

        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let eLanguage = GetLanguageEnum(view.common, &mut re.font);

        // asian language needs to use the new font system to print glyphs...
        // (ignore colours since we're going to print the whole thing as one string)
        if Language_IsAsian(eLanguage) {
            let font_scale = 0.75f32 * cl.con.yadjust;
            let millis = sys_milliseconds(view.common);
            let iSE_Language_ModificationCount =
                re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
            // Raven caches this registration in a fn-scope `static int
            // iFontIndex` (`oracle/codemp/client/cl_console.cpp:521`), a
            // hidden global (porting-rules §B3). `Client` has no slot for it
            // yet, so this re-registers "ocr_a" on every call; `RE_RegisterFont`
            // is idempotent per name, so this only costs a lookup, not a
            // behavior change.
            let font_index_asian_notify = RE_RegisterFont(
                &mut re.qs,
                &mut re.frame,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
                &mut re.font,
                eLanguage,
                iSE_Language_ModificationCount,
                "ocr_a",
            );
            let pixel_height_to_advance = 2
                + ((1.3f32 / cl.con.yadjust)
                    * RE_Font_HeightPixels(
                        &mut re.qs,
                        &mut re.frame,
                        Arc::make_mut(&mut re.sim.published),
                        view,
                        &re.cvars,
                        rm,
                        &mut re.img_state,
                        &mut re.sky_view,
                        &mut re.sky,
                        &mut re.font,
                        eLanguage,
                        iSE_Language_ModificationCount,
                        font_index_asian_notify,
                        font_scale,
                    ) as f32) as c_int;

            // concat the text to be printed...
            let mut s_temp = String::new();
            for x in 0..cl.con.linewidth {
                let ch = cl.con.text[(text_off + x) as usize];
                if ((ch >> 8) & 7) as c_int != current_color {
                    current_color = ((ch >> 8) & 7) as c_int;
                    s_temp.push_str(&format!("^{}", current_color));
                }
                s_temp.push((ch & 0xff) as u8 as char);
            }

            RE_Font_DrawString(
                &mut re.qs,
                &mut re.frame,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
                &mut re.font,
                eLanguage,
                iSE_Language_ModificationCount,
                &mut re.frame_data,
                view.common.cvar(cl.cl_conXOffset).integer
                    + (cl.con.xadjust * (cl.con.xadjust + (1.0 * SMALLCHAR_WIDTH as f32))) as c_int,
                (cl.con.yadjust * v as f32) as c_int,
                s_temp.as_bytes(),
                g_color_table_ptr(current_color),
                font_index_asian_notify,
                -1,
                font_scale,
                millis,
            );

            v += pixel_height_to_advance;
        } else {
            for x in 0..cl.con.linewidth {
                let ch = cl.con.text[(text_off + x) as usize];
                if (ch & 0xff) as u8 == b' ' {
                    continue;
                }
                if ((ch >> 8) & 7) as c_int != current_color {
                    current_color = ((ch >> 8) & 7) as c_int;
                    let re = unsafe { re_from_view(view) };
                    RE_SetColor(&mut re.frame_data, g_color_table_ptr(current_color));
                }
                if cl.cl_conXOffset.is_none() {
                    cl.cl_conXOffset = Some(Cvar_Get(view, "cl_conXOffset", "0", 0));
                }
                SCR_DrawSmallChar(
                    view,
                    cl,
                    view.common.cvar(cl.cl_conXOffset).integer
                        + cl.con.xadjust as c_int
                        + (x + 1) * SMALLCHAR_WIDTH,
                    v,
                    (ch & 0xff) as c_int,
                );
            }

            v += SMALLCHAR_HEIGHT;
        }
        i += 1;
    }

    let re = unsafe { re_from_view(view) };
    RE_SetColor(&mut re.frame_data, None);

    if (cl.cls.keyCatchers & (KEYCATCH_UI | KEYCATCH_CGAME)) != 0 {
        return;
    }

    // draw the chat line
    if (cl.cls.keyCatchers & KEYCATCH_MESSAGE) != 0 {
        // The Raven call site passes a package + key pair, so `SE_GetString2`
        // is the shape match (see shape_mismatches).
        let (chattext, skip);
        if cl.chat_team == qtrue {
            chattext = SE_GetString2(view, "MP_SVGAME", "SAY_TEAM");
            let chattext_c = std::ffi::CString::new(chattext.clone()).unwrap_or_default();
            SCR_DrawBigString(view, cl, 8, v, chattext_c.as_ptr(), 1.0);
            skip = chattext.len() as c_int + 1;
        } else {
            chattext = SE_GetString2(view, "MP_SVGAME", "SAY");
            let chattext_c = std::ffi::CString::new(chattext.clone()).unwrap_or_default();
            SCR_DrawBigString(view, cl, 8, v, chattext_c.as_ptr(), 1.0);
            skip = chattext.len() as c_int + 1;
        }

        // The field pointer is taken first, so the `cl` receiver stays free for the call.
        let edit: *mut _ = &mut cl.chatField;
        Field_BigDraw(
            view,
            cl,
            edit,
            skip * BIGCHAR_WIDTH,
            v,
            SCREEN_WIDTH - (skip + 1) * BIGCHAR_WIDTH,
            qtrue,
        );

        v += BIGCHAR_HEIGHT;
    }
    let _ = SAY_TEAM;
    let _ = v;
}

/// Raven `g_color_table[i]` lookup — `RE_SetColor`/`RE_Font_DrawString` take
/// an `Option<[f32; 4]>` rgba, the port's nullable-color model.
// PORT-NOTE(state): `g_color_table` is `q_shared.h`'s shared 8-entry color
// ramp, not yet threaded to a receiver reachable here; resolved against
// `mp_qshared`'s copy at integration.
fn g_color_table_ptr(_index: c_int) -> Option<[f32; 4]> {
    None
}

/// Raven `Con_DrawSolidConsole` — draws the full drop-down console at the
/// given screen fraction.
///
/// Source: `oracle/codemp/client/cl_console.cpp:601-731`
pub fn Con_DrawSolidConsole(view: &mut EngineHostView, cl: &mut Client, frac: f32) {
    let mut lines = (cl.cls.glconfig.vidHeight as f32 * frac) as c_int;
    if lines <= 0 {
        return;
    }

    if lines > cl.cls.glconfig.vidHeight {
        lines = cl.cls.glconfig.vidHeight;
    }

    // draw the background
    let mut y = (frac * SCREEN_HEIGHT as f32 - 2.0) as c_int;
    if y < 1 {
        y = 0;
    } else {
        SCR_DrawPic(
            view,
            cl,
            0.0,
            0.0,
            SCREEN_WIDTH as f32,
            y as f32,
            cl.cls.consoleShader,
        );
    }

    let color: [f32; 4] = [0.509f32, 0.609f32, 0.847f32, 1.0f32];
    // draw the bottom bar and version number

    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let re = unsafe { re_from_view(view) };
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let rm = unsafe { rm_from_view(view) };

    RE_SetColor(&mut re.frame_data, Some(color));
    RE_StretchPic(
        &mut re.frame_data,
        &re.sim.published,
        view.common,
        0.0,
        y as f32,
        SCREEN_WIDTH as f32,
        2.0,
        0.0,
        0.0,
        0.0,
        0.0,
        cl.cls.whiteShader,
    );

    let i = Q3_VERSION.len() as c_int;

    for x in 0..i {
        SCR_DrawSmallChar(
            view,
            cl,
            cl.cls.glconfig.vidWidth - (i - x) * SMALLCHAR_WIDTH,
            lines - (SMALLCHAR_HEIGHT + SMALLCHAR_HEIGHT / 2),
            Q3_VERSION.as_bytes()[x as usize] as c_int,
        );
    }

    // draw the text
    cl.con.vislines = lines;
    let mut rows = (lines - SMALLCHAR_WIDTH) / SMALLCHAR_WIDTH;

    y = lines - (SMALLCHAR_HEIGHT * 3);

    // draw from the bottom up
    if cl.con.display != cl.con.current {
        // draw arrows to show the buffer is backscrolled
        let re = unsafe { re_from_view(view) };
        RE_SetColor(&mut re.frame_data, g_color_table_ptr(ColorIndex(COLOR_RED)));
        let mut x = 0;
        while x < cl.con.linewidth {
            SCR_DrawSmallChar(
                view,
                cl,
                (cl.con.xadjust + (x + 1) as f32 * SMALLCHAR_WIDTH as f32) as c_int,
                y,
                b'^' as c_int,
            );
            x += 4;
        }
        y -= SMALLCHAR_HEIGHT;
        rows -= 1;
    }

    let mut row = cl.con.display;

    if cl.con.x == 0 {
        row -= 1;
    }

    let mut current_color = 7;
    let re = unsafe { re_from_view(view) };
    RE_SetColor(&mut re.frame_data, g_color_table_ptr(current_color));

    // PORT-NOTE(statics): `iFontIndexForAsian` is genuine cross-frame state
    // (the font registers once, then the handle is reused), so it lives on
    // `cl` (fork-3 three-kind rule, kind 3) rather than a hidden static.
    let font_scale_asian = 0.75f32 * cl.con.yadjust;
    let millis = sys_milliseconds(view.common);
    let mut pixel_height_to_advance = SMALLCHAR_HEIGHT;
    let eLanguage = GetLanguageEnum(view.common, &mut re.font);
    let iSE_Language_ModificationCount = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
    if Language_IsAsian(eLanguage) {
        if cl.iFontIndexForAsian == 0 {
            cl.iFontIndexForAsian = RE_RegisterFont(
                &mut re.qs,
                &mut re.frame,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
                &mut re.font,
                eLanguage,
                iSE_Language_ModificationCount,
                "ocr_a",
            );
        }
        pixel_height_to_advance = ((1.3f32 / cl.con.yadjust)
            * RE_Font_HeightPixels(
                &mut re.qs,
                &mut re.frame,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
                &mut re.font,
                eLanguage,
                iSE_Language_ModificationCount,
                cl.iFontIndexForAsian,
                font_scale_asian,
            ) as f32) as c_int;
    }

    let mut i = 0;
    while i < rows {
        if row < 0 {
            break;
        }
        if cl.con.current - row >= cl.con.totallines {
            // past scrollback wrap point
            i += 1;
            y -= pixel_height_to_advance;
            row -= 1;
            continue;
        }

        let text_off = (row % cl.con.totallines) * cl.con.linewidth;

        // asian language needs to use the new font system to print glyphs...
        // (ignore colours since we're going to print the whole thing as one string)
        if Language_IsAsian(eLanguage) {
            // concat the text to be printed...
            let mut s_temp = String::new();
            for x in 0..cl.con.linewidth {
                let ch = cl.con.text[(text_off + x) as usize];
                if ((ch >> 8) & 7) as c_int != current_color {
                    current_color = ((ch >> 8) & 7) as c_int;
                    s_temp.push_str(&format!("^{}", current_color));
                }
                s_temp.push((ch & 0xff) as u8 as char);
            }

            RE_Font_DrawString(
                &mut re.qs,
                &mut re.frame,
                Arc::make_mut(&mut re.sim.published),
                view,
                &re.cvars,
                rm,
                &mut re.img_state,
                &mut re.sky_view,
                &mut re.sky,
                &mut re.font,
                eLanguage,
                iSE_Language_ModificationCount,
                &mut re.frame_data,
                (cl.con.xadjust * (cl.con.xadjust + (1.0 * SMALLCHAR_WIDTH as f32))) as c_int,
                (cl.con.yadjust * y as f32) as c_int,
                s_temp.as_bytes(),
                g_color_table_ptr(current_color),
                cl.iFontIndexForAsian,
                -1,
                font_scale_asian,
                millis,
            );
        } else {
            for x in 0..cl.con.linewidth {
                let ch = cl.con.text[(text_off + x) as usize];
                if (ch & 0xff) as u8 == b' ' {
                    continue;
                }

                if ((ch >> 8) & 7) as c_int != current_color {
                    current_color = ((ch >> 8) & 7) as c_int;
                    RE_SetColor(&mut re.frame_data, g_color_table_ptr(current_color));
                }
                SCR_DrawSmallChar(
                    view,
                    cl,
                    (cl.con.xadjust + (x + 1) as f32 * SMALLCHAR_WIDTH as f32) as c_int,
                    y,
                    (ch & 0xff) as c_int,
                );
            }
        }

        i += 1;
        y -= pixel_height_to_advance;
        row -= 1;
    }

    // draw the input prompt, user text, and cursor if desired
    Con_DrawInput(view, cl);

    let re = unsafe { re_from_view(view) };
    RE_SetColor(&mut re.frame_data, None);
}

/// Raven `Con_DrawConsole` — the top-level console draw dispatch: solid
/// full-screen console, drop-down console, or the notify overlay.
///
/// Source: `oracle/codemp/client/cl_console.cpp:740-760`
pub fn Con_DrawConsole(view: &mut EngineHostView, cl: &mut Client) {
    // check for console width changes from a vid mode change
    Con_CheckResize(cl);

    // if disconnected, render console full screen
    if cl.cls.state == connstate_t::CA_DISCONNECTED {
        if (cl.cls.keyCatchers & (KEYCATCH_UI | KEYCATCH_CGAME)) == 0 {
            Con_DrawSolidConsole(view, cl, 1.0);
            return;
        }
    }

    if cl.con.displayFrac != 0.0 {
        Con_DrawSolidConsole(view, cl, cl.con.displayFrac);
    } else {
        // draw notify lines
        if cl.cls.state == connstate_t::CA_ACTIVE {
            Con_DrawNotify(view, cl);
        }
    }
}
