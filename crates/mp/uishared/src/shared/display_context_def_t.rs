#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_uint};

use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::{qboolean, qhandle_t, sfxHandle_t, vec3_t, vec4_t};

use super::cached_assets_t::cachedAssets_t;
use super::item_def_s::itemDef_t;

/// Raven `displayContextDef_t` — the UI module's function-pointer table into
/// the engine plus the display/frame state the engine keeps refreshed for it.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:400-477`
#[repr(C)]
pub struct displayContextDef_t {
    pub registerShaderNoMip: Option<unsafe extern "C" fn(p: *const c_char) -> qhandle_t>,
    pub setColor: Option<unsafe extern "C" fn(v: *const vec4_t)>,
    pub drawHandlePic:
        Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, asset: qhandle_t)>,
    pub drawStretchPic: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            s1: f32,
            t1: f32,
            s2: f32,
            t2: f32,
            hShader: qhandle_t,
        ),
    >,
    pub drawText: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            scale: f32,
            color: *mut vec4_t,
            text: *const c_char,
            adjust: f32,
            limit: c_int,
            style: c_int,
            iMenuFont: c_int,
        ),
    >,
    pub textWidth:
        Option<unsafe extern "C" fn(text: *const c_char, scale: f32, iMenuFont: c_int) -> c_int>,
    pub textHeight:
        Option<unsafe extern "C" fn(text: *const c_char, scale: f32, iMenuFont: c_int) -> c_int>,
    pub registerModel: Option<unsafe extern "C" fn(p: *const c_char) -> qhandle_t>,
    pub modelBounds: Option<
        unsafe extern "C" fn(model: qhandle_t, min: *mut vec3_t, max: *mut vec3_t),
    >,
    pub fillRect:
        Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, color: *const vec4_t)>,
    pub drawRect: Option<
        unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, size: f32, color: *const vec4_t),
    >,
    pub drawSides: Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, size: f32)>,
    pub drawTopBottom: Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, size: f32)>,
    pub clearScene: Option<unsafe extern "C" fn()>,
    pub addRefEntityToScene: Option<unsafe extern "C" fn(re: *const refEntity_t)>,
    pub renderScene: Option<unsafe extern "C" fn(fd: *const refdef_t)>,

    pub RegisterFont: Option<unsafe extern "C" fn(fontName: *const c_char) -> qhandle_t>,
    pub Font_StrLenPixels: Option<
        unsafe extern "C" fn(text: *const c_char, iFontIndex: c_int, scale: f32) -> c_int,
    >,
    pub Font_StrLenChars: Option<unsafe extern "C" fn(text: *const c_char) -> c_int>,
    pub Font_HeightPixels: Option<unsafe extern "C" fn(iFontIndex: c_int, scale: f32) -> c_int>,
    pub Font_DrawString: Option<
        unsafe extern "C" fn(
            ox: c_int,
            oy: c_int,
            text: *const c_char,
            rgba: *const f32,
            setIndex: c_int,
            iCharLimit: c_int,
            scale: f32,
        ),
    >,
    pub Language_IsAsian: Option<unsafe extern "C" fn() -> qboolean>,
    pub Language_UsesSpaces: Option<unsafe extern "C" fn() -> qboolean>,
    pub AnyLanguage_ReadCharFromString: Option<
        unsafe extern "C" fn(
            psText: *const c_char,
            piAdvanceCount: *mut c_int,
            pbIsTrailingPunctuation: *mut qboolean,
        ) -> c_uint,
    >,
    pub ownerDrawItem: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            text_x: f32,
            text_y: f32,
            ownerDraw: c_int,
            ownerDrawFlags: c_int,
            align: c_int,
            special: f32,
            scale: f32,
            color: *mut vec4_t,
            shader: qhandle_t,
            textStyle: c_int,
            iMenuFont: c_int,
        ),
    >,
    pub getValue: Option<unsafe extern "C" fn(ownerDraw: c_int) -> f32>,
    pub ownerDrawVisible: Option<unsafe extern "C" fn(flags: c_int) -> qboolean>,
    pub runScript: Option<unsafe extern "C" fn(p: *mut *mut c_char)>,
    pub deferScript: Option<unsafe extern "C" fn(p: *mut *mut c_char) -> qboolean>,
    pub getTeamColor: Option<unsafe extern "C" fn(color: *mut vec4_t)>,
    pub getCVarString:
        Option<unsafe extern "C" fn(cvar: *const c_char, buffer: *mut c_char, bufsize: c_int)>,
    pub getCVarValue: Option<unsafe extern "C" fn(cvar: *const c_char) -> f32>,
    pub setCVar: Option<unsafe extern "C" fn(cvar: *const c_char, value: *const c_char)>,
    pub drawTextWithCursor: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            scale: f32,
            color: *mut vec4_t,
            text: *const c_char,
            cursorPos: c_int,
            cursor: c_char,
            limit: c_int,
            style: c_int,
            iFontIndex: c_int,
        ),
    >,
    pub setOverstrikeMode: Option<unsafe extern "C" fn(b: qboolean)>,
    pub getOverstrikeMode: Option<unsafe extern "C" fn() -> qboolean>,
    pub startLocalSound: Option<unsafe extern "C" fn(sfx: sfxHandle_t, channelNum: c_int)>,
    pub ownerDrawHandleKey: Option<
        unsafe extern "C" fn(ownerDraw: c_int, flags: c_int, special: *mut f32, key: c_int) -> qboolean,
    >,
    pub feederCount: Option<unsafe extern "C" fn(feederID: f32) -> c_int>,
    pub feederItemText: Option<
        unsafe extern "C" fn(
            feederID: f32,
            index: c_int,
            column: c_int,
            handle1: *mut qhandle_t,
            handle2: *mut qhandle_t,
            handle3: *mut qhandle_t,
        ) -> *const c_char,
    >,
    pub feederItemImage: Option<unsafe extern "C" fn(feederID: f32, index: c_int) -> qhandle_t>,
    pub feederSelection:
        Option<unsafe extern "C" fn(feederID: f32, index: c_int, item: *mut itemDef_t) -> qboolean>,
    pub keynumToStringBuf:
        Option<unsafe extern "C" fn(keynum: c_int, buf: *mut c_char, buflen: c_int)>,
    pub getBindingBuf: Option<unsafe extern "C" fn(keynum: c_int, buf: *mut c_char, buflen: c_int)>,
    pub setBinding: Option<unsafe extern "C" fn(keynum: c_int, binding: *const c_char)>,
    pub executeText: Option<unsafe extern "C" fn(exec_when: c_int, text: *const c_char)>,
    pub Error: Option<unsafe extern "C" fn(level: c_int, error: *const c_char, ...)>,
    pub Print: Option<unsafe extern "C" fn(msg: *const c_char, ...)>,
    pub Pause: Option<unsafe extern "C" fn(b: qboolean)>,
    pub ownerDrawWidth: Option<unsafe extern "C" fn(ownerDraw: c_int, scale: f32) -> c_int>,
    pub registerSound: Option<unsafe extern "C" fn(name: *const c_char) -> sfxHandle_t>,
    pub startBackgroundTrack: Option<
        unsafe extern "C" fn(
            intro: *const c_char,
            loop_: *const c_char,
            bReturnWithoutStarting: qboolean,
        ),
    >,
    pub stopBackgroundTrack: Option<unsafe extern "C" fn()>,
    pub playCinematic:
        Option<unsafe extern "C" fn(name: *const c_char, x: f32, y: f32, w: f32, h: f32) -> c_int>,
    pub stopCinematic: Option<unsafe extern "C" fn(handle: c_int)>,
    pub drawCinematic:
        Option<unsafe extern "C" fn(handle: c_int, x: f32, y: f32, w: f32, h: f32)>,
    pub runCinematicFrame: Option<unsafe extern "C" fn(handle: c_int)>,

    pub yscale: f32,
    pub xscale: f32,
    pub bias: f32,
    pub realTime: c_int,
    pub frameTime: c_int,
    pub cursorx: c_int,
    pub cursory: c_int,
    pub debug: qboolean,

    pub Assets: cachedAssets_t,

    pub glconfig: glconfig_t,
    pub whiteShader: qhandle_t,
    pub gradientImage: qhandle_t,
    pub cursor: qhandle_t,
    pub FPS: f32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<displayContextDef_t>() == 872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerShaderNoMip) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setColor) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawHandlePic) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawStretchPic) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawText) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, textWidth) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, textHeight) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerModel) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, modelBounds) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, fillRect) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawRect) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawSides) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawTopBottom) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, clearScene) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, addRefEntityToScene) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, renderScene) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, RegisterFont) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Font_StrLenPixels) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Font_StrLenChars) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Font_HeightPixels) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Font_DrawString) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Language_IsAsian) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Language_UsesSpaces) == 176);
#[cfg(target_pointer_width = "64")]
const _: () =
    assert!(core::mem::offset_of!(displayContextDef_t, AnyLanguage_ReadCharFromString) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawItem) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getValue) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawVisible) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, runScript) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, deferScript) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getTeamColor) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getCVarString) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getCVarValue) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setCVar) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawTextWithCursor) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setOverstrikeMode) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getOverstrikeMode) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, startLocalSound) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawHandleKey) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederCount) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederItemText) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederItemImage) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederSelection) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, keynumToStringBuf) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getBindingBuf) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setBinding) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, executeText) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Error) == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Print) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Pause) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawWidth) == 392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerSound) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, startBackgroundTrack) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, stopBackgroundTrack) == 416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, playCinematic) == 424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, stopCinematic) == 432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawCinematic) == 440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, runCinematicFrame) == 448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, yscale) == 456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, xscale) == 460);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, bias) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, realTime) == 468);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, frameTime) == 472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, cursorx) == 476);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, cursory) == 480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, debug) == 484);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Assets) == 488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, glconfig) == 760);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, whiteShader) == 856);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, gradientImage) == 860);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, cursor) == 864);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, FPS) == 868);
