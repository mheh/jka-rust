//! `cl_scrn.cpp` — screen drawing: named pics, small/big text, the debug
//! graph, center-print, and the top-level screen field / update-screen pump.
//!
//! Source: `oracle/codemp/client/cl_scrn.cpp`

#![allow(non_snake_case, non_camel_case_types, unused_variables, unused_mut)]

use core::ffi::{c_char, c_int};

use crate::cl_console::Con_ClearNotify;
use mp_abi::ui::exports::MpUiExport;
use mp_abi::ui::public::ui_menu_command_t::UIMENU_MAIN;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{Com_DPrintf, Com_Memcpy};
use mp_engine_qcommon::cvar_fns::Cvar_Get;
use mp_engine_qcommon::files_pc::FS_FTell;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_qshared::common::mp::cgame::stereo_frame_t::{
    stereoFrame_t, STEREO_CENTER, STEREO_LEFT, STEREO_RIGHT,
};
use mp_qshared::shared::connstate::connstate_t;
use mp_qshared::shared::cvar::CVAR_CHEAT;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_color::{g_color_table, Q_IsColorString};
use native_types::qhandle_t;

use crate::client_host::Client;

/// Raven `SCR_DrawNamedPic`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:24-31`
pub fn SCR_DrawNamedPic(
    cl: &mut Client,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    picname: *const c_char,
) {
    assert!(width != 0.0);

    // PORT-NOTE(missing-symbol): `refexport_t::RegisterShader` is not in the tree yet.
    // Source: oracle/codemp/client/client.h:388
    let hShader: qhandle_t = crate::cl_renderer::re(cl).RegisterShader(picname);
    // PORT-NOTE(missing-symbol): `refexport_t::DrawStretchPic` is not in the tree yet.
    // Source: oracle/codemp/client/client.h:388
    crate::cl_renderer::re(cl).DrawStretchPic(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}

/// Raven `SCR_FillRect`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:41-47`
pub fn SCR_FillRect(cl: &mut Client, x: f32, y: f32, width: f32, height: f32, color: *const f32) {
    // PORT-NOTE(missing-symbol): `refexport_t::SetColor` is not in the tree yet.
    // Source: oracle/codemp/client/client.h:388
    crate::cl_renderer::re(cl).SetColor(color);

    crate::cl_renderer::re(cl).DrawStretchPic(
        x,
        y,
        width,
        height,
        0.0,
        0.0,
        0.0,
        0.0,
        cl.cls.whiteShader,
    );

    crate::cl_renderer::re(cl).SetColor(core::ptr::null());
}

/// Raven `SCR_DrawPic`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:57-59`
pub fn SCR_DrawPic(cl: &mut Client, x: f32, y: f32, width: f32, height: f32, hShader: qhandle_t) {
    crate::cl_renderer::re(cl).DrawStretchPic(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}

/// Raven `SCR_DrawChar` (Raven `static`, file-private in the oracle).
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:67-101`
pub fn SCR_DrawChar(cl: &mut Client, x: c_int, y: c_int, size: f32, ch: c_int) {
    let mut ch = ch & 255;

    if ch == ' ' as c_int {
        return;
    }

    if (y as f32) < -size {
        return;
    }

    let ax = x as f32;
    let ay = y as f32;
    let aw = size;
    let ah = size;

    let row = ch >> 4;
    let col = ch & 15;

    let frow = row as f32 * (0.0625f32 as f64) as f32;
    let fcol = col as f32 * (0.0625f32 as f64) as f32;
    let size = 0.03125f32 as f64 as f32;
    let size2 = 0.0625f32 as f64 as f32;

    crate::cl_renderer::re(cl).DrawStretchPic(
        ax,
        ay,
        aw,
        ah,
        fcol,
        frow,
        fcol + size,
        frow + size2,
        cl.cls.charSetShader,
    );
}

/// Raven `SCR_DrawSmallChar`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:107-142`
pub fn SCR_DrawSmallChar(cl: &mut Client, x: c_int, y: c_int, ch: c_int) {
    let ch = ch & 255;

    if ch == ' ' as c_int {
        return;
    }

    // PORT-NOTE(missing-symbol): `SMALLCHAR_HEIGHT` is not in the tree yet.
    // Source: oracle/codemp/client/cl_scrn.h
    if (y as f32) < -(SMALLCHAR_HEIGHT as f32) {
        return;
    }

    let row = ch >> 4;
    let col = ch & 15;

    let frow = row as f32 * (0.0625f32 as f64) as f32;
    let fcol = col as f32 * (0.0625f32 as f64) as f32;

    // Raven's `_JK2` branch never applies to the MP tree; the else arm is Raven's live value.
    let size = 0.0625f32 as f64 as f32;
    let size2 = 0.0625f32 as f64 as f32;

    crate::cl_renderer::re(cl).DrawStretchPic(
        x as f32 * cl.con.xadjust,
        y as f32 * cl.con.yadjust,
        // PORT-NOTE(missing-symbol): `SMALLCHAR_WIDTH` is not in the tree yet.
        // Source: oracle/codemp/client/cl_scrn.h
        SMALLCHAR_WIDTH as f32 * cl.con.xadjust,
        SMALLCHAR_HEIGHT as f32 * cl.con.yadjust,
        fcol,
        frow,
        fcol + size,
        frow + size2,
        cl.cls.charSetShader,
    );
}

/// Raven `SCR_Strlen` (Raven `static`, file-private in the oracle).
///
/// This counts printable characters and skips over `^N` color escapes.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:253-267`
pub fn SCR_Strlen(str: *const c_char) -> c_int {
    let mut s = str;
    let mut count: c_int = 0;

    unsafe {
        while *s != 0 {
            let pair = core::slice::from_raw_parts(s as *const u8, 2.min(strlen_remaining(s)));
            if Q_IsColorString(pair) {
                s = s.add(2);
            } else {
                count += 1;
                s = s.add(1);
            }
        }
    }

    count
}

/// Raven's `Q_IsColorString` reads up to two bytes ahead of `s`; this bounds the
/// slice to the remaining NUL-terminated run so the second byte read never
/// crosses the terminator.
fn strlen_remaining(s: *const c_char) -> usize {
    let mut n = 0usize;
    unsafe {
        while *s.add(n) != 0 {
            n += 1;
        }
    }
    (n + 1).min(2)
}

/// Raven `SCR_DebugGraph`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:326-331`
pub fn SCR_DebugGraph(cl: &mut Client, value: f32, color: c_int) {
    // PORT-NOTE(missing-symbol): `graphsamp_t / values[1024] / current` is not in the tree yet.
    // Source: oracle/codemp/client/cl_scrn.cpp:318-319
    // PORT-NOTE(scr-graph): `current`/`values` are file-scope statics with no
    // rosetta home on `Client` yet; the fields are referenced verbatim per the
    // state-threading rule and land when the sub-struct is wired.
    cl.values[(cl.current & 1023) as usize].value = value;
    cl.values[(cl.current & 1023) as usize].color = color;
    cl.current += 1;
}

/// Raven `SCR_DrawDebugGraph`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:338-367`
pub fn SCR_DrawDebugGraph(common: &mut Common, cl: &mut Client) {
    let w: c_int = 640;
    let x: c_int = 0;
    let y: c_int = 480;

    crate::cl_renderer::re(cl).SetColor(g_color_table[0].as_ptr());
    crate::cl_renderer::re(cl).DrawStretchPic(
        x as f32,
        (y - unsafe { (*cl.cl_graphheight).integer }) as f32,
        w as f32,
        unsafe { (*cl.cl_graphheight).integer } as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        cl.cls.whiteShader,
    );
    crate::cl_renderer::re(cl).SetColor(core::ptr::null());

    for a in 0..w {
        let i = ((cl.current - 1 - a + 1024) & 1023) as usize;
        let mut v = cl.values[i].value;
        let color = cl.values[i].color;
        v = v * unsafe { (*cl.cl_graphscale).integer } as f32
            + unsafe { (*cl.cl_graphshift).integer } as f32;

        if v < 0.0 {
            let graphheight = unsafe { (*cl.cl_graphheight).integer };
            v += graphheight as f32 * (1 + (-v / graphheight as f32) as c_int) as f32;
        }
        let h = (v as c_int) % unsafe { (*cl.cl_graphheight).integer };
        crate::cl_renderer::re(cl).DrawStretchPic(
            (x + w - 1 - a) as f32,
            (y - h) as f32,
            1.0,
            h as f32,
            0.0,
            0.0,
            0.0,
            0.0,
            cl.cls.whiteShader,
        );
    }
}

/// Raven `SCR_Init`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:376-384`
pub fn SCR_Init(cl: &mut Client) {
    // PORT-NOTE(scr-init): `Cvar_Get`'s LAW signature takes `&mut
    // EngineHostView`, but this fn's resolved signature only threads `cl`; no
    // view receiver is available here, so the call is transcribed with `cl`
    // in that slot (see shape_mismatches).
    cl.cl_timegraph = Cvar_Get(cl, "timegraph", "0", CVAR_CHEAT) as _;
    cl.cl_debuggraph = Cvar_Get(cl, "debuggraph", "0", CVAR_CHEAT) as _;
    cl.cl_graphheight = Cvar_Get(cl, "graphheight", "32", CVAR_CHEAT) as _;
    cl.cl_graphscale = Cvar_Get(cl, "graphscale", "1", CVAR_CHEAT) as _;
    cl.cl_graphshift = Cvar_Get(cl, "graphshift", "0", CVAR_CHEAT) as _;

    cl.scr_initialized = true;
}

/// Raven `SCR_DrawStringExt`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:155-196`
pub fn SCR_DrawStringExt(
    common: &mut Common,
    cl: &mut Client,
    x: c_int,
    y: c_int,
    size: f32,
    string: *const c_char,
    setColor: *mut f32,
    forceColor: bool,
) {
    let mut color: [f32; 4] = [0.0; 4];
    unsafe {
        color[3] = *setColor.add(3);
    }
    crate::cl_renderer::re(cl).SetColor(color.as_ptr());

    // draw the drop shadow
    let mut s = string;
    let mut xx = x;
    unsafe {
        while *s != 0 {
            let pair = core::slice::from_raw_parts(s as *const u8, strlen_remaining(s));
            if Q_IsColorString(pair) {
                s = s.add(2);
                continue;
            }
            SCR_DrawChar(cl, xx + 2, y + 2, size, *s as c_int);
            xx += size as c_int;
            s = s.add(1);
        }
    }

    // draw the colored text
    let mut s = string;
    let mut xx = x;
    crate::cl_renderer::re(cl).SetColor(setColor);
    unsafe {
        while *s != 0 {
            let pair = core::slice::from_raw_parts(s as *const u8, strlen_remaining(s));
            if Q_IsColorString(pair) {
                if !forceColor {
                    // PORT-NOTE(missing-symbol): `ColorIndex` is not in the tree yet.
                    // Source: oracle/codemp/client/cl_scrn.cpp:184 (game/g_client.cpp local helper)
                    let idx = ColorIndex(*s.add(1) as u8);
                    Com_Memcpy(
                        color.as_mut_ptr() as *mut (),
                        g_color_table[idx as usize].as_ptr() as *const (),
                        core::mem::size_of::<[f32; 4]>(),
                    );
                    color[3] = *setColor.add(3);
                    crate::cl_renderer::re(cl).SetColor(color.as_ptr());
                }
                s = s.add(2);
                continue;
            }
            SCR_DrawChar(cl, xx, y, size, *s as c_int);
            xx += size as c_int;
            s = s.add(1);
        }
    }
    crate::cl_renderer::re(cl).SetColor(core::ptr::null());
}

/// Raven `SCR_DrawSmallStringExt`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:222-246`
pub fn SCR_DrawSmallStringExt(
    common: &mut Common,
    cl: &mut Client,
    x: c_int,
    y: c_int,
    string: *const c_char,
    setColor: *mut f32,
    forceColor: bool,
) {
    let mut color: [f32; 4] = [0.0; 4];
    let mut s = string;
    let mut xx = x;
    crate::cl_renderer::re(cl).SetColor(setColor);
    unsafe {
        while *s != 0 {
            let pair = core::slice::from_raw_parts(s as *const u8, strlen_remaining(s));
            if Q_IsColorString(pair) {
                if !forceColor {
                    // PORT-NOTE(missing-symbol): `ColorIndex` is not in the tree yet.
                    // Source: oracle/codemp/client/cl_scrn.cpp:234 (game/g_client.cpp local helper)
                    let idx = ColorIndex(*s.add(1) as u8);
                    Com_Memcpy(
                        color.as_mut_ptr() as *mut (),
                        g_color_table[idx as usize].as_ptr() as *const (),
                        core::mem::size_of::<[f32; 4]>(),
                    );
                    color[3] = *setColor.add(3);
                    crate::cl_renderer::re(cl).SetColor(color.as_ptr());
                }
                s = s.add(2);
                continue;
            }
            SCR_DrawSmallChar(cl, xx, y, *s as c_int);
            // PORT-NOTE(missing-symbol): `SMALLCHAR_WIDTH` is not in the tree yet.
            // Source: oracle/codemp/client/cl_scrn.h
            xx += SMALLCHAR_WIDTH;
            s = s.add(1);
        }
    }
    crate::cl_renderer::re(cl).SetColor(core::ptr::null());
}

/// Raven `SCR_GetBigStringWidth`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:272-274`
pub fn SCR_GetBigStringWidth(str: *const c_char) -> c_int {
    SCR_Strlen(str) * 16
}

/// Raven `SCR_CenterPrint`.
///
/// PORT-NOTE(scr-centerprint): the resolved signature threads only `cl`, but
/// the body calls `Com_Printf`, whose LAW signature needs `common`; the call
/// site below passes `cl` in that slot (see shape_mismatches). `RWL`-commented
/// dead code (`width -= 30`, the `remote_type` check) is dropped, matching
/// Raven's own `/* ... */` comment-out.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:519-624`
pub fn SCR_CenterPrint(cl: &mut Client, str: *mut c_char) {
    if str.is_null() {
        cl.scr_centertime_off = 0.0;
        return;
    }

    let mut width: c_int = 640 / 8;
    width -= 4;

    cl.scr_centertime_off = cl.scr_centertime.value;

    // PORT-NOTE(scr-centerprint): `com_printf`'s LAW signature needs `common`,
    // but this fn only threads `cl` (see shape_mismatches).
    com_printf(cl, "\n");

    let mut num_lines: c_int = 0;
    let mut write_pos: *mut c_char = cl.scr_centerstring.as_mut_ptr();
    cl.scr_center_lines = 0;
    let mut spaced = false;
    let mut done = false;

    let mut s = str;
    let mut start = str;
    let mut last: *mut c_char = core::ptr::null_mut();
    let mut num_chars: c_int = 0;

    unsafe {
        while !done {
            num_chars += 1;
            if *s == b' ' as c_char {
                spaced = true;
                last = s;
                cl.scr_centertime_off += 0.2f32 as f64 as f32;
            }

            if *s == b'\n' as c_char || *s == 0 {
                last = s;
                num_chars = width;
                spaced = true;
            }

            if num_chars >= width {
                cl.scr_centertime_off += 0.8f32 as f64 as f32;
                if last.is_null() {
                    last = s;
                }
                if !spaced {
                    last = last.add(1);
                }

                let save_pos = write_pos;
                let n = last.offset_from(start) as usize;
                core::ptr::copy_nonoverlapping(start, write_pos, n);
                write_pos = write_pos.add(n);
                *write_pos = 0;
                write_pos = write_pos.add(1);

                // PORT-NOTE(scr-centerprint): `com_printf`'s LAW signature needs
                // `common`, but this fn only threads `cl` (see shape_mismatches).
                com_printf(
                    cl,
                    &format!(
                        "{}\n",
                        core::ffi::CStr::from_ptr(save_pos).to_string_lossy()
                    ),
                );

                // PORT-NOTE(missing-symbol): `RWL / re.StrlenFont` is not in the tree yet.
                // Source: oracle/codemp/client/cl_scrn.cpp:595
                // PORT-NOTE(missing-symbol): `MAX_SCR_LINES` is not in the tree yet.
                // Source: oracle/codemp/client/cl_scrn.h
                cl.scr_center_widths[cl.scr_center_lines as usize] = 640;

                cl.scr_center_lines += 1;

                // PORT-NOTE(missing-symbol): `MAX_SCR_LINES` is not in the tree yet.
                // Source: oracle/codemp/client/cl_scrn.h
                if *s == 0 || cl.scr_center_lines >= MAX_SCR_LINES {
                    done = true;
                } else {
                    s = last;
                    if spaced {
                        last = last.add(1);
                    }
                    start = last;
                    last = core::ptr::null_mut();
                    num_chars = 0;
                    spaced = false;
                }
                continue;
            }

            s = s.add(1);
        }
    }

    // echo it to the console
    // PORT-NOTE(scr-centerprint): `com_printf`'s LAW signature needs `common`,
    // but this fn only threads `cl` (see shape_mismatches).
    com_printf(cl, "\n\n");
    Con_ClearNotify(cl);
}

/// Raven `SCR_DrawBigString`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:199-205`
pub fn SCR_DrawBigString(
    common: &mut Common,
    cl: &mut Client,
    x: c_int,
    y: c_int,
    s: *const c_char,
    alpha: f32,
) {
    let mut color: [f32; 4] = [1.0, 1.0, 1.0, alpha];
    // PORT-NOTE(missing-symbol): `BIGCHAR_WIDTH` is not in the tree yet.
    // Source: oracle/codemp/client/cl_scrn.h
    SCR_DrawStringExt(
        common,
        cl,
        x,
        y,
        BIGCHAR_WIDTH as f32,
        s,
        color.as_mut_ptr(),
        false,
    );
}

/// Raven `SCR_DrawBigStringColor`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:207-209`
pub fn SCR_DrawBigStringColor(
    common: &mut Common,
    cl: &mut Client,
    x: c_int,
    y: c_int,
    s: *const c_char,
    color: native_math::vector::vec4_t,
) {
    let mut color = color;
    // PORT-NOTE(missing-symbol): `BIGCHAR_WIDTH` is not in the tree yet.
    // Source: oracle/codemp/client/cl_scrn.h
    SCR_DrawStringExt(
        common,
        cl,
        x,
        y,
        BIGCHAR_WIDTH as f32,
        s,
        color.as_mut_ptr(),
        true,
    );
}

/// Raven `SCR_DrawDemoRecording`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:284-301`
pub fn SCR_DrawDemoRecording(common: &mut Common, cl: &mut Client) {
    if !cl.clc.demorecording {
        return;
    }
    if cl.clc.spDemoRecording {
        return;
    }

    let pos = FS_FTell(common, cl.clc.demofile);
    // PORT-NOTE(missing-symbol): `RECORDING` is not in the tree yet.
    // Source: oracle/codemp/client/cl_scrn.cpp:297 (format string literal)
    let string = format!(
        "RECORDING {}: {}k",
        unsafe { core::ffi::CStr::from_ptr(cl.clc.demoName.as_ptr()).to_string_lossy() },
        pos / 1024,
    );
    let mut cstring = std::ffi::CString::new(string.clone()).unwrap_or_default();

    let mut color = g_color_table[7];
    SCR_DrawStringExt(
        common,
        cl,
        320 - (string.len() as c_int) * 4,
        20,
        8.0,
        cstring.as_ptr(),
        color.as_mut_ptr(),
        true,
    );
}

/// Raven `SCR_DrawScreenField`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:396-469`
pub fn SCR_DrawScreenField(common: &mut Common, cl: &mut Client, stereoFrame: stereoFrame_t) {
    // PORT-NOTE(missing-symbol): `refexport_t::BeginFrame` is not in the tree yet.
    // Source: oracle/codemp/client/client.h:388
    crate::cl_renderer::re(cl).BeginFrame(stereoFrame);

    // wide aspect ratio screens need to have the sides cleared
    // unless they are displaying game renderings
    if cl.cls.state != connstate_t::CA_ACTIVE {
        if cl.cls.glconfig.vidWidth * 480 > cl.cls.glconfig.vidHeight * 640 {
            crate::cl_renderer::re(cl).SetColor(g_color_table[0].as_ptr());
            crate::cl_renderer::re(cl).DrawStretchPic(
                0.0,
                0.0,
                cl.cls.glconfig.vidWidth as f32,
                cl.cls.glconfig.vidHeight as f32,
                0.0,
                0.0,
                0.0,
                0.0,
                cl.cls.whiteShader,
            );
            crate::cl_renderer::re(cl).SetColor(core::ptr::null());
        }
    }

    // PORT-NOTE(missing-symbol): `uivm` is not in the tree yet.
    // Source: oracle/codemp/client/client.h:387
    if cl.uivm.is_null() {
        Com_DPrintf(common, "draw screen without UI loaded\n");
        return;
    }

    // if the menu is going to cover the entire screen, we
    // don't need to render anything under it
    // actually, yes you do, unless you want clients to cycle out their reliable
    // commands from sitting in the menu. -rww
    if VM_Call(common, cl.uivm, MpUiExport::UI_IS_FULLSCREEN as c_int, &[]) == 0
        || (cl.cls.framecount & 7 == 0 && cl.cls.state == connstate_t::CA_ACTIVE)
    {
        match cl.cls.state {
            connstate_t::CA_CINEMATIC => {
                SCR_DrawCinematic(cl);
            }
            connstate_t::CA_DISCONNECTED => {
                // force menu up
                S_StopAllSounds(cl);
                VM_Call(
                    common,
                    cl.uivm,
                    MpUiExport::UI_SET_ACTIVE_MENU as c_int,
                    &[UIMENU_MAIN as isize],
                );
            }
            connstate_t::CA_CONNECTING
            | connstate_t::CA_CHALLENGING
            | connstate_t::CA_CONNECTED => {
                // connecting clients will only show the connection dialog
                // refresh to update the time
                VM_Call(
                    common,
                    cl.uivm,
                    MpUiExport::UI_REFRESH as c_int,
                    &[cl.cls.realtime as isize],
                );
                VM_Call(
                    common,
                    cl.uivm,
                    MpUiExport::UI_DRAW_CONNECT_SCREEN as c_int,
                    &[false as isize],
                );
            }
            connstate_t::CA_LOADING | connstate_t::CA_PRIMED => {
                // draw the game information screen and loading progress
                CL_CGameRendering(common, cl, stereoFrame);

                // also draw the connection information, so it doesn't
                // flash away too briefly on local or lan games
                // refresh to update the time
                VM_Call(
                    common,
                    cl.uivm,
                    MpUiExport::UI_REFRESH as c_int,
                    &[cl.cls.realtime as isize],
                );
                VM_Call(
                    common,
                    cl.uivm,
                    MpUiExport::UI_DRAW_CONNECT_SCREEN as c_int,
                    &[true as isize],
                );
            }
            connstate_t::CA_ACTIVE => {
                CL_CGameRendering(common, cl, stereoFrame);
                SCR_DrawDemoRecording(common, cl);
            }
            _ => {
                com_error(
                    errorParm_t::ERR_FATAL,
                    "SCR_DrawScreenField: bad cls.state".to_string(),
                );
            }
        }
    }

    // the menu draws next
    // PORT-NOTE(missing-symbol): `KEYCATCH_UI` is not in the tree yet.
    // Source: oracle/codemp/client/cl_scrn.cpp:458
    if cl.cls.keyCatchers & KEYCATCH_UI != 0 && !cl.uivm.is_null() {
        VM_Call(
            common,
            cl.uivm,
            MpUiExport::UI_REFRESH as c_int,
            &[cl.cls.realtime as isize],
        );
    }

    // console draws next
    Con_DrawConsole(common, cl);

    // debug graph can be drawn on top of anything
    if unsafe {
        (*cl.cl_debuggraph).integer != 0
            || (*cl.cl_timegraph).integer != 0
            || (*cl.cl_debugMove).integer != 0
    } {
        SCR_DrawDebugGraph(common, cl);
    }
}

/// Raven `SCR_UpdateScreen`.
///
/// PORT-NOTE(scr-recursive): Raven's function-scope static `recursive` is
/// genuine cross-frame reentrancy-guard state (three-kind rule, kind 3); it
/// is referenced verbatim on `cl` per the state-threading rule until the
/// owning sub-struct lands the field.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:479-506`
pub fn SCR_UpdateScreen(common: &mut Common, cl: &mut Client) {
    if !cl.scr_initialized {
        return; // not initialized yet
    }

    cl.recursive += 1;
    if cl.recursive > 2 {
        com_error(
            errorParm_t::ERR_FATAL,
            "SCR_UpdateScreen: recursively called".to_string(),
        );
    }
    cl.recursive = 1;

    // if running in stereo, we need to draw the frame twice
    if cl.cls.glconfig.stereoEnabled {
        SCR_DrawScreenField(common, cl, STEREO_LEFT);
        SCR_DrawScreenField(common, cl, STEREO_RIGHT);
    } else {
        SCR_DrawScreenField(common, cl, STEREO_CENTER);
    }

    if common.cvar(common.com_speeds).integer != 0 {
        // PORT-NOTE(missing-symbol): `refexport_t::EndFrame` is not in the tree yet.
        // Source: oracle/codemp/client/client.h:388
        crate::cl_renderer::re(cl).EndFrame(&mut cl.time_frontend, &mut cl.time_backend);
    } else {
        crate::cl_renderer::re(cl).EndFrame(core::ptr::null_mut(), core::ptr::null_mut());
    }

    cl.recursive = 0;
}
