#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

use sp_qshared::common::sp::renderer::glconfig_t::glconfig_t;
use sp_qshared::common::sp::renderer::poly_vert_t::polyVert_t;
use sp_qshared::common::sp::renderer::ref_entity_t::refEntity_t;
use sp_qshared::common::sp::renderer::refdef_t::refdef_t;
use sp_qshared::shared::{
    clipHandle_t, connstate_t, fileHandle_t, fsMode_t, orientation_t, qboolean, qhandle_t,
    sfxHandle_t,
};

/// Raven `uiimport_t` — engine import table handed to the statically-linked SP UI module.
///
/// Raven: general Quake services, renderer, sound, filesystem, and input calls the UI code
/// needs from the engine. Unlike MP (which routes UI through a VM syscall table), SP links
/// its UI code directly into the engine binary, so this is a plain function-pointer struct
/// rather than a syscall enum.
/// Type definition source: `oracle/oracle/code/ui/ui_public.h:11-141`
#[repr(C)]
pub struct uiimport_t {
    //============== general Quake services ==================

    /// print message on the local console
    //TODO: Port Printf variadic args
    // Source: oracle/oracle/code/ui/ui_public.h:15
    pub Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,

    /// abort the game
    //TODO: Port Error variadic args
    // Source: oracle/oracle/code/ui/ui_public.h:18
    pub Error: Option<unsafe extern "C" fn(level: c_int, fmt: *const c_char, ...)>,

    // console variable interaction
    pub Cvar_Set: Option<unsafe extern "C" fn(name: *const c_char, value: *const c_char)>,
    pub Cvar_VariableValue: Option<unsafe extern "C" fn(var_name: *const c_char) -> f32>,
    pub Cvar_VariableStringBuffer: Option<
        unsafe extern "C" fn(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int),
    >,
    pub Cvar_SetValue: Option<unsafe extern "C" fn(var_name: *const c_char, value: f32)>,
    pub Cvar_Reset: Option<unsafe extern "C" fn(name: *const c_char)>,
    pub Cvar_Create: Option<
        unsafe extern "C" fn(var_name: *const c_char, var_value: *const c_char, flags: c_int),
    >,
    pub Cvar_InfoStringBuffer:
        Option<unsafe extern "C" fn(bit: c_int, buffer: *mut c_char, bufsize: c_int)>,

    // console command interaction
    pub Argc: Option<unsafe extern "C" fn() -> c_int>,
    pub Argv: Option<unsafe extern "C" fn(n: c_int, buffer: *mut c_char, bufferLength: c_int)>,
    pub Cmd_ExecuteText: Option<unsafe extern "C" fn(exec_when: c_int, text: *const c_char)>,
    pub Cmd_TokenizeString: Option<unsafe extern "C" fn(text: *const c_char)>,

