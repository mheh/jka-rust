#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int, c_uint};

use sp_qshared::common::sp::renderer::glconfig_t::glconfig_t;
use sp_qshared::common::sp::renderer::poly_vert_t::polyVert_t;
use sp_qshared::common::sp::renderer::ref_entity_t::refEntity_t;
use sp_qshared::common::sp::renderer::refdef_t::refdef_t;
use sp_qshared::common::sp::renderer::stereo_frame_t::stereoFrame_t;
use sp_qshared::shared::{markFragment_t, orientation_t, qboolean, qhandle_t, vec3_t, ForceReload_e};

/// Raven `refexport_t` — renderer export function table (`re` in the client),
/// handed from the loaded renderer module to the engine.
///
/// Raven: called before the library is unloaded; if the system is just
/// reconfiguring, pass destroyWindow = qfalse, which will keep the screen from
/// flashing to the desktop. All data that will be used in a level should be
/// registered before rendering any frames to prevent disk hits, but they can
/// still be registered at a later time if necessary. BeginRegistration makes
/// any existing media pointers invalid and returns the current gl
/// configuration, including screen width and height, which can be used by the
/// client to intelligently size display elements. `RegisterMedia_LevelLoadBegin`
/// / `RegisterMedia_LevelLoadEnd` were added to help with the new model alloc
/// scheme. The vis data is a large enough block of data that we go to the
/// trouble of sharing it with the clipmodel subsystem. EndRegistration will
/// draw a tiny polygon with each texture, forcing them to be loaded into card
/// memory. A scene is built up by calls to R_ClearScene and the various R_Add
/// functions. Nothing is drawn until R_RenderScene is called. If the pointers
/// are not NULL, timing info will be returned. `GetScreenShot` is for use with
/// save-games mainly; `TempRawImage_ReadFromFile` gives access to raw pixels
/// from a graphics format (TGA/JPG/BMP etc), currently only the save game uses
/// it (to make raw shots for the autosaves).
/// Type definition source: `oracle/oracle/code/renderer/tr_public.h:19-139`
#[repr(C)]
pub struct refexport_t {
    /// called before the library is unloaded
    /// if the system is just reconfiguring, pass destroyWindow = qfalse,
    /// which will keep the screen from flashing to the desktop.
    pub Shutdown: Option<unsafe extern "C" fn(destroyWindow: qboolean)>,

    /// All data that will be used in a level should be
    /// registered before rendering any frames to prevent disk hits,
    /// but they can still be registered at a later time
    /// if necessary.
    ///
    /// BeginRegistration makes any existing media pointers invalid
    /// and returns the current gl configuration, including screen width
    /// and height, which can be used by the client to intelligently
    /// size display elements
    pub BeginRegistration: Option<unsafe extern "C" fn(config: *mut glconfig_t)>,
    pub RegisterModel: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub RegisterSkin: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub GetAnimationCFG:
        Option<unsafe extern "C" fn(psCFGFilename: *const c_char, psDest: *mut c_char, iDestSize: c_int) -> c_int>,
    pub RegisterShader: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub RegisterShaderNoMip: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub LoadWorld: Option<unsafe extern "C" fn(name: *const c_char)>,

    // these two functions added to help with the new model alloc scheme...
    pub RegisterMedia_LevelLoadBegin: Option<
        unsafe extern "C" fn(psMapName: *const c_char, eForceReload: ForceReload_e, bAllowScreenDissolve: qboolean),
    >,
    pub RegisterMedia_LevelLoadEnd: Option<unsafe extern "C" fn()>,

    // the vis data is a large enough block of data that we go to the trouble
    // of sharing it with the clipmodel subsystem
    pub SetWorldVisData: Option<unsafe extern "C" fn(vis: *const u8)>,

    /// EndRegistration will draw a tiny polygon with each texture, forcing
    /// them to be loaded into card memory
    pub EndRegistration: Option<unsafe extern "C" fn()>,

    // a scene is built up by calls to R_ClearScene and the various R_Add functions.
    // Nothing is drawn until R_RenderScene is called.
    pub ClearScene: Option<unsafe extern "C" fn()>,
    pub AddRefEntityToScene: Option<unsafe extern "C" fn(re: *const refEntity_t)>,
    pub AddPolyToScene:
        Option<unsafe extern "C" fn(hShader: qhandle_t, numVerts: c_int, verts: *const polyVert_t)>,
    pub AddLightToScene: Option<
        unsafe extern "C" fn(org: *const vec3_t, intensity: c_float, r: c_float, g: c_float, b: c_float),
    >,
    pub RenderScene: Option<unsafe extern "C" fn(fd: *const refdef_t)>,
    pub GetLighting: Option<
        unsafe extern "C" fn(
            org: *const vec3_t,
            ambientLight: *mut vec3_t,
            directedLight: *mut vec3_t,
            lightDir: *mut vec3_t,
        ) -> qboolean,
    >,

