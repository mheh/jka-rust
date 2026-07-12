#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int, c_uint};

use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::mini_ref_entity_s::miniRefEntity_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::cgame::stereo_frame_t::stereoFrame_t;
use mp_qshared::shared::{markFragment_t, orientation_t, qboolean, qhandle_t, vec3_t};

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
/// client to intelligently size display elements. The vis data is a large
/// enough block of data that we go to the trouble of sharing it with the
/// clipmodel subsystem. EndRegistration will draw a tiny polygon with each
/// texture, forcing them to be loaded into card memory. A scene is built up by
/// calls to R_ClearScene and the various R_Add functions. Nothing is drawn
/// until R_RenderScene is called. If the pointers are not NULL, timing info
/// will be returned.
/// Type definition source: `oracle/codemp/renderer/tr_public.h:14-110`
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
    pub RegisterShader: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub RegisterShaderNoMip: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub ShaderNameFromIndex: Option<unsafe extern "C" fn(index: c_int) -> *const c_char>,
    pub LoadWorld: Option<unsafe extern "C" fn(name: *const c_char)>,

    // the vis data is a large enough block of data that we go to the trouble
    // of sharing it with the clipmodel subsystem
    pub SetWorldVisData: Option<unsafe extern "C" fn(vis: *const u8)>,

    /// EndRegistration will draw a tiny polygon with each texture, forcing
    /// them to be loaded into card memory
    pub EndRegistration: Option<unsafe extern "C" fn()>,

    // a scene is built up by calls to R_ClearScene and the various R_Add functions.
    // Nothing is drawn until R_RenderScene is called.
    pub ClearScene: Option<unsafe extern "C" fn()>,
    pub ClearDecals: Option<unsafe extern "C" fn()>,
    pub AddRefEntityToScene: Option<unsafe extern "C" fn(re: *const refEntity_t)>,
    pub AddMiniRefEntityToScene: Option<unsafe extern "C" fn(re: *const miniRefEntity_t)>,
    pub AddPolyToScene: Option<
        unsafe extern "C" fn(
            hShader: qhandle_t,
            numVerts: c_int,
            verts: *const polyVert_t,
            num: c_int,
        ),
    >,
    pub AddDecalToScene: Option<
        unsafe extern "C" fn(
            shader: qhandle_t,
            origin: *const vec3_t,
            dir: *const vec3_t,
            orientation: c_float,
            r: c_float,
            g: c_float,
            b: c_float,
            a: c_float,
            alphaFade: qboolean,
            radius: c_float,
            temporary: qboolean,
        ),
    >,
    pub LightForPoint: Option<
        unsafe extern "C" fn(
            point: *const vec3_t,
            ambientLight: *mut vec3_t,
            directedLight: *mut vec3_t,
            lightDir: *mut vec3_t,
        ) -> c_int,
    >,
    pub AddLightToScene: Option<
        unsafe extern "C" fn(
            org: *const vec3_t,
            intensity: c_float,
            r: c_float,
            g: c_float,
            b: c_float,
        ),
    >,
    pub AddAdditiveLightToScene: Option<
        unsafe extern "C" fn(
            org: *const vec3_t,
            intensity: c_float,
            r: c_float,
            g: c_float,
            b: c_float,
        ),
    >,
    pub RenderScene: Option<unsafe extern "C" fn(fd: *const refdef_t)>,

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
        unsafe extern "C" fn(
            cols: c_int,
            rows: c_int,
            data: *const u8,
            client: c_int,
            dirty: qboolean,
        ),
    >,

    pub BeginFrame: Option<unsafe extern "C" fn(stereoFrame: stereoFrame_t)>,

    /// if the pointers are not NULL, timing info will be returned
    pub EndFrame: Option<unsafe extern "C" fn(frontEndMsec: *mut c_int, backEndMsec: *mut c_int)>,

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

    pub LerpTag: Option<
        unsafe extern "C" fn(
            tag: *mut orientation_t,
            model: qhandle_t,
            startFrame: c_int,
            endFrame: c_int,
            frac: c_float,
            tagName: *const c_char,
        ) -> c_int,
    >,
    pub ModelBounds:
        Option<unsafe extern "C" fn(model: qhandle_t, mins: *mut vec3_t, maxs: *mut vec3_t)>,

    pub RegisterFont: Option<unsafe extern "C" fn(fontName: *const c_char) -> qhandle_t>,
    pub Font_StrLenPixels: Option<
        unsafe extern "C" fn(text: *const c_char, iFontIndex: c_int, scale: c_float) -> c_int,
    >,
    pub Font_StrLenChars: Option<unsafe extern "C" fn(text: *const c_char) -> c_int>,
    pub Font_HeightPixels: Option<unsafe extern "C" fn(iFontIndex: c_int, scale: c_float) -> c_int>,
    pub Font_DrawString: Option<
        unsafe extern "C" fn(
            ox: c_int,
            oy: c_int,
            text: *const c_char,
            rgba: *const c_float,
            setIndex: c_int,
            iCharLimit: c_int,
            scale: c_float,
        ),
    >,
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

    pub RemapShader: Option<
        unsafe extern "C" fn(
            oldShader: *const c_char,
            newShader: *const c_char,
            offsetTime: *const c_char,
        ),
    >,
    pub GetEntityToken: Option<unsafe extern "C" fn(buffer: *mut c_char, size: c_int) -> qboolean>,
    pub inPVS: Option<
        unsafe extern "C" fn(p1: *const vec3_t, p2: *const vec3_t, mask: *mut u8) -> qboolean,
    >,

    // Raven declares the param as `color4ub_t color`; a C array typedef decays
    // to `byte *` as a parameter, which is what crosses the call.
    pub GetLightStyle: Option<unsafe extern "C" fn(style: c_int, color: *mut u8)>,
    pub SetLightStyle: Option<unsafe extern "C" fn(style: c_int, color: c_int)>,

    pub GetBModelVerts:
        Option<unsafe extern "C" fn(bmodelIndex: c_int, vec: *mut vec3_t, normal: *mut vec3_t)>,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<refexport_t>() == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Shutdown) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, BeginRegistration) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterModel) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterSkin) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterShader) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterShaderNoMip) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ShaderNameFromIndex) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, LoadWorld) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, SetWorldVisData) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, EndRegistration) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ClearScene) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ClearDecals) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddRefEntityToScene) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddMiniRefEntityToScene) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddPolyToScene) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddDecalToScene) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, LightForPoint) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddLightToScene) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddAdditiveLightToScene) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RenderScene) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, SetColor) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawStretchPic) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawRotatePic) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawRotatePic2) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawStretchRaw) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, UploadCinematic) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, BeginFrame) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, EndFrame) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, MarkFragments) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, LerpTag) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ModelBounds) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterFont) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_StrLenPixels) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_StrLenChars) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_HeightPixels) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_DrawString) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Language_IsAsian) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Language_UsesSpaces) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AnyLanguage_ReadCharFromString) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RemapShader) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, GetEntityToken) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, inPVS) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, GetLightStyle) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, SetLightStyle) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(refexport_t, GetBModelVerts) == 352);

// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<refexport_t>() == 180);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Shutdown) == 0);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, BeginRegistration) == 4);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterModel) == 8);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterSkin) == 12);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterShader) == 16);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterShaderNoMip) == 20);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ShaderNameFromIndex) == 24);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, LoadWorld) == 28);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, SetWorldVisData) == 32);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, EndRegistration) == 36);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ClearScene) == 40);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ClearDecals) == 44);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddRefEntityToScene) == 48);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddMiniRefEntityToScene) == 52);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddPolyToScene) == 56);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddDecalToScene) == 60);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, LightForPoint) == 64);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddLightToScene) == 68);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AddAdditiveLightToScene) == 72);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RenderScene) == 76);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, SetColor) == 80);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawStretchPic) == 84);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawRotatePic) == 88);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawRotatePic2) == 92);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, DrawStretchRaw) == 96);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, UploadCinematic) == 100);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, BeginFrame) == 104);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, EndFrame) == 108);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, MarkFragments) == 112);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, LerpTag) == 116);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, ModelBounds) == 120);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RegisterFont) == 124);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_StrLenPixels) == 128);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_StrLenChars) == 132);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_HeightPixels) == 136);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Font_DrawString) == 140);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Language_IsAsian) == 144);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, Language_UsesSpaces) == 148);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, AnyLanguage_ReadCharFromString) == 152);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, RemapShader) == 156);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, GetEntityToken) == 160);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, inPVS) == 164);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, GetLightStyle) == 168);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, SetLightStyle) == 172);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(refexport_t, GetBModelVerts) == 176);