    // filesystem access
    pub FS_FOpenFile: Option<
        unsafe extern "C" fn(qpath: *const c_char, file: *mut fileHandle_t, mode: fsMode_t) -> c_int,
    >,
    pub FS_Read:
        Option<unsafe extern "C" fn(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int>,
    pub FS_Write:
        Option<unsafe extern "C" fn(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int>,
    pub FS_FCloseFile: Option<unsafe extern "C" fn(f: fileHandle_t)>,
    pub FS_GetFileList: Option<
        unsafe extern "C" fn(
            path: *const c_char,
            extension: *const c_char,
            listbuf: *mut c_char,
            bufsize: c_int,
        ) -> c_int,
    >,
    pub FS_ReadFile:
        Option<unsafe extern "C" fn(name: *const c_char, buf: *mut *mut c_void) -> c_int>,
    pub FS_FreeFile: Option<unsafe extern "C" fn(buf: *mut c_void)>,

    // =========== renderer function calls ================

    /// returns rgb axis if not found
    pub R_RegisterModel: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    /// returns all white if not found
    pub R_RegisterSkin: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    /// returns white if not found
    pub R_RegisterShader: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    /// returns white if not found
    pub R_RegisterShaderNoMip: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    /// returns 0 for bad font
    pub R_RegisterFont: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,

    // Raven's `#ifdef _XBOX` branch inlines these three as default-argument member
    // functions; dead on the shipping PC engine. Only the `#else` function-pointer
    // branch (with the C++ default `scale = 1.0f` dropped, since Rust fn pointers
    // have no default args) is faithful here.
    pub R_Font_StrLenPixels:
        Option<unsafe extern "C" fn(text: *const c_char, setIndex: c_int, scale: f32) -> c_int>,
    pub R_Font_HeightPixels: Option<unsafe extern "C" fn(setIndex: c_int, scale: f32) -> c_int>,
    pub R_Font_DrawString: Option<
        unsafe extern "C" fn(
            ox: c_int,
            oy: c_int,
            text: *const c_char,
            rgba: *const f32,
            setIndex: c_int,
            iMaxPixelWidth: c_int,
            scale: f32,
        ),
    >,
    pub R_Font_StrLenChars: Option<unsafe extern "C" fn(text: *const c_char) -> c_int>,
    pub Language_IsAsian: Option<unsafe extern "C" fn() -> qboolean>,
    pub Language_UsesSpaces: Option<unsafe extern "C" fn() -> qboolean>,
    pub AnyLanguage_ReadCharFromString: Option<
        unsafe extern "C" fn(
            psText: *const c_char,
            piAdvanceCount: *mut c_int,
            pbIsTrailingPunctuation: *mut qboolean,
        ) -> c_uint,
    >,

    // a scene is built up by calls to R_ClearScene and the various R_Add functions.
    // Nothing is drawn until R_RenderScene is called.
    pub R_ClearScene: Option<unsafe extern "C" fn()>,
    pub R_AddRefEntityToScene: Option<unsafe extern "C" fn(re: *const refEntity_t)>,
    pub R_AddPolyToScene:
        Option<unsafe extern "C" fn(hShader: qhandle_t, numVerts: c_int, verts: *const polyVert_t)>,
    pub R_AddLightToScene:
        Option<unsafe extern "C" fn(org: *const f32, intensity: f32, r: f32, g: f32, b: f32)>,
    pub R_RenderScene: Option<unsafe extern "C" fn(fd: *const refdef_t)>,

    pub R_ModelBounds: Option<unsafe extern "C" fn(handle: qhandle_t, mins: *mut f32, maxs: *mut f32)>,

    /// NULL = 1,1,1,1
    pub R_SetColor: Option<unsafe extern "C" fn(rgba: *const f32)>,
    /// 0 = white
    pub R_DrawStretchPic: Option<
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
    /// 0 = white
    pub R_ScissorPic: Option<
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

    /// force a screen update, only used during gamestate load
    pub UpdateScreen: Option<unsafe extern "C" fn()>,

    // Raven's `#ifdef _XBOX` PrecacheScreenshot member is dead on the shipping PC engine
    // and is omitted here (its offsets are absent from the provided layout too).

    //========= model collision ===============

    /// R_LerpTag is only valid for md3 models
    pub R_LerpTag: Option<
        unsafe extern "C" fn(
            tag: *mut orientation_t,
            mod_: clipHandle_t,
            startFrame: c_int,
            endFrame: c_int,
            frac: f32,
            tagName: *const c_char,
        ),
    >,

    // =========== sound function calls ===============
    pub S_StartLocalSound: Option<unsafe extern "C" fn(sfxHandle: sfxHandle_t, channelNum: c_int)>,
    pub S_RegisterSound: Option<unsafe extern "C" fn(name: *const c_char) -> sfxHandle_t>,
    pub S_StartLocalLoopingSound: Option<unsafe extern "C" fn(sfxHandle: sfxHandle_t)>,
    pub S_StopSounds: Option<unsafe extern "C" fn()>,

    // =========== getting save game picture ===============
    pub DrawStretchRaw: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            w: c_int,
            h: c_int,
            cols: c_int,
            rows: c_int,
            data: *const c_uchar,
            client: c_int,
            dirty: qboolean,
        ),
    >,
    pub SG_GetSaveGameComment: Option<
        unsafe extern "C" fn(
            psPathlessBaseName: *const c_char,
            sComment: *mut c_char,
            sMapName: *mut c_char,
        ) -> c_int,
    >,
    pub SG_GameAllowedToSaveHere: Option<unsafe extern "C" fn(inCamera: qboolean) -> qboolean>,
    pub SG_StoreSaveGameComment: Option<unsafe extern "C" fn(sComment: *const c_char)>,

    // =========== data shared with the client system =============

    // keyboard and key binding interaction
    pub Key_KeynumToStringBuf:
        Option<unsafe extern "C" fn(keynum: c_int, buf: *mut c_char, buflen: c_int)>,
    pub Key_GetBindingBuf:
        Option<unsafe extern "C" fn(keynum: c_int, buf: *mut c_char, buflen: c_int)>,
    pub Key_SetBinding: Option<unsafe extern "C" fn(keynum: c_int, binding: *const c_char)>,
    pub Key_IsDown: Option<unsafe extern "C" fn(keynum: c_int) -> qboolean>,
    pub Key_GetOverstrikeMode: Option<unsafe extern "C" fn() -> qboolean>,
    pub Key_SetOverstrikeMode: Option<unsafe extern "C" fn(state: qboolean)>,
    pub Key_ClearStates: Option<unsafe extern "C" fn()>,
    pub Key_GetCatcher: Option<unsafe extern "C" fn() -> c_int>,
    pub Key_SetCatcher: Option<unsafe extern "C" fn(catcher: c_int)>,

    pub GetClipboardData: Option<unsafe extern "C" fn(buf: *mut c_char, bufsize: c_int)>,

    pub GetGlconfig: Option<unsafe extern "C" fn(config: *mut glconfig_t)>,

    pub GetClientState: Option<unsafe extern "C" fn() -> connstate_t>,

    pub GetConfigString: Option<unsafe extern "C" fn(index: c_int, buff: *mut c_char, buffsize: c_int)>,

    pub Milliseconds: Option<unsafe extern "C" fn() -> c_int>,
    pub Draw_DataPad: Option<unsafe extern "C" fn(HUDType: c_int)>,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<uiimport_t>() == 528);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Printf) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Error) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cvar_Set) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cvar_VariableValue) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cvar_VariableStringBuffer) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cvar_SetValue) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cvar_Reset) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cvar_Create) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cvar_InfoStringBuffer) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Argc) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Argv) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cmd_ExecuteText) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Cmd_TokenizeString) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, FS_FOpenFile) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, FS_Read) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, FS_Write) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, FS_FCloseFile) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, FS_GetFileList) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, FS_ReadFile) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, FS_FreeFile) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_RegisterModel) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_RegisterSkin) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_RegisterShader) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_RegisterShaderNoMip) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_RegisterFont) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_Font_StrLenPixels) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_Font_HeightPixels) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_Font_DrawString) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_Font_StrLenChars) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Language_IsAsian) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Language_UsesSpaces) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, AnyLanguage_ReadCharFromString) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_ClearScene) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_AddRefEntityToScene) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_AddPolyToScene) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_AddLightToScene) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_RenderScene) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_ModelBounds) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_SetColor) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_DrawStretchPic) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_ScissorPic) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, UpdateScreen) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, R_LerpTag) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, S_StartLocalSound) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, S_RegisterSound) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, S_StartLocalLoopingSound) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, S_StopSounds) == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, DrawStretchRaw) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, SG_GetSaveGameComment) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, SG_GameAllowedToSaveHere) == 392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, SG_StoreSaveGameComment) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_KeynumToStringBuf) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_GetBindingBuf) == 416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_SetBinding) == 424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_IsDown) == 432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_GetOverstrikeMode) == 440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_SetOverstrikeMode) == 448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_ClearStates) == 456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_GetCatcher) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Key_SetCatcher) == 472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, GetClipboardData) == 480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, GetGlconfig) == 488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, GetClientState) == 496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, GetConfigString) == 504);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Milliseconds) == 512);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(uiimport_t, Draw_DataPad) == 520);