    /// NULL = 1,1,1,1
    pub SetColor: Option<unsafe extern "C" fn(rgba: *const c_float)>,
    /// 0 = white
    pub DrawStretchPic: Option<
        unsafe extern "C" fn(
            x: c_float,
            y: c_float,
            w: c_float,
            h: c_float,
            s1: c_float,
            t1: c_float,
            s2: c_float,
            t2: c_float,
            hShader: qhandle_t,
        ),
    >,
    /// 0 = white
    pub DrawRotatePic: Option<
        unsafe extern "C" fn(
            x: c_float,
            y: c_float,
            w: c_float,
            h: c_float,
            s1: c_float,
            t1: c_float,
            s2: c_float,
            t2: c_float,
            a1: c_float,
            hShader: qhandle_t,
        ),
    >,
    /// 0 = white
    pub DrawRotatePic2: Option<
        unsafe extern "C" fn(
            x: c_float,
            y: c_float,
            w: c_float,
            h: c_float,
            s1: c_float,
            t1: c_float,
            s2: c_float,
            t2: c_float,
            a1: c_float,
            hShader: qhandle_t,
        ),
    >,
    pub LAGoggles: Option<unsafe extern "C" fn()>,
    /// 0 = white
    pub Scissor: Option<unsafe extern "C" fn(x: c_float, y: c_float, w: c_float, h: c_float)>,

    // Draw images for cinematic rendering, pass as 32 bit rgba
    pub DrawStretchRaw: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            w: c_int,
            h: c_int,
            cols: c_int,
            rows: c_int,
            data: *const u8,
            client: c_int,
            dirty: qboolean,
        ),
    >,
    pub UploadCinematic: Option<
        unsafe extern "C" fn(cols: c_int, rows: c_int, data: *const u8, client: c_int, dirty: qboolean),
    >,

    pub BeginFrame: Option<unsafe extern "C" fn(stereoFrame: stereoFrame_t)>,

    /// if the pointers are not NULL, timing info will be returned
    pub EndFrame: Option<unsafe extern "C" fn(frontEndMsec: *mut c_int, backEndMsec: *mut c_int)>,

    pub ProcessDissolve: Option<unsafe extern "C" fn() -> qboolean>,
    pub InitDissolve: Option<unsafe extern "C" fn(bForceCircularExtroWipe: qboolean) -> qboolean>,

    /// for use with save-games mainly...
    pub GetScreenShot: Option<unsafe extern "C" fn(data: *mut u8, w: c_int, h: c_int)>,

    // this is so you can get access to raw pixels from a graphics format (TGA/JPG/BMP etc),
    // currently only the save game uses it (to make raw shots for the autosaves)
    pub TempRawImage_ReadFromFile: Option<
        unsafe extern "C" fn(
            psLocalFilename: *const c_char,
            piWidth: *mut c_int,
            piHeight: *mut c_int,
            pbReSampleBuffer: *mut u8,
            qbVertFlip: qboolean,
        ) -> *mut u8,
    >,
    pub TempRawImage_CleanUp: Option<unsafe extern "C" fn()>,

    //misc stuff
    pub MarkFragments: Option<
        unsafe extern "C" fn(
            numPoints: c_int,
            points: *const vec3_t,
            projection: *const vec3_t,
            maxPoints: c_int,
            pointBuffer: *mut vec3_t,
            maxFragments: c_int,
            fragmentBuffer: *mut markFragment_t,
        ) -> c_int,
    >,

    //model stuff
    pub LerpTag: Option<
        unsafe extern "C" fn(
            tag: *mut orientation_t,
            model: qhandle_t,
            startFrame: c_int,
            endFrame: c_int,
            frac: c_float,
            tagName: *const c_char,
        ),
    >,
    pub ModelBounds: Option<unsafe extern "C" fn(model: qhandle_t, mins: *mut vec3_t, maxs: *mut vec3_t)>,

    // color4ub_t decays to byte* as a function parameter
    pub GetLightStyle: Option<unsafe extern "C" fn(style: c_int, color: *mut u8)>,
    pub SetLightStyle: Option<unsafe extern "C" fn(style: c_int, color: c_int)>,

    pub GetBModelVerts: Option<unsafe extern "C" fn(bmodelIndex: c_int, vec: *mut vec3_t, normal: *mut vec3_t)>,
    pub WorldEffectCommand: Option<unsafe extern "C" fn(command: *const c_char)>,

    pub RegisterFont: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    pub Font_HeightPixels: Option<unsafe extern "C" fn(index: c_int, scale: c_float) -> c_int>,
    pub Font_StrLenPixels: Option<unsafe extern "C" fn(s: *const c_char, index: c_int, scale: c_float) -> c_int>,
    pub Font_DrawString: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            s: *const c_char,
            rgba: *const c_float,
            iFontHandle: c_int,
            iMaxPixelWidth: c_int,
            scale: c_float,
        ),
    >,
    pub Font_StrLenChars: Option<unsafe extern "C" fn(s: *const c_char) -> c_int>,
    pub Language_IsAsian: Option<unsafe extern "C" fn() -> qboolean>,
    pub Language_UsesSpaces: Option<unsafe extern "C" fn() -> qboolean>,
    /// pbIsTrailingPunctuation may be NULL
    pub AnyLanguage_ReadCharFromString: Option<
        unsafe extern "C" fn(
            psText: *const c_char,
            piAdvanceCount: *mut c_int,
            pbIsTrailingPunctuation: *mut qboolean,
        ) -> c_uint,
    >,
}

