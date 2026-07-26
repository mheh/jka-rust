//! `RendererCvars` — the renderer's cvar handles as one owned carrier
//! (DEC-37 A13.1).

#![allow(non_snake_case)]

use mp_qshared::shared::cvar::CvarHandle;

/// The 125 `cvar_t *` handles the renderer holds as file-scope globals, one
/// field per cvar, registered in `R_Register` and threaded as a carrier — the
/// engine-island `EngineCvars` precedent (`Common`'s `sv_*` handle block).
/// Reads go through the engine cvar table live; an R4 render-thread snapshot
/// is explicitly deferred (A13.1).
///
/// `Option<CvarHandle>` is the engine-side spelling of Raven's cached
/// `cvar_t*` (§B5 index-not-pointer): `None` is Raven's null pointer, i.e. a
/// cvar not yet registered.
///
/// Four of them (`r_drawTerrain`, `r_showFrameVariance`,
/// `r_terrainTessellate`, `r_terrainWaterOffset`) are declared and
/// registered in `tr_terrain.cpp`; the other 121 in `tr_init.cpp`.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:40-216` (declarations),
/// `:985-1205` (`R_Register`);
/// `oracle/codemp/renderer/tr_terrain.cpp:20-23,1031-1034`
#[derive(Clone, Default)]
pub struct RendererCvars {
    /// `"broadsword"` — default `"0"`, `0`.
    pub broadsword: Option<CvarHandle>,
    /// `"broadsword_dircap"` — default `"64"`, `0`.
    pub broadsword_dircap: Option<CvarHandle>,
    /// `"broadsword_dontstopanim"` — default `"0"`, `0`.
    pub broadsword_dontstopanim: Option<CvarHandle>,
    /// `"broadsword_effcorr"` — default `"1"`, `0`.
    pub broadsword_effcorr: Option<CvarHandle>,
    /// `"broadsword_extra1"` — default `"0"`, `0`.
    pub broadsword_extra1: Option<CvarHandle>,
    /// `"broadsword_extra2"` — default `"0"`, `0`.
    pub broadsword_extra2: Option<CvarHandle>,
    /// `"broadsword_kickbones"` — default `"1"`, `0`.
    pub broadsword_kickbones: Option<CvarHandle>,
    /// `"broadsword_kickorigin"` — default `"1"`, `0`.
    pub broadsword_kickorigin: Option<CvarHandle>,
    /// `"broadsword_playflop"` — default `"1"`, `0`.
    pub broadsword_playflop: Option<CvarHandle>,
    /// `"broadsword_ragtobase"` — default `"2"`, `0`.
    pub broadsword_ragtobase: Option<CvarHandle>,
    /// `"broadsword_smallbbox"` — default `"0"`, `0`.
    pub broadsword_smallbbox: Option<CvarHandle>,
    /// `"broadsword_waitforshot"` — default `"0"`, `0`.
    pub broadsword_waitforshot: Option<CvarHandle>,
    /// `"r_DynamicGlow"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_DynamicGlow: Option<CvarHandle>,
    /// `"r_DynamicGlowDelta"` — default `"0.8f"`, `CVAR_CHEAT`.
    pub r_DynamicGlowDelta: Option<CvarHandle>,
    /// `"r_DynamicGlowHeight"` — default `"240"`, `CVAR_CHEAT | CVAR_LATCH`.
    pub r_DynamicGlowHeight: Option<CvarHandle>,
    /// `"r_DynamicGlowIntensity"` — default `"1.13f"`, `CVAR_CHEAT`.
    pub r_DynamicGlowIntensity: Option<CvarHandle>,
    /// `"r_DynamicGlowPasses"` — default `"5"`, `CVAR_CHEAT`.
    pub r_DynamicGlowPasses: Option<CvarHandle>,
    /// `"r_DynamicGlowSoft"` — default `"1"`, `CVAR_CHEAT`.
    pub r_DynamicGlowSoft: Option<CvarHandle>,
    /// `"r_DynamicGlowWidth"` — default `"320"`, `CVAR_CHEAT | CVAR_LATCH`.
    pub r_DynamicGlowWidth: Option<CvarHandle>,
    /// `"r_allowExtensions"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_allowExtensions: Option<CvarHandle>,
    /// `"r_ambientScale"` — default `"0.6"`, `CVAR_CHEAT`.
    pub r_ambientScale: Option<CvarHandle>,
    /// `"r_autoMap"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_autoMap: Option<CvarHandle>,
    /// `"r_autoMapBackAlpha"` — default `"0"`, `0`.
    pub r_autoMapBackAlpha: Option<CvarHandle>,
    /// `"r_autoMapDisable"` — default `"1"`, `0`.
    pub r_autoMapDisable: Option<CvarHandle>,
    /// `"r_autolodscalevalue"` — default `"0"`, `CVAR_ROM`.
    pub r_autolodscalevalue: Option<CvarHandle>,
    /// `"r_clear"` — default `"0"`, `CVAR_CHEAT`.
    pub r_clear: Option<CvarHandle>,
    /// `"r_colorMipLevels"` — default `"0"`, `CVAR_LATCH`.
    pub r_colorMipLevels: Option<CvarHandle>,
    /// `"r_colorbits"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_colorbits: Option<CvarHandle>,
    /// `"r_cullRoofFaces"` — default `"0"`, `CVAR_CHEAT`.
    pub r_cullRoofFaces: Option<CvarHandle>,
    /// `"r_customheight"` — default `"1024"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_customheight: Option<CvarHandle>,
    /// `"r_customwidth"` — default `"1600"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_customwidth: Option<CvarHandle>,
    /// `"r_debuglight"` — default `"0"`, `CVAR_TEMP`.
    pub r_debugLight: Option<CvarHandle>,
    /// `"r_debugSort"` — default `"0"`, `CVAR_CHEAT`.
    pub r_debugSort: Option<CvarHandle>,
    /// `"r_debugSurface"` — default `"0"`, `CVAR_CHEAT`.
    pub r_debugSurface: Option<CvarHandle>,
    /// `"r_depthbits"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_depthbits: Option<CvarHandle>,
    /// `"r_detailtextures"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_detailTextures: Option<CvarHandle>,
    /// `"r_directedScale"` — default `"1"`, `CVAR_CHEAT`.
    pub r_directedScale: Option<CvarHandle>,
    /// `"r_displayRefresh"` — default `"0"`, `CVAR_LATCH`.
    pub r_displayRefresh: Option<CvarHandle>,
    /// `"r_dlightStyle"` — default `"1"`, `CVAR_TEMP`.
    pub r_dlightStyle: Option<CvarHandle>,
    /// `"r_drawSun"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_drawSun: Option<CvarHandle>,
    /// `"r_drawTerrain"` — default `"1"`, `CVAR_CHEAT`.
    pub r_drawTerrain: Option<CvarHandle>,
    /// `"r_drawentities"` — default `"1"`, `CVAR_CHEAT`.
    pub r_drawentities: Option<CvarHandle>,
    /// `"r_drawfog"` — default `"2"`, `CVAR_CHEAT`.
    pub r_drawfog: Option<CvarHandle>,
    /// `"r_drawworld"` — default `"1"`, `CVAR_CHEAT`.
    pub r_drawworld: Option<CvarHandle>,
    /// `"r_dynamiclight"` — default `"1"`, `CVAR_ARCHIVE`.
    pub r_dynamiclight: Option<CvarHandle>,
    /// `"r_ext_compiled_vertex_array"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ext_compiled_vertex_array: Option<CvarHandle>,
    /// `"r_ext_compress_lightmaps"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ext_compressed_lightmaps: Option<CvarHandle>,
    /// `"r_ext_compress_textures"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ext_compressed_textures: Option<CvarHandle>,
    /// `"r_ext_gamma_control"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ext_gamma_control: Option<CvarHandle>,
    /// `"r_ext_multitexture"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ext_multitexture: Option<CvarHandle>,
    /// `"r_ext_preferred_tc_method"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ext_preferred_tc_method: Option<CvarHandle>,
    /// `"r_ext_texture_env_add"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ext_texture_env_add: Option<CvarHandle>,
    /// `"r_ext_texture_filter_anisotropic"` — default `"16"`, `CVAR_ARCHIVE`.
    pub r_ext_texture_filter_anisotropic: Option<CvarHandle>,
    /// `"r_facePlaneCull"` — default `"1"`, `CVAR_ARCHIVE`.
    pub r_facePlaneCull: Option<CvarHandle>,
    /// `"r_fastsky"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_fastsky: Option<CvarHandle>,
    /// `"r_finish"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_finish: Option<CvarHandle>,
    /// `"r_flares"` — default `"1"`, `CVAR_ARCHIVE`.
    pub r_flares: Option<CvarHandle>,
    /// `"r_fullbright"` — default `"0"`, `CVAR_CHEAT`.
    pub r_fullbright: Option<CvarHandle>,
    /// `"r_fullscreen"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_fullscreen: Option<CvarHandle>,
    /// `"r_gamma"` — default `"1.2"`, `CVAR_ARCHIVE`.
    pub r_gamma: Option<CvarHandle>,
    /// `"r_ignore"` — default `"1"`, `CVAR_CHEAT`.
    pub r_ignore: Option<CvarHandle>,
    /// `"r_ignoreGLErrors"` — default `"1"`, `CVAR_ARCHIVE`.
    pub r_ignoreGLErrors: Option<CvarHandle>,
    /// `"r_ignorehwgamma"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_ignorehwgamma: Option<CvarHandle>,
    /// `"r_inGameVideo"` — default `"1"`, `CVAR_ARCHIVE`.
    pub r_inGameVideo: Option<CvarHandle>,
    /// `"r_intensity"` — default `"1"`, `CVAR_LATCH`.
    pub r_intensity: Option<CvarHandle>,
    /// `"r_lightmap"` — default `"0"`, `CVAR_CHEAT`.
    pub r_lightmap: Option<CvarHandle>,
    /// `"r_lockpvs"` — default `"0"`, `CVAR_CHEAT`.
    pub r_lockpvs: Option<CvarHandle>,
    /// `"r_lodCurveError"` — default `"250"`, `CVAR_ARCHIVE`.
    pub r_lodCurveError: Option<CvarHandle>,
    /// `"r_lodbias"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_lodbias: Option<CvarHandle>,
    /// `"r_lodscale"` — default `"5"`, `0`.
    pub r_lodscale: Option<CvarHandle>,
    /// `"r_logFile"` — default `"0"`, `CVAR_CHEAT`.
    pub r_logFile: Option<CvarHandle>,
    /// `"r_markcount"` — default `"100"`, `CVAR_ARCHIVE`.
    pub r_markcount: Option<CvarHandle>,
    /// `"r_maxpolys"` — default `MAX_POLYS` (600), `0`.
    pub r_maxpolys: Option<CvarHandle>,
    /// `"r_maxpolyverts"` — default `MAX_POLYVERTS` (3000), `0`.
    pub r_maxpolyverts: Option<CvarHandle>,
    /// `"r_measureOverdraw"` — default `"0"`, `CVAR_CHEAT`.
    pub r_measureOverdraw: Option<CvarHandle>,
    /// `"r_mode"` — default `"4"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_mode: Option<CvarHandle>,
    /// `"r_modelpoolmegs"` — default `"20"`, `CVAR_ARCHIVE`.
    pub r_modelpoolmegs: Option<CvarHandle>,
    /// `"r_noserverghoul2"` — default `"0"`, `CVAR_CHEAT`.
    pub r_noServerGhoul2: Option<CvarHandle>,
    /// `"r_nobind"` — default `"0"`, `CVAR_CHEAT`.
    pub r_nobind: Option<CvarHandle>,
    /// `"r_nocull"` — default `"0"`, `CVAR_CHEAT`.
    pub r_nocull: Option<CvarHandle>,
    /// `"r_nocurves"` — default `"0"`, `CVAR_CHEAT`.
    pub r_nocurves: Option<CvarHandle>,
    /// `"r_noportals"` — default `"0"`, `CVAR_CHEAT`.
    pub r_noportals: Option<CvarHandle>,
    /// `"r_norefresh"` — default `"0"`, `CVAR_CHEAT`.
    pub r_norefresh: Option<CvarHandle>,
    /// `"r_novis"` — default `"0"`, `CVAR_CHEAT`.
    pub r_novis: Option<CvarHandle>,
    /// `"r_offsetfactor"` — default `"-1"`, `CVAR_CHEAT`.
    pub r_offsetFactor: Option<CvarHandle>,
    /// `"r_offsetunits"` — default `"-2"`, `CVAR_CHEAT`.
    pub r_offsetUnits: Option<CvarHandle>,
    /// `"r_overBrightBits"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_overBrightBits: Option<CvarHandle>,
    /// `"r_picmip"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_picmip: Option<CvarHandle>,
    /// `"r_portalOnly"` — default `"0"`, `CVAR_CHEAT`.
    pub r_portalOnly: Option<CvarHandle>,
    /// `"r_primitives"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_primitives: Option<CvarHandle>,
    /// `"r_roofCullCeilDist"` — default `"256"`, `CVAR_CHEAT`.
    pub r_roofCullCeilDist: Option<CvarHandle>,
    /// `"r_roofCeilFloorDist"` — default `"128"`, `CVAR_CHEAT`.
    pub r_roofCullFloorDist: Option<CvarHandle>,
    /// `"cg_shadows"` — default `"1"`, `0`.
    pub r_shadows: Option<CvarHandle>,
    /// `"r_showFrameVariance"` — default `"0"`, `0`.
    pub r_showFrameVariance: Option<CvarHandle>,
    /// `"r_showImages"` — default `"0"`, `CVAR_CHEAT`.
    pub r_showImages: Option<CvarHandle>,
    /// `"r_showcluster"` — default `"0"`, `CVAR_CHEAT`.
    pub r_showcluster: Option<CvarHandle>,
    /// `"r_shownormals"` — default `"0"`, `CVAR_CHEAT`.
    pub r_shownormals: Option<CvarHandle>,
    /// `"r_showsky"` — default `"0"`, `CVAR_CHEAT`.
    pub r_showsky: Option<CvarHandle>,
    /// `"r_showtris"` — default `"0"`, `CVAR_CHEAT`.
    pub r_showtris: Option<CvarHandle>,
    /// `"r_simpleMipMaps"` — default `"1"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_simpleMipMaps: Option<CvarHandle>,
    /// `"r_singleShader"` — default `"0"`, `CVAR_CHEAT | CVAR_LATCH`.
    pub r_singleShader: Option<CvarHandle>,
    /// `"r_skipBackEnd"` — default `"0"`, `CVAR_CHEAT`.
    pub r_skipBackEnd: Option<CvarHandle>,
    /// `"r_speeds"` — default `"0"`, `CVAR_CHEAT`.
    pub r_speeds: Option<CvarHandle>,
    /// `"r_stencilbits"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_stencilbits: Option<CvarHandle>,
    /// `"r_stereo"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_stereo: Option<CvarHandle>,
    /// `"r_subdivisions"` — default `"4"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_subdivisions: Option<CvarHandle>,
    /// `"r_surfaceSprites"` — default `"1"`, `CVAR_TEMP`.
    pub r_surfaceSprites: Option<CvarHandle>,
    /// `"r_surfaceWeather"` — default `"0"`, `CVAR_TEMP`.
    pub r_surfaceWeather: Option<CvarHandle>,
    /// `"r_swapInterval"` — default `"0"`, `CVAR_ARCHIVE`.
    pub r_swapInterval: Option<CvarHandle>,
    /// `"r_terrainTessellate"` — default `"3"`, `CVAR_CHEAT`.
    pub r_terrainTessellate: Option<CvarHandle>,
    /// `"r_terrainWaterOffset"` — default `"0"`, `0`.
    pub r_terrainWaterOffset: Option<CvarHandle>,
    /// `"r_textureMode"` — default `"GL_LINEAR_MIPMAP_NEAREST"`, `CVAR_ARCHIVE`.
    pub r_textureMode: Option<CvarHandle>,
    /// `"r_texturebits"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_texturebits: Option<CvarHandle>,
    /// `"r_texturebitslm"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_texturebitslm: Option<CvarHandle>,
    /// `"r_uifullscreen"` — default `"0"`, `0`.
    pub r_uiFullScreen: Option<CvarHandle>,
    /// `"r_verbose"` — default `"0"`, `CVAR_CHEAT`.
    pub r_verbose: Option<CvarHandle>,
    /// `"r_vertexLight"` — default `"0"`, `CVAR_ARCHIVE | CVAR_LATCH`.
    pub r_vertexLight: Option<CvarHandle>,
    /// `"r_windAngle"` — default `"0"`, `0`.
    pub r_windAngle: Option<CvarHandle>,
    /// `"r_windDampFactor"` — default `"0.1"`, `0`.
    pub r_windDampFactor: Option<CvarHandle>,
    /// `"r_windGust"` — default `"0"`, `0`.
    pub r_windGust: Option<CvarHandle>,
    /// `"r_windPointForce"` — default `"0"`, `0`.
    pub r_windPointForce: Option<CvarHandle>,
    /// `"r_windPointX"` — default `"0"`, `0`.
    pub r_windPointX: Option<CvarHandle>,
    /// `"r_windPointY"` — default `"0"`, `0`.
    pub r_windPointY: Option<CvarHandle>,
    /// `"r_windSpeed"` — default `"0"`, `0`.
    pub r_windSpeed: Option<CvarHandle>,
    /// `"r_znear"` — default `"2"`, `CVAR_CHEAT`.
    pub r_znear: Option<CvarHandle>,
}
