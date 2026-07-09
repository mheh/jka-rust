#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use sp_qshared::common::sp::ff::ff_handle_t::ffHandle_t;
use sp_qshared::common::sp::ghoul2::cghoul2_info_v::CGhoul2Info_v;
use sp_qshared::common::sp::renderer::glconfig_t::glconfig_t;
use sp_qshared::common::sp::renderer::ref_entity_t::refEntity_t;
use sp_qshared::common::sp::renderer::refdef_t::refdef_t;
use sp_qshared::shared::{
    mdxaBone_t, qboolean, qhandle_t, sfxHandle_t, vec3_t, vec4_t, Eorientations,
};

use super::cached_assets_t::cachedAssets_t;
use super::item_def_s::itemDef_s;

/// Raven `displayContextDef_t` — the UI module's function-pointer table into
/// the engine plus the display/frame state the engine keeps refreshed for it.
///
/// SP diverges from MP: SP's table is built against Raven's `_IMMERSION`
/// build (adds `registerForce`/`startForce`) and adds the Ghoul2 `g2_*`
/// utility hooks; it lacks MP's font/localization block (`RegisterFont`,
/// `Font_*`, `Language_*`, `AnyLanguage_ReadCharFromString`) and cinematic
/// playback hooks (`startBackgroundTrack`, `playCinematic`, `drawCinematic`,
/// `runCinematicFrame`), and adds `cursorShow`.
/// Type definition source: `oracle/code/ui/ui_shared.h:169-265`
#[repr(C)]
pub struct displayContextDef_t {
    pub addRefEntityToScene: Option<unsafe extern "C" fn(re: *const refEntity_t)>,
    pub clearScene: Option<unsafe extern "C" fn()>,
    pub drawHandlePic:
        Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, asset: qhandle_t)>,
    pub drawRect: Option<
        unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, size: f32, color: *const vec4_t),
    >,
    pub drawSides: Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, size: f32)>,
    pub drawText: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            scale: f32,
            color: *mut vec4_t,
            text: *const c_char,
            iMaxPixelWidth: c_int,
            style: c_int,
            iFontIndex: c_int,
        ),
    >,
    pub drawTextWithCursor: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            scale: f32,
            color: *mut vec4_t,
            text: *const c_char,
            cursorPos: c_int,
            cursor: c_char,
            iMaxPixelWidth: c_int,
            style: c_int,
            iFontIndex: c_int,
        ),
    >,
    pub drawTopBottom: Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, size: f32)>,
    pub executeText: Option<unsafe extern "C" fn(exec_when: c_int, text: *const c_char)>,
    pub feederCount: Option<unsafe extern "C" fn(feederID: f32) -> c_int>,
    pub feederSelection:
        Option<unsafe extern "C" fn(feederID: f32, index: c_int, item: *mut itemDef_s)>,
    pub fillRect:
        Option<unsafe extern "C" fn(x: f32, y: f32, w: f32, h: f32, color: *const vec4_t)>,
    pub getBindingBuf: Option<unsafe extern "C" fn(keynum: c_int, buf: *mut c_char, buflen: c_int)>,
    pub getCVarString:
        Option<unsafe extern "C" fn(cvar: *const c_char, buffer: *mut c_char, bufsize: c_int)>,
    pub getCVarValue: Option<unsafe extern "C" fn(cvar: *const c_char) -> f32>,
    pub getOverstrikeMode: Option<unsafe extern "C" fn() -> qboolean>,
    pub getValue: Option<unsafe extern "C" fn(ownerDraw: c_int) -> f32>,
    pub keynumToStringBuf:
        Option<unsafe extern "C" fn(keynum: c_int, buf: *mut c_char, buflen: c_int)>,
    pub modelBounds:
        Option<unsafe extern "C" fn(model: qhandle_t, min: *mut vec3_t, max: *mut vec3_t)>,
    pub ownerDrawHandleKey: Option<
        unsafe extern "C" fn(ownerDraw: c_int, flags: c_int, special: *mut f32, key: c_int) -> qboolean,
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
            iFontIndex: c_int,
        ),
    >,
    pub ownerDrawVisible: Option<unsafe extern "C" fn(flags: c_int) -> qboolean>,
    pub ownerDrawWidth: Option<unsafe extern "C" fn(ownerDraw: c_int, scale: f32) -> c_int>,
    pub Pause: Option<unsafe extern "C" fn(b: qboolean)>,
    pub Print: Option<unsafe extern "C" fn(msg: *const c_char, ...)>,
    pub registerFont: Option<unsafe extern "C" fn(pFontname: *const c_char) -> c_int>,
    pub registerModel: Option<unsafe extern "C" fn(p: *const c_char) -> qhandle_t>,
    pub registerShaderNoMip: Option<unsafe extern "C" fn(p: *const c_char) -> qhandle_t>,
    pub registerSound:
        Option<unsafe extern "C" fn(name: *const c_char, compressed: qboolean) -> sfxHandle_t>,
    pub renderScene: Option<unsafe extern "C" fn(fd: *const refdef_t)>,
    pub runScript: Option<unsafe extern "C" fn(p: *mut *const c_char) -> qboolean>,
    pub deferScript: Option<unsafe extern "C" fn(p: *mut *const c_char) -> qboolean>,
    pub setBinding: Option<unsafe extern "C" fn(keynum: c_int, binding: *const c_char)>,
    pub setColor: Option<unsafe extern "C" fn(v: *const vec4_t)>,
    pub setCVar: Option<unsafe extern "C" fn(cvar: *const c_char, value: *const c_char)>,
    pub setOverstrikeMode: Option<unsafe extern "C" fn(b: qboolean)>,
    pub startLocalSound: Option<unsafe extern "C" fn(sfx: sfxHandle_t, channelNum: c_int)>,
    pub stopCinematic: Option<unsafe extern "C" fn(handle: c_int)>,
    pub textHeight:
        Option<unsafe extern "C" fn(text: *const c_char, scale: f32, iFontIndex: c_int) -> c_int>,
    pub textWidth:
        Option<unsafe extern "C" fn(text: *const c_char, scale: f32, iFontIndex: c_int) -> c_int>,
    pub feederItemImage: Option<unsafe extern "C" fn(feederID: f32, index: c_int) -> qhandle_t>,
    pub feederItemText: Option<
        unsafe extern "C" fn(
            feederID: f32,
            index: c_int,
            column: c_int,
            handle: *mut qhandle_t,
        ) -> *const c_char,
    >,

    pub registerSkin: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,

    // Raven: rww - ghoul2 stuff. Add whatever you need here, remember to set it in
    // _UI_Init or it will crash when you try to use it.
    //TODO: Port CGhoul2Info
    // Source: oracle/code/game/ghoul2_shared.h:240
    pub g2_SetSkin: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            customSkin: qhandle_t,
            renderSkin: qhandle_t,
        ) -> qboolean,
    >,
    //TODO: Port CGhoul2Info
    // Source: oracle/code/game/ghoul2_shared.h:240
    pub g2_SetBoneAnim: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneName: *const c_char,
            startFrame: c_int,
            endFrame: c_int,
            flags: c_int,
            animSpeed: f32,
            currentTime: c_int,
            setFrame: f32,
            blendTime: c_int,
        ) -> qboolean,
    >,
    // Raven passes `CGhoul2Info_v &` — reference ABI is a pointer.
    pub g2_RemoveGhoul2Model:
        Option<unsafe extern "C" fn(ghlInfo: *mut CGhoul2Info_v, modelIndex: c_int) -> qboolean>,
    pub g2_InitGhoul2Model: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            fileName: *const c_char,
            unused: c_int,
            customSkin: qhandle_t,
            customShader: qhandle_t,
            modelFlags: c_int,
            lodBias: c_int,
        ) -> c_int,
    >,
    pub g2_CleanGhoul2Models: Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v)>,
    //TODO: Port CGhoul2Info
    // Source: oracle/code/game/ghoul2_shared.h:240
    pub g2_AddBolt:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char) -> c_int>,
    pub g2_GetBoltMatrix: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            modelIndex: c_int,
            boltIndex: c_int,
            matrix: *mut mdxaBone_t,
            angles: *const vec3_t,
            position: *const vec3_t,
            frameNum: c_int,
            modelList: *mut qhandle_t,
            scale: *const vec3_t,
        ) -> qboolean,
    >,
    pub g2_GiveMeVectorFromMatrix: Option<
        unsafe extern "C" fn(boltMatrix: *mut mdxaBone_t, flags: Eorientations, vec: *mut vec3_t),
    >,

    // Raven: Utility functions that don't immediately redirect to ghoul2 functions
    //TODO: Port CGhoul2Info
    // Source: oracle/code/game/ghoul2_shared.h:240
    pub g2hilev_SetAnim: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneName: *const c_char,
            animNum: c_int,
            freeze: qboolean,
        ) -> c_int,
    >,

    // Raven: `#ifdef _IMMERSION` — force-feedback registration; layout reflects
    // the `_IMMERSION`-enabled build the offsets were captured against.
    pub registerForce:
        Option<unsafe extern "C" fn(name: *const c_char, channel: c_int) -> ffHandle_t>,
    pub startForce: Option<unsafe extern "C" fn(ff: ffHandle_t)>,

    pub yscale: f32,
    pub xscale: f32,
    pub bias: f32,
    pub realTime: c_int,
    pub frameTime: c_int,
    pub cursorShow: qboolean,
    pub cursorx: c_int,
    pub cursory: c_int,
    pub debug: qboolean,

    pub Assets: cachedAssets_t,

    pub glconfig: glconfig_t,
    pub whiteShader: qhandle_t,
    pub gradientImage: qhandle_t,
    pub FPS: f32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<displayContextDef_t>() == 792);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, addRefEntityToScene) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, clearScene) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawHandlePic) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawRect) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawSides) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawText) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawTextWithCursor) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, drawTopBottom) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, executeText) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederCount) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederSelection) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, fillRect) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getBindingBuf) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getCVarString) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getCVarValue) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getOverstrikeMode) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, getValue) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, keynumToStringBuf) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, modelBounds) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawHandleKey) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawItem) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawVisible) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, ownerDrawWidth) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Pause) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Print) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerFont) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerModel) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerShaderNoMip) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerSound) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, renderScene) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, runScript) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, deferScript) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setBinding) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setColor) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setCVar) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, setOverstrikeMode) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, startLocalSound) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, stopCinematic) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, textHeight) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, textWidth) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederItemImage) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, feederItemText) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerSkin) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_SetSkin) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_SetBoneAnim) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_RemoveGhoul2Model) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_InitGhoul2Model) == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_CleanGhoul2Models) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_AddBolt) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_GetBoltMatrix) == 392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2_GiveMeVectorFromMatrix) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, g2hilev_SetAnim) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, registerForce) == 416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, startForce) == 424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, yscale) == 432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, xscale) == 436);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, bias) == 440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, realTime) == 444);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, frameTime) == 448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, cursorShow) == 452);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, cursorx) == 456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, cursory) == 460);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, debug) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, Assets) == 468);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, glconfig) == 680);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, whiteShader) == 776);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, gradientImage) == 780);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(displayContextDef_t, FPS) == 784);