const _: () = assert!(core::mem::size_of::<refexport_t>() == 384);
const _: () = assert!(core::mem::offset_of!(refexport_t, Shutdown) == 0);
const _: () = assert!(core::mem::offset_of!(refexport_t, BeginRegistration) == 8);
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterModel) == 16);
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterSkin) == 24);
const _: () = assert!(core::mem::offset_of!(refexport_t, GetAnimationCFG) == 32);
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterShader) == 40);
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterShaderNoMip) == 48);
const _: () = assert!(core::mem::offset_of!(refexport_t, LoadWorld) == 56);
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterMedia_LevelLoadBegin) == 64);
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterMedia_LevelLoadEnd) == 72);
const _: () = assert!(core::mem::offset_of!(refexport_t, SetWorldVisData) == 80);
const _: () = assert!(core::mem::offset_of!(refexport_t, EndRegistration) == 88);
const _: () = assert!(core::mem::offset_of!(refexport_t, ClearScene) == 96);
const _: () = assert!(core::mem::offset_of!(refexport_t, AddRefEntityToScene) == 104);
const _: () = assert!(core::mem::offset_of!(refexport_t, AddPolyToScene) == 112);
const _: () = assert!(core::mem::offset_of!(refexport_t, AddLightToScene) == 120);
const _: () = assert!(core::mem::offset_of!(refexport_t, RenderScene) == 128);
const _: () = assert!(core::mem::offset_of!(refexport_t, GetLighting) == 136);
const _: () = assert!(core::mem::offset_of!(refexport_t, SetColor) == 144);
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawStretchPic) == 152);
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawRotatePic) == 160);
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawRotatePic2) == 168);
const _: () = assert!(core::mem::offset_of!(refexport_t, LAGoggles) == 176);
const _: () = assert!(core::mem::offset_of!(refexport_t, Scissor) == 184);
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawStretchRaw) == 192);
const _: () = assert!(core::mem::offset_of!(refexport_t, UploadCinematic) == 200);
const _: () = assert!(core::mem::offset_of!(refexport_t, BeginFrame) == 208);
const _: () = assert!(core::mem::offset_of!(refexport_t, EndFrame) == 216);
const _: () = assert!(core::mem::offset_of!(refexport_t, ProcessDissolve) == 224);
const _: () = assert!(core::mem::offset_of!(refexport_t, InitDissolve) == 232);
const _: () = assert!(core::mem::offset_of!(refexport_t, GetScreenShot) == 240);
const _: () = assert!(core::mem::offset_of!(refexport_t, TempRawImage_ReadFromFile) == 248);
const _: () = assert!(core::mem::offset_of!(refexport_t, TempRawImage_CleanUp) == 256);
const _: () = assert!(core::mem::offset_of!(refexport_t, MarkFragments) == 264);
const _: () = assert!(core::mem::offset_of!(refexport_t, LerpTag) == 272);
const _: () = assert!(core::mem::offset_of!(refexport_t, ModelBounds) == 280);
const _: () = assert!(core::mem::offset_of!(refexport_t, GetLightStyle) == 288);
const _: () = assert!(core::mem::offset_of!(refexport_t, SetLightStyle) == 296);
const _: () = assert!(core::mem::offset_of!(refexport_t, GetBModelVerts) == 304);
const _: () = assert!(core::mem::offset_of!(refexport_t, WorldEffectCommand) == 312);
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterFont) == 320);
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_HeightPixels) == 328);
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_StrLenPixels) == 336);
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_DrawString) == 344);
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_StrLenChars) == 352);
const _: () = assert!(core::mem::offset_of!(refexport_t, Language_IsAsian) == 360);
const _: () = assert!(core::mem::offset_of!(refexport_t, Language_UsesSpaces) == 368);
const _: () = assert!(core::mem::offset_of!(refexport_t, AnyLanguage_ReadCharFromString) == 376);
