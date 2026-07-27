//! Raven `tr_image.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_image.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use std::f32::consts::PI;
use std::sync::Arc;

use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::Com_DPrintf;
use mp_engine_qcommon::cvar_fns::Cvar_Set;
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Read, FS_ReadFileVec};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_color::{S_COLOR_RED, S_COLOR_YELLOW};
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{fileHandle_t, MAX_QPATH};
use native_math::qmath::Com_Clamp;

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::image_asset::{ImageAsset, ImageHandle};
use crate::render_state::placeholders::GlConfig;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::tr_globals_t::FOG_TABLE_SIZE;
use crate::tr_model::render_models::RenderModels;

// PORT-NOTE: several functions below read/write fields on two root types
// this crate owns elsewhere (`render_state::placeholders::{GlConfig,
// FunctionTables}`) that are still empty placeholder structs pending their
// owning wave's fields. `tr_init.rs`'s already-landed `GfxInfo_f` reads
// `RenderAssets::glconfig` the same way (`.color_bits`, `.vid_width`,
// `.vid_height`, …) against the same empty `GlConfig {}` — this transcriber
// follows that established multi-wave-fills-one-struct pattern rather than
// inventing a divergent shape: `GlConfig::{color_bits, vid_width,
// vid_height}` match `tr_init.rs`'s exact field names;
// `GlConfig::{device_supports_gamma, is_fullscreen}` are this file's own
// addition (same snake_case-of-Raven-name convention: `deviceSupportsGamma`,
// `isFullscreen`); `FunctionTables::fog_table: [f32; FOG_TABLE_SIZE]` is
// likewise assumed. The wave that lands `placeholders.rs`'s interiors
// reconciles these. Where a field has no analogous precedent to lean on
// (`trGlobals_t`'s `overbrightBits`/`identityLight`/`identityLightByte`
// frontend scratch, homed on `FrameState`), this file follows
// `tr_init.rs`'s `GfxInfo_f` precedent instead and DEFERS the write rather
// than inventing the field (porting-rules §A2, no speculative behavior) —
// see `R_SetColorMappings` below.

// PORT-NOTE (wave 1): this wave's fns (`R_SumOfUsedImages`, `R_ImageList_f`,
// `R_Images_DeleteImage`, `R_Images_Clear`, `RE_RegisterImages_Info_f`,
// `R_FindImageFile_NoLoad`) read/write per-image fields on
// `crate::render_state::image_asset::ImageAsset` and a frame counter on
// `crate::render_state::frame_state::FrameState` that do not exist yet —
// `ImageAsset` is still `pub struct ImageAsset {}` (its own doc comment:
// "fields land with the tr_image R3 wave", which is this wave, but this
// packet's scope restricts every transcriber to their one target file, so
// `image_asset.rs`/`frame_state.rs` cannot be touched from here). This
// extends the exact multi-wave-fills-one-struct pattern the file-top
// PORT-NOTE above already established for `GlConfig`/`FunctionTables`:
// fields are referenced under their expected names/types below, and the
// wave/integrator that can edit those files adds them verbatim. Required
// additions, snake-case of the Raven `image_t`
// (`oracle/codemp/renderer/tr_local.h:136-151`) / `trGlobals_t` names:
//
// `ImageAsset`:
//   - `img_name: String`        (`imgName[64]`)
//   - `width: i32`
//   - `height: i32`
//   - `mipmap: bool`
//   - `allow_picmip: bool`
//   - `wrap_clamp_mode: i32`    (`wrapClampMode`, a stored GLenum)
//   - `internal_format: i32`   (`internalFormat`, a stored GLenum)
//   - `frame_used: i32`         (`frameUsed`)
//   - `last_level_used_on: i32` (`iLastLevelUsedOn`)
//
// `FrameState`:
//   - `frame_count: i32` (`tr.frameCount` — this wave's packet explicitly
//     SPLITs `R_SumOfUsedImages`'s `tr` read across `RenderAssets` +
//     `FrameState`, matching R2's frontend-scratch-counter row)
//
// Flagged as an escalation per the preamble ("A state home this packet
// marks UNMAPPED is an ESCALATION, never an invention") rather than
// silently invented past this file's boundary.

/// Per-subsystem state for `tr_image.cpp`'s two 256-entry gamma/intensity
/// lookup tables — Raven file-scope statics `s_gammatable`/`s_intensitytable`
/// with no R2 carrier of their own (kind-3, genuine cross-frame state);
/// named by this wave per DEC-37 A13.3. Written by `R_SetColorMappings`,
/// read by `R_GammaCorrect`/`R_LightScaleTexture`. Zero-initialized to match
/// the oracle's zero-filled `static` arrays before the first
/// `R_SetColorMappings` call.
///
/// Extended by wave 1 with three more `tr_image.cpp` file-scope statics
/// (kind-3, A13.3): `gl_filter_min`/`gl_filter_max` (`GL_TextureMode`'s
/// selected minify/magnify GL filter enum) and `giTextureBindNum` (the next
/// GL texture name to hand out, reset by `R_Images_Clear`).
///
/// Source: `oracle/codemp/renderer/tr_image.cpp` (file-scope statics near
/// `R_SetColorMappings`, `:2847-2919`; `gl_filter_min`/`gl_filter_max`
/// near `:90-97`; `giTextureBindNum` near `:1030`)
#[derive(Clone)]
pub struct TrImageState {
    /// Raven `byte s_gammatable[256]`.
    pub gamma_table: [u8; 256],
    /// Raven `byte s_intensitytable[256]`.
    pub intensity_table: [u8; 256],
    /// Raven file-scope `int gl_filter_min`.
    pub gl_filter_min: i32,
    /// Raven file-scope `int gl_filter_max`.
    pub gl_filter_max: i32,
    /// Raven file-scope `int giTextureBindNum`.
    pub gi_texture_bind_num: i32,
}

impl Default for TrImageState {
    fn default() -> TrImageState {
        TrImageState {
            gamma_table: [0; 256],
            intensity_table: [0; 256],
            gl_filter_min: 0,
            gl_filter_max: 0,
            gi_texture_bind_num: 0,
        }
    }
}

// PORT-NOTE: standard OpenGL / `GL_EXT_texture_compression_s3tc` enum
// values, transcribed from general knowledge rather than the oracle header
// (this wave's rule forbids opening `oracle/`) — flag for a header diff if a
// reviewer has direct access. `GL_RGB4_S3TC` mirrors the pre-EXT id
// Software `tr_image.c` define of the same name/value.
const GL_RGBA4: i32 = 0x8056;
const GL_RGB5: i32 = 0x8050;
const GL_RGBA8: i32 = 0x8058;
const GL_RGB8: i32 = 0x8051;
const GL_RGB4_S3TC: i32 = 0x83A1;
const GL_COMPRESSED_RGB_S3TC_DXT1_EXT: i32 = 0x83F0;
const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT: i32 = 0x83F3;

/// Raven `#define LANCZOS3 3.0` — `R_Resample`'s filter-window radius. Not
/// guessed: this exact value is already load-bearing in this same file as
/// `Lanczos3`'s own cutoff (`if t < 3.0`, wave 0, `tr_image.cpp:2352-2364`),
/// so naming it here reuses a value already present in the target file
/// rather than inventing one (wave law: never guess a `#define` not in the
/// packet/target file).
const LANCZOS3: f32 = 3.0;

/// Raven `R_GammaCorrect`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:47-53
pub fn R_GammaCorrect(buffer: &mut [u8], state: &TrImageState) {
    for b in buffer.iter_mut() {
        *b = state.gamma_table[*b as usize];
    }
}

/// Raven `GenerateImageMappingName` — `sName[MAX_QPATH]` was a rotating
/// scratch/return buffer (kind-2, three-kind rule); the R3 shape returns an
/// owned `String` instead of a pointer into hidden state.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:72-88
pub fn GenerateImageMappingName(name: &str) -> String {
    let mut out = String::new();
    for &b in name.as_bytes() {
        if out.len() >= MAX_QPATH as usize - 1 {
            break;
        }
        let mut letter = (b as char).to_ascii_lowercase();
        if letter == '.' {
            break; // don't include extension
        }
        if letter == '\\' {
            letter = '/'; // damn path names
        }
        out.push(letter);
    }
    out
}

/// Raven `R_BytesPerTex`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:145-199
pub fn R_BytesPerTex(glconfig: &GlConfig, format: i32) -> f32 {
    match format {
        1 => 1.0,                                   // "I    "
        2 => 2.0,                                   // "IA   "
        3 => glconfig.color_bits as f32 / 8.0,      // "RGB  "
        4 => glconfig.color_bits as f32 / 8.0,      // "RGBA "
        GL_RGBA4 => 2.0,                            // "RGBA4"
        GL_RGB5 => 2.0,                             // "RGB5 "
        GL_RGBA8 => 4.0,                            // "RGBA8"
        GL_RGB8 => 4.0,                             // "RGB8"
        GL_RGB4_S3TC => 0.33333,                    // "S3TC "
        GL_COMPRESSED_RGB_S3TC_DXT1_EXT => 0.33333, // "DXT1 "
        GL_COMPRESSED_RGBA_S3TC_DXT5_EXT => 1.0,    // "DXT5 "
        _ => 4.0,                                   // "???? "
    }
}

/// Raven `R_LightScaleTexture`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:325-373
pub fn R_LightScaleTexture(
    in_: &mut [u32],
    inwidth: i32,
    inheight: i32,
    only_gamma: bool,
    glconfig: &GlConfig,
    state: &TrImageState,
) {
    let count = (inwidth * inheight).max(0) as usize;

    if only_gamma {
        if !glconfig.device_supports_gamma {
            for px in in_.iter_mut().take(count) {
                let mut b = px.to_le_bytes();
                b[0] = state.gamma_table[b[0] as usize];
                b[1] = state.gamma_table[b[1] as usize];
                b[2] = state.gamma_table[b[2] as usize];
                *px = u32::from_le_bytes(b);
            }
        }
    } else if glconfig.device_supports_gamma {
        for px in in_.iter_mut().take(count) {
            let mut b = px.to_le_bytes();
            b[0] = state.intensity_table[b[0] as usize];
            b[1] = state.intensity_table[b[1] as usize];
            b[2] = state.intensity_table[b[2] as usize];
            *px = u32::from_le_bytes(b);
        }
    } else {
        for px in in_.iter_mut().take(count) {
            let mut b = px.to_le_bytes();
            b[0] = state.gamma_table[state.intensity_table[b[0] as usize] as usize];
            b[1] = state.gamma_table[state.intensity_table[b[1] as usize] as usize];
            b[2] = state.gamma_table[state.intensity_table[b[2] as usize] as usize];
            *px = u32::from_le_bytes(b);
        }
    }
}

/// Raven `R_MipMap2` — box-filter downsample in place. Raven's
/// `Hunk_AllocateTempMemory`/`Hunk_FreeTempMemory` scratch pair becomes an
/// owned local `Vec` (porting-rules §C9: manual alloc/free -> ownership);
/// `(byte*)in` casts become little-endian byte views via `to_le_bytes`/
/// `from_le_bytes` (native target is little-endian; matches the oracle's
/// byte-order-dependent channel unpacking exactly).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:384-430
pub fn R_MipMap2(in_: &mut [u32], in_width: i32, in_height: i32) {
    let out_width = in_width >> 1;
    let out_height = in_height >> 1;
    let mut temp = vec![0u32; (out_width * out_height).max(0) as usize];

    let in_width_mask = in_width - 1;
    let in_height_mask = in_height - 1;

    let src: &[u32] = in_;
    let sample = |y: i32, x: i32| -> [u8; 4] {
        let idx = (((y & in_height_mask) * in_width) + (x & in_width_mask)) as usize;
        src[idx].to_le_bytes()
    };

    for i in 0..out_height {
        for j in 0..out_width {
            let mut out_pixel = [0u8; 4];
            for k in 0..4usize {
                let total: i32 = 1 * sample(i * 2 - 1, j * 2 - 1)[k] as i32
                    + 2 * sample(i * 2 - 1, j * 2)[k] as i32
                    + 2 * sample(i * 2 - 1, j * 2 + 1)[k] as i32
                    + 1 * sample(i * 2 - 1, j * 2 + 2)[k] as i32
                    + 2 * sample(i * 2, j * 2 - 1)[k] as i32
                    + 4 * sample(i * 2, j * 2)[k] as i32
                    + 4 * sample(i * 2, j * 2 + 1)[k] as i32
                    + 2 * sample(i * 2, j * 2 + 2)[k] as i32
                    + 2 * sample(i * 2 + 1, j * 2 - 1)[k] as i32
                    + 4 * sample(i * 2 + 1, j * 2)[k] as i32
                    + 4 * sample(i * 2 + 1, j * 2 + 1)[k] as i32
                    + 2 * sample(i * 2 + 1, j * 2 + 2)[k] as i32
                    + 1 * sample(i * 2 + 2, j * 2 - 1)[k] as i32
                    + 2 * sample(i * 2 + 2, j * 2)[k] as i32
                    + 2 * sample(i * 2 + 2, j * 2 + 1)[k] as i32
                    + 1 * sample(i * 2 + 2, j * 2 + 2)[k] as i32;
                out_pixel[k] = (total / 36) as u8;
            }
            temp[(i * out_width + j) as usize] = u32::from_le_bytes(out_pixel);
        }
    }

    in_[..temp.len()].copy_from_slice(&temp);
}

/// Raven `R_BlendOverTexture`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:487-502
pub fn R_BlendOverTexture(data: &mut [u8], pixel_count: i32, blend: [u8; 4]) {
    let inverse_alpha = 255i32 - blend[3] as i32;
    let premult = [
        blend[0] as i32 * blend[3] as i32,
        blend[1] as i32 * blend[3] as i32,
        blend[2] as i32 * blend[3] as i32,
    ];
    for i in 0..pixel_count.max(0) as usize {
        let p = i * 4;
        data[p] = ((data[p] as i32 * inverse_alpha + premult[0]) >> 9) as u8;
        data[p + 1] = ((data[p + 1] as i32 * inverse_alpha + premult[1]) >> 9) as u8;
        data[p + 2] = ((data[p + 2] as i32 * inverse_alpha + premult[2]) >> 9) as u8;
    }
}

// PORT-NOTE: Raven `CStringComparator::operator()`
// (oracle/codemp/renderer/tr_image.cpp:530) was the
// `std::map<char*, image_t*, CStringComparator>` comparator for
// `AllocatedImages`. R2 replaces that map with `RenderAssets::images:
// Arena<ImageAsset>` + `image_names: HashMap<String, ImageHandle>`
// (`R2-D3`/`R2-D4`) — the comparator has zero live consumers in that shape.
// Dropped per porting-rules §20 (dead surface, not ported speculatively).

/// Raven `R_Images_StartIteration`. R2 assigns `itAllocatedImages`'
/// std::map-iterator role NO carrier of its own ("an arena iteration is a
/// local `images.iter()` at the R3 body, not stored state") — this returns
/// the count only; see `R_Images_GetNextIteration` for the walk itself.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:541-545
pub fn R_Images_StartIteration(assets: &RenderAssets) -> usize {
    assets.images.iter().count()
}

/// Raven `R_Images_GetNextIteration`.
///
/// PORT-NOTE: `itAllocatedImages`'s std::map iterator has no R2 carrier
/// (see `R_Images_StartIteration`). The two-call C++ iterator protocol
/// becomes an explicit `cursor` the caller threads between calls (porting-
/// rules §B4: state is threaded, not reached), replacing the TU-local
/// static; iteration order is the arena's insertion order rather than the
/// oracle's `std::map` key-sorted order (accepted per ruling 1 — the
/// interior is free, R2-D3/R2-D4 already replaced the map).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:547-555
pub fn R_Images_GetNextIteration(assets: &RenderAssets, cursor: &mut usize) -> Option<ImageHandle> {
    let result = assets.images.iter().nth(*cursor).map(|(handle, _)| handle);
    *cursor += 1;
    result
}

/// Raven `R_Images_DeleteImageContents`.
///
/// Raven: `assert(pImage); // should never be called with NULL` — dropped;
/// an `ImageHandle` cannot be null (§B5 index-not-pointer). `Z_Free(pImage)`
/// is replaced by the arena slot's own drop (owned Rust storage, porting-
/// rules §C9). Registry mutation goes through `Arc::make_mut` (A9), matching
/// every other `RenderAssets` registry write.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:561-571
pub fn R_Images_DeleteImageContents(sim: &mut RenderAssetsSim, handle: ImageHandle) {
    // DEFERRED: R4 — qglDeleteTextures(1, &pImage->texnum): the fixed-
    // function GL surface; R2 leaves GL entry points unhomed until the R4
    // wgpu rewrite (DEC-01/DEC-37 A13.2; `GpuResources::gl_state` named
    // placeholder).
    // Source: oracle/codemp/renderer/tr_image.cpp:566-567
    Arc::make_mut(&mut sim.published).images.remove(handle);
}

/// Raven's `switch (pHeader->byImagePlanes)` RGB/greyscale pixel reader
/// (`LoadTGA`, image type 2/3) — private helper, not itself an oracle fn.
fn read_tga_pixel_rgb(
    buf: &[u8],
    pos: &mut usize,
    by_image_planes: u8,
) -> Option<(u8, u8, u8, u8)> {
    match by_image_planes {
        8 => {
            let b = buf[*pos];
            *pos += 1;
            Some((b, b, b, 255))
        }
        24 => {
            let (b, g, r) = (buf[*pos], buf[*pos + 1], buf[*pos + 2]);
            *pos += 3;
            Some((r, g, b, 255))
        }
        32 => {
            let (b, g, r, a) = (buf[*pos], buf[*pos + 1], buf[*pos + 2], buf[*pos + 3]);
            *pos += 4;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

/// Raven's `switch (pHeader->byImagePlanes)` RLE pixel reader (`LoadTGA`,
/// image type 10) — 24/32-bit planes only (no 8-bit case, matching the
/// oracle's separate switch). Private helper, not itself an oracle fn.
fn read_tga_pixel_rle(
    buf: &[u8],
    pos: &mut usize,
    by_image_planes: u8,
) -> Option<(u8, u8, u8, u8)> {
    match by_image_planes {
        24 => {
            let (b, g, r) = (buf[*pos], buf[*pos + 1], buf[*pos + 2]);
            *pos += 3;
            Some((r, g, b, 255))
        }
        32 => {
            let (b, g, r, a) = (buf[*pos], buf[*pos + 1], buf[*pos + 2], buf[*pos + 3]);
            *pos += 4;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

/// Raven `LoadTGAPalletteImage`.
///
/// Out-params (`pic`/`width`/`height`) collapse to a return value (§C7);
/// `TargaHeader_t` fields not read again after parsing (`colormap_index`,
/// `x/y_origin`, `pixel_size`, `attributes`) are skipped by cursor advance
/// rather than bound to names, matching their dead-read status in the
/// oracle.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1318-1387
pub fn LoadTGAPalletteImage(view: &mut EngineHostView, name: &str) -> Option<(Vec<u8>, i32, i32)> {
    let buf = FS_ReadFileVec(view, name)?;

    let mut p = 0usize;
    let id_length = buf[p];
    p += 1;
    let colormap_type = buf[p];
    p += 1;
    let image_type = buf[p];
    p += 1;
    p += 2; // colormap_index — parsed by Raven, never read again
    let colormap_length = u16::from_le_bytes([buf[p], buf[p + 1]]);
    p += 2;
    let colormap_size = buf[p];
    p += 1;
    p += 2; // x_origin
    p += 2; // y_origin
    let width = u16::from_le_bytes([buf[p], buf[p + 1]]) as i32;
    p += 2;
    let height = u16::from_le_bytes([buf[p], buf[p + 1]]) as i32;
    p += 2;
    p += 2; // pixel_size, attributes — parsed by Raven, never read again

    if image_type != 1 {
        com_error(
            errorParm_t::ERR_DROP,
            "LoadTGAPalletteImage: Only type 1 (uncompressed pallettised) TGA images supported\n"
                .to_string(),
        );
    }
    if colormap_type == 0 {
        com_error(
            errorParm_t::ERR_DROP,
            "LoadTGAPalletteImage: colormaps ONLY supported\n".to_string(),
        );
    }

    let num_pixels = (width * height).max(0) as usize;

    if id_length != 0 {
        p += id_length as usize; // skip TARGA image comment
    }
    let data_start = p + (colormap_length as usize) * (colormap_size as usize / 4);
    let pic = buf[data_start..data_start + num_pixels].to_vec();

    Some((pic, width, height))
}

/// Raven `LoadTGA`.
///
/// Out-params collapse to a return value (§C7); `Com_Error`/`goto TGADone`
/// become an `Option<&str>` short-circuit chain; the oracle's single
/// `TGADone:` exit was there to run `FS_FreeFile`, which the owned
/// [`FS_ReadFileVec`] buffer's drop now does on every path.
/// Preserves the oracle's own `x`-direction quirk verbatim (see the
/// comment at the RGB/greyscale branch below).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1421-1743
pub fn LoadTGA(view: &mut EngineHostView, name: &str) -> Option<(Vec<u8>, i32, i32)> {
    let buf = FS_ReadFileVec(view, name)?;

    // `TGAHeader_t` — 18-byte fixed layout (id_length, colormap_type,
    // image_type, colormap_origin/length/depth, x/y origin, width, height,
    // bits-per-pixel, image-descriptor).
    let by_id_field_length = buf[0];
    let by_colourmap_type = buf[1];
    let by_image_type = buf[2];
    let w1st_colourmap_entry = u16::from_le_bytes([buf[3], buf[4]]);
    let w_colourmap_length = u16::from_le_bytes([buf[5], buf[6]]);
    let by_colourmap_entry_size = buf[7];
    let w_image_width = u16::from_le_bytes([buf[12], buf[13]]) as i32;
    let w_image_height = u16::from_le_bytes([buf[14], buf[15]]) as i32;
    let by_image_planes = buf[16];
    let by_scan_line_order = buf[17];

    let error: Option<&str> = if by_colourmap_type != 0 {
        Some("LoadTGA: colourmaps not supported\n")
    } else if by_image_type != 2 && by_image_type != 3 && by_image_type != 10 {
        Some("LoadTGA: Only type 2 (RGB), 3 (gray), and 10 (RLE-RGB) images supported\n")
    } else if w1st_colourmap_entry != 0 {
        Some("LoadTGA: colourmaps not supported\n")
    } else if w_colourmap_length != 0 && w_colourmap_length != 256 {
        Some("LoadTGA: ColourMapLength must be either 0 or 256\n")
    } else if by_colourmap_entry_size != 0 && by_colourmap_entry_size != 24 {
        Some("LoadTGA: ColourMapEntrySize must be either 0 or 24\n")
    } else if (by_image_planes != 24 && by_image_planes != 32)
        && (by_image_planes != 8 && by_image_type != 3)
    {
        Some("LoadTGA: Only type 2 (RGB), 3 (gray), and 10 (RGB) TGA images supported\n")
    } else if !matches!(by_scan_line_order & 0x30, 0x00 | 0x10 | 0x20 | 0x30) {
        Some("LoadTGA: ScanLineOrder must be either 0x00,0x10,0x20, or 0x30\n")
    } else if by_image_type == 10 && (by_scan_line_order & 0x30) != 0x00 {
        Some("LoadTGA: RLE-RGB Images (type 10) must be in bottom-to-top format\n")
    } else if by_image_type == 10 && by_image_planes != 24 && by_image_planes != 32 {
        Some("LoadTGA: RLE-RGB Images (type 10) must be 24 or 32 bit\n")
    } else {
        None
    };

    if let Some(msg) = error {
        com_error(
            errorParm_t::ERR_DROP,
            format!("{}( File: \"{}\" )\n", msg, name),
        );
    }

    // Raven quirk preserved verbatim: `iXStart`/`iXStep` are computed from
    // the scan-line-order flag, but the RGB/greyscale decode loop below
    // writes through a sequentially-incrementing output pointer regardless
    // of their value — only the vertical (`iYStart`/`iYStep`) direction
    // actually changes the output layout (`tr_image.cpp:1577-1624`).
    let (y_start, y_step): (i32, i32) = match by_scan_line_order & 0x30 {
        0x20 | 0x30 => (0, 1),
        _ => (w_image_height - 1, -1),
    };

    let mut pic = vec![0u8; (w_image_width * w_image_height * 4).max(0) as usize];
    let mut in_pos = 18usize; // sizeof(TGAHeader_t)
    if by_id_field_length != 0 {
        in_pos += by_id_field_length as usize; // skip TARGA image comment
    }

    if by_image_type == 2 || by_image_type == 3 {
        // RGB or greyscale
        let mut y = y_start;
        for _ in 0..w_image_height {
            let mut out_pos = (y * w_image_width * 4) as usize;
            for _ in 0..w_image_width {
                let (red, green, blue, alpha) = match read_tga_pixel_rgb(
                    &buf,
                    &mut in_pos,
                    by_image_planes,
                ) {
                    Some(rgba) => rgba,
                    None => {
                        // Raven: `assert(0); // if we ever hit this, someone
                        // deleted a header check higher up`
                        com_error(
                            errorParm_t::ERR_DROP,
                            format!(
                                "LoadTGA: Image can only have 8, 24 or 32 planes for RGB/greyscale\n( File: \"{}\" )\n",
                                name
                            ),
                        );
                    }
                };
                pic[out_pos] = red;
                pic[out_pos + 1] = green;
                pic[out_pos + 2] = blue;
                pic[out_pos + 3] = alpha;
                out_pos += 4;
            }
            y += y_step;
        }
    } else if by_image_type == 10 {
        // RLE-RGB — "I've no idea if this stuff works, I normally reject RLE
        // targas, but this is from ID's code so maybe I should try and
        // support it..."
        let mut y = w_image_height - 1;
        let mut out_pos = (y * w_image_width * 4) as usize;
        let mut x = 0i32;
        'decode: loop {
            let packet_header = buf[in_pos];
            in_pos += 1;
            let packet_size = 1 + (packet_header & 0x7f);

            if packet_header & 0x80 != 0 {
                // run-length packet
                let (red, green, blue, alpha) = match read_tga_pixel_rle(
                    &buf,
                    &mut in_pos,
                    by_image_planes,
                ) {
                    Some(rgba) => rgba,
                    None => {
                        com_error(
                            errorParm_t::ERR_DROP,
                            format!("LoadTGA: RLE-RGB can only have 24 or 32 planes\n( File: \"{}\" )\n", name),
                        );
                    }
                };
                for _ in 0..packet_size {
                    pic[out_pos] = red;
                    pic[out_pos + 1] = green;
                    pic[out_pos + 2] = blue;
                    pic[out_pos + 3] = alpha;
                    out_pos += 4;
                    x += 1;
                    if x == w_image_width {
                        // run spans across rows
                        x = 0;
                        if y > 0 {
                            y -= 1;
                        } else {
                            break 'decode;
                        }
                        out_pos = (y * w_image_width * 4) as usize;
                    }
                }
            } else {
                // non run-length packet
                for _ in 0..packet_size {
                    let (red, green, blue, alpha) = match read_tga_pixel_rle(
                        &buf,
                        &mut in_pos,
                        by_image_planes,
                    ) {
                        Some(rgba) => rgba,
                        None => {
                            com_error(
                                errorParm_t::ERR_DROP,
                                format!("LoadTGA: RLE-RGB can only have 24 or 32 planes\n( File: \"{}\" )\n", name),
                            );
                        }
                    };
                    pic[out_pos] = red;
                    pic[out_pos + 1] = green;
                    pic[out_pos + 2] = blue;
                    pic[out_pos + 3] = alpha;
                    out_pos += 4;
                    x += 1;
                    if x == w_image_width {
                        // pixel packet run spans across rows
                        x = 0;
                        if y > 0 {
                            y -= 1;
                        } else {
                            break 'decode;
                        }
                        out_pos = (y * w_image_width * 4) as usize;
                    }
                }
            }
        }
    }

    Some((pic, w_image_width, w_image_height))
}

/// Raven `LoadJPG`.
///
/// The file-read CPU logic (`FS_FOpenFileRead`/`FS_Read`/`FS_FCloseFile`) is
/// ported; the libjpeg decompression pipeline itself is escalated — see the
/// `DEFERRED` block in the body.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1746-1915
pub fn LoadJPG(view: &mut EngineHostView, filename: &str) -> Option<(Vec<u8>, i32, i32)> {
    let mut handle: fileHandle_t = 0;
    let len = FS_FOpenFileRead(view, filename, &mut handle as *mut fileHandle_t, false);
    if handle == 0 {
        return None;
    }

    let mut fbuffer = vec![0u8; (len + 4096).max(0) as usize];
    FS_Read(view.common, fbuffer.as_mut_ptr() as *mut (), len, handle);
    FS_FCloseFile(view.common, handle);

    // DEFERRED: escalate — the libjpeg decompression pipeline
    // (jpeg_create_decompress/jpeg_stdio_src/jpeg_read_header/
    // jpeg_start_decompress/jpeg_read_scanlines/jpeg_finish_decompress/
    // jpeg_destroy_decompress) is vendored libjpeg with no Rust-crate seam
    // confirmed wired in this workspace (packet tr_image.wave0 "image-codec
    // seam" note: "escalate if the seam lacks a wrapper", never byte-port).
    // The file read above is the CPU logic ported around it; the
    // grayscale-expansion and alpha-channel-clear post-processing
    // (`:1862-1890`) would run once `out` is decoded, once the codec seam
    // lands.
    // Source: oracle/codemp/renderer/tr_image.cpp:1788-1899
    let _ = fbuffer;
    None
}

// DEFERRED-WHOLE: vendored libjpeg destination-manager glue — Raven's
// `init_destination`/`empty_output_buffer`/`jpeg_start_compress`/
// `jpeg_write_scanlines`/`term_destination`/`jpegDest`
// (oracle/codemp/renderer/tr_image.cpp:1935-2111) implement libjpeg's own
// `jpeg_destination_mgr` callback contract and internal compression-pipeline
// entry points (`cinfo->master`/`cinfo->main`/`cinfo->progress` walks) —
// vendored library internals, not Jedi Academy logic, with no Rust-crate
// jpeg-encode seam confirmed wired in this workspace (packet tr_image.wave0
// "image-codec seam" note: "escalate if the seam lacks a wrapper", never
// byte-port). No stub body is written per the GL-fn precedent ("if a fn is
// pure GL, leave one deferred stub comment block, no body") extended by
// analogy to this pure-vendored-codec case; `hackSize` (`term_destination`'s
// write target) has no consumer elsewhere in this packet and is left
// unhomed alongside these functions.
//
// - `init_destination` — oracle/codemp/renderer/tr_image.cpp:1935-1941
// - `empty_output_buffer` — oracle/codemp/renderer/tr_image.cpp:1967-1970
// - `jpeg_start_compress` — oracle/codemp/renderer/tr_image.cpp:1988-2009
// - `jpeg_write_scanlines` — oracle/codemp/renderer/tr_image.cpp:2027-2062
// - `term_destination` — oracle/codemp/renderer/tr_image.cpp:2075-2080
// - `jpegDest` — oracle/codemp/renderer/tr_image.cpp:2089-2111

/// Raven `R_InvertImage` — flips an image vertically in place. Raven's
/// `Z_Malloc`/`Z_Free` scratch pair becomes an owned local `Vec` (porting-
/// rules §C9).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2310-2331
pub fn R_InvertImage(data: &mut [u8], width: i32, height: i32, depth: i32) {
    let stride = (width * depth).max(0) as usize;
    let height = height.max(0) as usize;
    let mut saved = vec![0u8; height * stride];
    for y in 0..height {
        let src_start = (height - 1 - y) * stride;
        saved[y * stride..(y + 1) * stride].copy_from_slice(&data[src_start..src_start + stride]);
    }
    data[..saved.len()].copy_from_slice(&saved);
}

/// Raven `Lanczos3`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2352-2364
pub fn Lanczos3(t: f32) -> f32 {
    if t == 0.0 {
        return 1.0;
    }
    let t = t.abs();
    if t < 3.0 {
        const M_PI_OVER_3: f32 = PI / 3.0;
        return (t * PI).sin() * (t * M_PI_OVER_3).sin() / (t * PI * t * M_PI_OVER_3);
    }
    0.0
}

/// Raven `R_InitFogTable`. Registry-adjacent mutation of
/// `RenderAssets::function_tables` — goes through `Arc::make_mut` (A9),
/// matching every other `RenderAssets` write.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2633-2645
pub fn R_InitFogTable(sim: &mut RenderAssetsSim) {
    let exp = 0.5f32;
    let assets = Arc::make_mut(&mut sim.published);
    for i in 0..FOG_TABLE_SIZE {
        // C `pow` is `double pow(double, double)`; f64 intermediate per wave-0
        // ruling 12.
        let d = ((i as f32 / (FOG_TABLE_SIZE - 1) as f32) as f64).powf(exp as f64) as f32;
        assets.function_tables.fog_table[i] = d;
    }
}

/// Raven `R_FogFactor`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2656-2680
pub fn R_FogFactor(assets: &RenderAssets, s: f32, t: f32) -> f32 {
    let mut s = s - 1.0 / 512.0;
    if s < 0.0 {
        return 0.0;
    }
    if t < 1.0 / 32.0 {
        return 0.0;
    }
    if t < 31.0 / 32.0 {
        s *= (t - 1.0 / 32.0) / (30.0 / 32.0);
    }

    // we need to leave a lot of clamp range
    s *= 8.0;

    if s > 1.0 {
        s = 1.0;
    }

    assets.function_tables.fog_table[(s * (FOG_TABLE_SIZE - 1) as f32) as usize]
}

/// Raven `R_SetColorMappings`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2847-2919
pub fn R_SetColorMappings(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    glconfig: &GlConfig,
    state: &mut TrImageState,
) {
    // setup the overbright lighting
    let mut overbright_bits = view.common.cvar(cvars.r_overBrightBits).integer;

    if !glconfig.device_supports_gamma {
        overbright_bits = 0; // need hardware gamma for overbright
    }
    if !glconfig.is_fullscreen {
        overbright_bits = 0; // never overbright in windowed mode
    }
    if overbright_bits > 1 {
        overbright_bits = 1;
    }
    if overbright_bits < 0 {
        overbright_bits = 0;
    }

    // DEFERRED: `tr.overbrightBits`/`identityLight`/`identityLightByte` are
    // `trGlobals_t` frontend scratch -> `RenderWorld::frame: FrameState`
    // (`## State ownership` "tr frontend scratch/counters" row); the fields
    // are not yet landed on `FrameState` and this wave does not own that
    // struct (same gap `tr_init.rs`'s `GfxInfo_f` hit and deferred whole) —
    // `identityLight = 1/(1<<overbrightBits)`/`identityLightByte =
    // 255*identityLight` are skipped rather than computed with nowhere to
    // land (porting-rules §A2, no speculative behavior). `overbright_bits`
    // itself is still used locally below for the gamma-table shift, matching
    // Raven's own reuse of the local.
    // Source: oracle/codemp/renderer/tr_image.cpp:2873-2874

    if view.common.cvar(cvars.r_intensity).value < 1.0 {
        Cvar_Set(view, "r_intensity", "1");
    }

    if view.common.cvar(cvars.r_gamma).value < 0.5 {
        Cvar_Set(view, "r_gamma", "0.5");
    } else if view.common.cvar(cvars.r_gamma).value > 3.0 {
        Cvar_Set(view, "r_gamma", "3.0");
    }

    let g = view.common.cvar(cvars.r_gamma).value;
    let shift = overbright_bits;

    for i in 0..256i32 {
        let mut inf = if g == 1.0 {
            i
        } else {
            (255.0 * (i as f32 / 255.0).powf(1.0 / g) + 0.5) as i32
        };
        inf <<= shift;
        if inf < 0 {
            inf = 0;
        }
        if inf > 255 {
            inf = 255;
        }
        state.gamma_table[i as usize] = inf as u8;
    }

    for i in 0..256i32 {
        let mut j = (i as f32 * view.common.cvar(cvars.r_intensity).value) as i32;
        if j > 255 {
            j = 255;
        }
        state.intensity_table[i as usize] = j as u8;
    }

    if glconfig.device_supports_gamma {
        // DEFERRED: R4 — GLimp_SetGamma(s_gammatable, s_gammatable,
        // s_gammatable): resolved-call-surface marks `GLimp_SetGamma` NOT
        // RESOLVED in this workspace ("either an idiomatic rename this
        // generator has no verified alias for, or genuinely unported
        // client-side surface … confirm before use; escalate, never stub").
        // Source: oracle/codemp/renderer/tr_image.cpp:2915-2918
    }
}

// PORT-NOTE: `RE_SplitSkins`/`CommaParse`/`R_InitSkins`/`R_GetSkinByHandle`
// (oracle/codemp/renderer/tr_image.cpp:2980-3027, 3193-3290, 3324-3335,
// 3342-3347) are already ported — `crates/mp/renderer/src/tr_model/
// server_skins.rs`'s `re_split_skins`/`comma_parse`/
// `RenderModels::init_skins`/`RenderModels::skin_surfaces`, under the live
// `RenderModels.skins: Vec<ServerSkin>` skin registry (user ruling
// 2026-07-12 "server skins name-pool", amending the FROZEN `tr-model.md`).
// Reconciled, not re-transcribed here (preamble: "Never re-port an
// already-ported fn … reconcile, never fork a second port").

// ESCALATION: `R_SkinList_f` (oracle/codemp/renderer/tr_image.cpp:3355-3371)
// walks `tr.skins[0..tr.numSkins]` — this packet's STATE HOMES table routes
// that to `RenderAssets::skins: Arena<SkinAsset>` (the "SPLIT" row), but per
// the PORT-NOTE above the skin registry's LIVE implementation is
// `RenderModels.skins: Vec<ServerSkin>` (`tr_model/server_skins.rs`).
// Writing `R_SkinList_f` against `RenderAssets::skins` would fork a second,
// dead skin registry contradicting the live one; `RenderModels` has no
// public enumerate-all accessor today (only the per-handle
// `skin_surfaces`). Flagged as a wave-planning defect (this packet's STATE
// HOMES row is stale relative to the 2026-07-12 ruling) rather than invented
// around — porting-rules §A2/preamble: "A state home this packet marks
// UNMAPPED is an ESCALATION, never an invention."
// Source: oracle/codemp/renderer/tr_image.cpp:3355-3371

// ============================================================================
// wave 1
// ============================================================================

/// Raven `GL_TextureMode`.
///
/// ESCALATION: `modes[6]` (name + `GL_TEXTURE_MIN_FILTER`/`MAG_FILTER` enum
/// pair) is declared immediately above `tr_image.cpp:99`, outside this
/// packet's verbatim oracle slice — its GL enum values are not guessable per
/// wave law ("never guess a numeric constant… the RF_* bit-value guesses
/// were wave-0 BLOCKERs"). The function's very first statement depends on
/// it (`modes[i].name` compared against the console arg), so nothing above
/// the lookup is separable from it — left as a cited `todo!()` rather than
/// fabricated, matching the `Taiwanese_CollapseBig5Code` precedent
/// (`tr_font.rs`: transcribe everything computable, `todo!()` at the exact
/// blocking point). `gl_filter_min`/`gl_filter_max` land on `TrImageState`
/// (A13.3, named by this wave) once the table is available; the
/// `qglTexParameterf` mipmap-refresh loop past it is DEFERRED: R4 regardless
/// (fixed-function GL surface, DEC-37 A13.2).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:99-143
pub fn GL_TextureMode(
    _view: &mut EngineHostView,
    _cvars: &RendererCvars,
    _assets: &RenderAssets,
    _state: &mut TrImageState,
    _gpu: &mut GpuResources,
    _string: &str,
) {
    //TODO: Port modes
    // Source: oracle/codemp/renderer/tr_image.cpp (modes[6] filter-mode
    // table, declared immediately above :99 — not included in this packet's
    // verbatim slice)
    todo!(
        "Port GL_TextureMode's modes[6] table — oracle/codemp/renderer/tr_image.cpp:99-143 (modes[] declared above :99, not in this packet)"
    )
}

/// Raven `R_SumOfUsedImages`.
///
/// Out-param-free already; `tr.frameCount` reads `FrameState::frame_count`
/// per this packet's explicit SPLIT digest (registries stay `RenderAssets`,
/// the frame counter is render-thread scratch) — see the wave-1 PORT-NOTE
/// above `TrImageState` for the not-yet-landed field. `int total` truncates
/// on every `+=` in the `bUseFormat` branch in the oracle (int lvalue, float
/// rvalue); reproduced by casting back to `i32` each iteration rather than
/// accumulating in `f32` and casting once at the end.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:206-228
pub fn R_SumOfUsedImages(assets: &RenderAssets, frame: &FrameState, use_format: bool) -> f32 {
    let mut total: i32 = 0;

    let _ = R_Images_StartIteration(assets);
    let mut cursor = 0usize;
    while let Some(handle) = R_Images_GetNextIteration(assets, &mut cursor) {
        let image = match assets.images.get(handle) {
            Some(image) => image,
            None => continue,
        };
        // it has already been advanced for the next frame, so...
        if image.frame_used == frame.frame_count - 1 {
            if use_format {
                let byte_per_tex = R_BytesPerTex(&assets.glconfig, image.internal_format);
                total = (total as f32 + byte_per_tex * (image.width * image.height) as f32) as i32;
            } else {
                total += image.width * image.height;
            }
        }
    }

    total as f32
}

/// Raven `R_ImageList_f`.
///
/// ESCALATION (partial): `image->wrapClampMode`'s `GL_REPEAT`/`GL_CLAMP`/
/// `GL_CLAMP_TO_EDGE` named branches are not resolvable — their values are
/// not in this packet or the target file (the `internalFormat` switch just
/// above reuses `GL_RGBA4`/`GL_RGB5`/`GL_RGBA8`/`GL_RGB8`/`GL_RGB4_S3TC`/
/// `GL_COMPRESSED_RGB_S3TC_DXT1_EXT`/`GL_COMPRESSED_RGBA_S3TC_DXT5_EXT`,
/// already landed in this file by wave 0, so those ARE reused here — but no
/// wrap-mode constant is present anywhere this wave can read). The wrap
/// column falls through to the oracle's own numeric default-case format
/// (`"%4i ", image->wrapClampMode`) for every mode, not just unrecognized
/// ones, until the three named constants land.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:235-312
pub fn R_ImageList_f(view: &mut EngineHostView, assets: &RenderAssets) {
    const YESNO: [&str; 2] = ["no ", "yes"];

    com_printf(
        view.common,
        "\n      -w-- -h-- -mm- -if-- wrap --name-------\n",
    );

    let mut texels: i32 = 0;
    let mut tex_bytes: f32 = 0.0;
    let mut i: i32 = 0;

    let num_images = R_Images_StartIteration(assets);
    let mut cursor = 0usize;
    while let Some(handle) = R_Images_GetNextIteration(assets, &mut cursor) {
        let image = match assets.images.get(handle) {
            Some(image) => image,
            None => continue,
        };
        texels += image.width * image.height;
        tex_bytes += (image.width * image.height) as f32
            * R_BytesPerTex(&assets.glconfig, image.internal_format);
        com_printf(
            view.common,
            &format!(
                "{:4}: {:4} {:4}  {} ",
                i, image.width, image.height, YESNO[image.mipmap as usize]
            ),
        );
        match image.internal_format {
            1 => com_printf(view.common, "I    "),
            2 => com_printf(view.common, "IA   "),
            3 => com_printf(view.common, "RGB  "),
            4 => com_printf(view.common, "RGBA "),
            GL_RGBA8 => com_printf(view.common, "RGBA8"),
            GL_RGB8 => com_printf(view.common, "RGB8"),
            GL_RGB4_S3TC => com_printf(view.common, "S3TC "),
            GL_COMPRESSED_RGB_S3TC_DXT1_EXT => com_printf(view.common, "DXT1 "),
            GL_COMPRESSED_RGBA_S3TC_DXT5_EXT => com_printf(view.common, "DXT5 "),
            GL_RGBA4 => com_printf(view.common, "RGBA4"),
            GL_RGB5 => com_printf(view.common, "RGB5 "),
            _ => com_printf(view.common, "???? "),
        }

        //TODO: Port GL_REPEAT
        //TODO: Port GL_CLAMP
        //TODO: Port GL_CLAMP_TO_EDGE
        // Source: oracle/codemp/renderer/tr_image.cpp:289-302 (wrap-mode GL
        // enum values not resolvable without oracle access; wave law forbids
        // guessing numeric constants — falls through to the default-case
        // numeric format for every mode)
        com_printf(view.common, &format!("{:4} ", image.wrap_clamp_mode));

        com_printf(view.common, &format!("{}\n", image.img_name));
        i += 1;
    }
    com_printf(view.common, " ---------\n");
    com_printf(
        view.common,
        "      -w-- -h-- -mm- -if- wrap --name-------\n",
    );
    com_printf(
        view.common,
        &format!(" {} total texels (not including mipmaps)\n", texels),
    );
    com_printf(
        view.common,
        &format!(
            " {:.2}MB total texture mem (not including mipmaps)\n",
            tex_bytes / 1048576.0
        ),
    );
    com_printf(view.common, &format!(" {} total images\n\n", num_images));
}

/// Raven `R_MipMap`.
///
/// `in`/`out` alias the same buffer in the oracle — `out` always trails `in`
/// (it advances 4 bytes per source-pixel-pair `in` advances 8, starting
/// equal), so it never overtakes an unread source pixel; represented here as
/// two index cursors over one mutable slice rather than two raw pointers
/// (interior-safety law). The `(unsigned *)in` reinterpret-cast on the
/// `!r_simpleMipMaps` fast path becomes an explicit little-endian
/// byte<->u32 round-trip around the already-ported `R_MipMap2` (matching
/// `R_MipMap2`'s own `to_le_bytes`/`from_le_bytes` precedent), since Rust has
/// no safe reinterpret between differently-typed slices.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:439-477
pub fn R_MipMap(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    data: &mut [u8],
    width: i32,
    height: i32,
) {
    if view.common.cvar(cvars.r_simpleMipMaps).integer == 0 {
        let pixel_count = (width * height).max(0) as usize;
        let mut px: Vec<u32> = data[..pixel_count * 4]
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        R_MipMap2(&mut px, width, height);
        let out_count = (((width >> 1).max(0)) * ((height >> 1).max(0))) as usize;
        for (i, value) in px[..out_count].iter().enumerate() {
            data[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        return;
    }

    if width == 1 && height == 1 {
        return;
    }

    let row = (width * 4) as usize;
    let mut out_pos = 0usize;
    let mut in_pos = 0usize;
    let out_width = width >> 1;
    let out_height = height >> 1;

    if out_width == 0 || out_height == 0 {
        let n = out_width + out_height; // get largest
        for _ in 0..n {
            let mut out_pixel = [0u8; 4];
            for k in 0..4usize {
                out_pixel[k] = ((data[in_pos + k] as u16 + data[in_pos + 4 + k] as u16) >> 1) as u8;
            }
            data[out_pos..out_pos + 4].copy_from_slice(&out_pixel);
            out_pos += 4;
            in_pos += 8;
        }
        return;
    }

    for _ in 0..out_height {
        for _ in 0..out_width {
            let mut out_pixel = [0u8; 4];
            for k in 0..4usize {
                let sum = data[in_pos + k] as u16
                    + data[in_pos + 4 + k] as u16
                    + data[in_pos + row + k] as u16
                    + data[in_pos + row + 4 + k] as u16;
                out_pixel[k] = (sum >> 2) as u8;
            }
            data[out_pos..out_pos + 4].copy_from_slice(&out_pixel);
            out_pos += 4;
            in_pos += 8;
        }
        in_pos += row;
    }
}

/// Raven `GL_ResetBinds`.
///
/// DEFERRED: R4 — entirely fixed-function GL: `memset(glState
/// .currenttextures, …)` plus `qglBindTexture`/`qglActiveTextureARB`-gated
/// calls through `GL_SelectTexture` (`tr_backend.rs`, already DEFERRED: R4).
/// No CPU-only logic survives the GL binding cache (DEC-37 A13.2), matching
/// `GL_Bind`'s identical empty-body precedent in the same crate.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:982-999
pub fn GL_ResetBinds(_gpu: &mut GpuResources) {
    // DEFERRED: R4 — GL_ResetBinds body (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_image.cpp:982-999
}

/// Raven `R_Images_DeleteImage`.
///
/// `image_t *pImage`'s std::map find-by-name-then-erase collapses to a
/// direct arena lookup by handle (§B5 index-not-pointer) — the name is read
/// back off the found `ImageAsset` (`img_name`, wave-1 PORT-NOTE field) so
/// `image_names`' matching entry can be erased alongside the arena slot, the
/// two registries the oracle's single `AllocatedImages` map served at once
/// (`R2-D3`/`R2-D4`). `assert(0)` (not-found path) becomes `debug_assert!`
/// (§19 — no defined oracle behavior past an assert in a release build).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1035-1049
pub fn R_Images_DeleteImage(sim: &mut RenderAssetsSim, handle: ImageHandle) {
    let name = sim
        .published
        .images
        .get(handle)
        .map(|image| image.img_name.clone());

    match name {
        Some(name) => {
            R_Images_DeleteImageContents(sim, handle);
            Arc::make_mut(&mut sim.published).image_names.remove(&name);
        }
        None => {
            debug_assert!(false, "R_Images_DeleteImage: handle not found in registry");
        }
    }
}

/// Raven `R_Images_Clear`.
///
/// The std::map iteration is collected into an owned `Vec<ImageHandle>`
/// first, then walked to call the already-ported `R_Images_DeleteImageContents`
/// (which itself empties the arena slot, wave 0) — avoids mutating
/// `RenderAssets::images` while a live borrow of it is mid-iteration.
/// `giTextureBindNum` lands on `TrImageState` (A13.3, named by this wave).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1053-1066
pub fn R_Images_Clear(sim: &mut RenderAssetsSim, state: &mut TrImageState) {
    let _ = R_Images_StartIteration(&sim.published);
    let mut cursor = 0usize;
    let mut handles = Vec::new();
    while let Some(handle) = R_Images_GetNextIteration(&sim.published, &mut cursor) {
        handles.push(handle);
    }
    for handle in handles {
        R_Images_DeleteImageContents(sim, handle);
    }

    Arc::make_mut(&mut sim.published).image_names.clear();

    state.gi_texture_bind_num = 1024;
}

/// Raven `RE_RegisterImages_Info_f`.
///
/// `RE_RegisterMedia_GetLevel()` reconciles to the already-live
/// `RenderModels::media_get_level` (`tr_model/cached_model_binary.rs`,
/// per the crate's `tr_model` PORT-NOTE above — reconciled, not re-ported).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1069-1087
pub fn RE_RegisterImages_Info_f(
    view: &mut EngineHostView,
    assets: &RenderAssets,
    models: &RenderModels,
) {
    let mut i_image: i32 = 0;
    let mut i_texels: i32 = 0;

    let num_images = R_Images_StartIteration(assets);
    let mut cursor = 0usize;
    while let Some(handle) = R_Images_GetNextIteration(assets, &mut cursor) {
        let image = match assets.images.get(handle) {
            Some(image) => image,
            None => continue,
        };
        com_printf(
            view.common,
            &format!(
                "{}: ({:4}x{:4}y) \"{}\"",
                i_image, image.width, image.height, image.img_name
            ),
        );
        Com_DPrintf(
            view.common,
            &format!(
                "{}, levused {}",
                S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII"),
                image.last_level_used_on
            ),
        );
        com_printf(view.common, "\n");

        i_texels += image.width * image.height;
        i_image += 1;
    }
    com_printf(
        view.common,
        &format!(
            "{} Images. {} ({:.2}MB) texels total, (not including mipmaps)\n",
            num_images,
            i_texels,
            i_texels as f32 / 1024.0 / 1024.0
        ),
    );
    Com_DPrintf(
        view.common,
        &format!(
            "{}RE_RegisterMedia_GetLevel(): {}",
            S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII"),
            models.media_get_level()
        ),
    );
}

/// Raven `R_FindImageFile_NoLoad`.
///
/// `const char *name`'s NULL check becomes `Option<&str>` (idiomatic
/// nullable-pointer translation, no Rust `&str` can be null); the std::map
/// `find` collapses to `RenderAssets::image_names`' `HashMap` lookup
/// (`R2-D3`/`R2-D4`); the `iLastLevelUsedOn` write goes through
/// `Arc::make_mut` (A9), matching every other `RenderAssets` mutation.
/// `allowTC` is read nowhere in the oracle body (dead parameter even in
/// retail) — dropped rather than threaded through unused (porting-rules
/// data-flow principle; the oracle itself never reads it).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1157-1193
pub fn R_FindImageFile_NoLoad(
    sim: &mut RenderAssetsSim,
    view: &mut EngineHostView,
    models: &RenderModels,
    name: Option<&str>,
    mipmap: bool,
    allow_picmip: bool,
    gl_wrap_clamp_mode: i32,
) -> Option<ImageHandle> {
    let name = name?;
    let p_name = GenerateImageMappingName(name);

    let handle = *sim.published.image_names.get(&p_name)?;

    if p_name != "*white" {
        if let Some(image) = sim.published.images.get(handle) {
            if image.mipmap != mipmap {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: reused image {} with mixed mipmap parm\n",
                        S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
                        p_name
                    ),
                );
            }
            if image.allow_picmip != allow_picmip {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: reused image {} with mixed allowPicmip parm\n",
                        S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
                        p_name
                    ),
                );
            }
            if image.wrap_clamp_mode != gl_wrap_clamp_mode {
                com_printf(
                    view.common,
                    &format!(
                        "{}WARNING: reused image {} with mixed glWrapClampMode parm\n",
                        S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
                        p_name
                    ),
                );
            }
        }
    }

    Arc::make_mut(&mut sim.published)
        .images
        .get_mut(handle)?
        .last_level_used_on = models.media_get_level();

    Some(handle)
}

// DEFERRED-WHOLE: `SaveJPG` — entirely a vendored-libjpeg compression
// pipeline (`jpeg_std_error`/`jpeg_create_compress`/`jpeg_set_defaults`/
// `jpeg_set_quality`/`jpeg_finish_compress`/`jpeg_destroy_compress`, plus the
// already-deferred `jpegDest`/`jpeg_start_compress`/`jpeg_write_scanlines`
// glue directly above) with no Rust-crate jpeg-encode seam wired in this
// workspace (`Cargo.toml` carries no image/jpeg dependency) — this packet's
// own threading digest for `SaveJPG` says exactly this: "vendored
// libjpeg/png; a Rust-crate seam, never byte-ported (escalate if the seam
// lacks a wrapper)", extending wave 0's jpegDest-family precedent (no stub
// body written, this comment block only) to the caller. `hackSize`
// (`term_destination`'s write target, the `FS_WriteFile` size argument) has
// no consumer elsewhere in this packet and stays unhomed alongside this
// glue.
//
// Source: oracle/codemp/renderer/tr_image.cpp:2113-2216

/// Raven `COM_DefaultExtension` on owned strings: append `extension` unless
/// the basename (scan back to the last `/`) already carries a `.`; result is
/// bounded at `MAX_QPATH - 1` like the original's `Com_sprintf( path,
/// maxSize, … )`.
/// Source: `oracle/codemp/game/q_shared.c:112-131`
fn com_default_extension(path: &str, extension: &str) -> String {
    // `while (*src != '/' && src != path)` tests the loop condition *before*
    // the body, so index 0 is never examined for `.` or `/`: a leading-dot
    // path still gets the extension appended. Scanning `[1..]` reproduces
    // that; an empty or 1-byte path scans nothing, as in C.
    if path.len() > 1 {
        for &b in path.as_bytes()[1..].iter().rev() {
            if b == b'/' {
                break;
            }
            if b == b'.' {
                return path.to_string();
            }
        }
    }
    let mut out = format!("{path}{extension}");
    let bound = MAX_QPATH as usize - 1;
    if out.len() > bound {
        let mut cut = bound;
        while !out.is_char_boundary(cut) {
            cut -= 1;
        }
        out.truncate(cut);
    }
    out
}

/// Raven `R_LoadImage`.
///
/// Out-params (`pic`/`width`/`height`) collapse to a return value (§C7).
/// `*format` is unconditionally `GL_RGBA` on every reachable path (assigned
/// once at entry, never reassigned) — its numeric value is not in this
/// packet or the target file and is left out of the return rather than
/// guessed (wave law: never guess a numeric constant); a reviewer restoring
/// it needs only the one always-true assignment, not new control flow.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2228-2256
pub fn R_LoadImage(view: &mut EngineHostView, shortname: &str) -> Option<(Vec<u8>, i32, i32)> {
    //TODO: Port GL_RGBA
    // Source: oracle/codemp/renderer/tr_image.cpp:2235 (`*format = GL_RGBA;`
    // — value not in packet/target file, dropped from the return per the
    // doc comment above)

    let name = com_default_extension(&COM_StripExtension(shortname), ".jpg");
    if let Some(result) = LoadJPG(view, &name) {
        return Some(result);
    }

    // DEFERRED: `LoadPNG32` — vendored libpng, no Rust-crate seam wired in
    // this workspace (`Cargo.toml` carries no png/image dependency);
    // escalate rather than byte-port, matching `LoadJPG`'s identical
    // codec-seam precedent above in this same file. The default-extension
    // computation is still performed for parity of the attempted-path
    // sequence; the decode itself is a no-op (always "no pic").
    // Source: oracle/codemp/renderer/tr_image.cpp:2243-2248
    let _name_png = com_default_extension(&COM_StripExtension(shortname), ".png");

    let name = com_default_extension(&COM_StripExtension(shortname), ".tga");
    if let Some(result) = LoadTGA(view, &name) {
        return Some(result);
    }

    None
}

/// Raven `R_LoadDataImage`.
///
/// Out-params collapse to a return value (§C7); both length guards
/// (`len >= MAX_QPATH`, `len < 5`) transcribed faithfully. `LoadPNG8` is the
/// same unresolved vendored-libpng codec seam as `R_LoadImage`'s `LoadPNG32`
/// — DEFERRED, matching precedent.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2259-2306
pub fn R_LoadDataImage(view: &mut EngineHostView, name: &str) -> Option<(Vec<u8>, i32, i32)> {
    let len = name.len();
    if len >= MAX_QPATH as usize {
        return None;
    }
    if len < 5 {
        return None;
    }

    // DEFERRED: `LoadPNG8` — vendored libpng, no Rust-crate seam wired (see
    // `R_LoadImage`'s identical PNG note above).
    // Source: oracle/codemp/renderer/tr_image.cpp:2281-2282
    let _work_png = com_default_extension(name, ".png");

    let work = com_default_extension(name, ".jpg");
    if let Some(result) = LoadJPG(view, &work) {
        return Some(result);
    }

    let work = com_default_extension(name, ".tga");
    if let Some(result) = LoadTGA(view, &work) {
        return Some(result);
    }

    com_printf(
        view.common,
        &format!("Couldn't read {} -- dataimage load failed\n", name),
    );
    None
}

/// Raven `R_Resample`.
///
/// `contrib_list_t`/`contrib_t`'s `Z_Malloc`/`Z_Free` scratch pairs become
/// owned local `Vec<Vec<Contrib>>` (porting-rules §C9: manual alloc/free ->
/// ownership) — one inner `Vec` per output row/column index, replacing the
/// oracle's fixed `num`-sized `Z_Malloc` block per index. `LANCZOS3` reuses
/// the value already present in this file (see the `const` above).
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2366-2520
pub fn R_Resample(
    source: &[u8],
    swidth: i32,
    sheight: i32,
    dest: &mut [u8],
    dwidth: i32,
    dheight: i32,
    components: i32,
) {
    struct Contrib {
        pixel: i32,
        weight: f32,
    }

    fn build_contributors(dcount: i32, scount: i32) -> Vec<Vec<Contrib>> {
        let scale_axis = dcount as f32 / scount as f32;
        let (window, scale) = if scale_axis < 1.0 {
            ((LANCZOS3 / scale_axis).ceil(), scale_axis)
        } else {
            (LANCZOS3, 1.0)
        };

        let mut contributors = Vec::with_capacity(dcount.max(0) as usize);
        for i in 0..dcount {
            let center = i as f32 / scale_axis;
            let left = (center - window).ceil() as i32;
            let right = (center + window).floor() as i32;

            let mut contrib = Vec::new();
            for j in left..=right {
                let weight = Lanczos3((center - j as f32) * scale) * scale;
                let pixel = if j < 0 {
                    -j
                } else if j >= scount {
                    (scount - j) + scount - 1
                } else {
                    j
                };
                contrib.push(Contrib { pixel, weight });
            }
            contributors.push(contrib);
        }
        contributors
    }

    // Pre-calculate filter contributions for rows, apply horizontally
    // (source -> work).
    let mut work = vec![0u8; (dwidth * sheight * components).max(0) as usize];
    let row_contributors = build_contributors(dwidth, swidth);

    for k in 0..sheight {
        let raster = &source[(k * swidth * components).max(0) as usize..];
        for i in 0..dwidth {
            for l in 0..components {
                let mut weight = 0.0f32;
                for c in &row_contributors[i as usize] {
                    weight += raster[((c.pixel * components) + l) as usize] as f32 * c.weight;
                }
                let pixel = Com_Clamp(0.0, 255.0, weight) as u8;
                work[((k * dwidth * components) + (i * components) + l) as usize] = pixel;
            }
        }
    }

    // Columns: pre-calculate filter contributions, apply vertically
    // (work -> dest).
    let col_contributors = build_contributors(dheight, sheight);

    for k in 0..dwidth {
        for l in 0..components {
            for i in 0..dheight {
                let mut weight = 0.0f32;
                for c in &col_contributors[i as usize] {
                    weight += work
                        [((c.pixel * dwidth * components) + (k * components) + l) as usize]
                        as f32
                        * c.weight;
                }
                let pixel = Com_Clamp(0.0, 255.0, weight) as u8;
                dest[((i * dwidth * components) + (k * components) + l) as usize] = pixel;
            }
        }
    }
}

// ============================================================================
// wave 2
// ============================================================================

/// Raven `Upload32`.
///
/// ESCALATION: the entire function body lives inside `if (format == GL_RGBA)`
/// (the oracle's `else` arm is empty — `tr_image.cpp:764-766`) — `GL_RGBA`'s
/// numeric value is not in this packet or the target file. This exact
/// constant is already marked the same way at this file's `R_LoadImage`
/// (wave 0, `//TODO: Port GL_RGBA`); reused here rather than guessing a
/// second, possibly-inconsistent value. Nothing past that first comparison
/// is separable from it, so this is `todo!()`'d at the exact blocking point,
/// matching the `GL_TextureMode` precedent directly above (wave 1, same
/// file: "transcribe everything computable, `todo!()` at the exact blocking
/// point"). Once `GL_RGBA` lands, the body also needs: `TrImageState`
/// extended with the `mipBlendColors[16][4]` blend table (STATE HOMES row
/// "NAMED BY THIS WAVE", DEC-37 A13.3 — not added here since nothing
/// consumes it while the function stays blocked, porting-rules §A2 no
/// speculative behavior); `RenderAssets::glconfig.max_texture_size`/
/// `.texture_compression` (already landed fields, `R_BytesPerTex`/
/// `tr_init.rs` precedent) for the picmip-clamp and format-select logic; and
/// the R4 `qglTexImage2D`/`qglTexParameterf` GL entry points (DEC-37 A13.2,
/// unhomed) for the upload/mipmap-refresh calls this fn's own threading
/// digest already flags DEFERRED.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:584-786
pub fn Upload32(
    _view: &mut EngineHostView,
    _cvars: &RendererCvars,
    _assets: &RenderAssets,
    _state: &TrImageState,
    _gpu: &mut GpuResources,
    _data: &mut [u32],
    format: i32,
    _mipmap: bool,
    _picmip: bool,
    _is_lightmap: bool,
    _allow_tc: bool,
    _upload_width: u16,
    _upload_height: u16,
    _b_rectangle: bool,
) -> (i32, u16, u16) {
    let _ = format;
    //TODO: Port GL_RGBA
    // Source: oracle/codemp/renderer/tr_image.cpp:599 (`if (format == GL_RGBA)`
    // — GL_RGBA's numeric value is not in this packet or the target file; see
    // the doc comment above)
    todo!(
        "Port Upload32 — blocked on unresolved GL_RGBA format constant, oracle/codemp/renderer/tr_image.cpp:584-786"
    )
}

/// Raven `R_Images_DeleteLightMaps`.
///
/// The std::map iteration + erase-on-match collapses to the same
/// collect-then-delete shape as `R_Images_Clear` (wave 1), avoiding a live
/// borrow of `RenderAssets::images` across arena mutation: images whose name
/// matches the oracle's loose `imgName[0] == '*' && strstr(imgName,
/// "lightmap")` check are collected first, then each is deleted through the
/// already-ported `R_Images_DeleteImageContents` (empties the arena slot)
/// with its `image_names` entry removed alongside — `R_Images_
/// DeleteImageContents` alone only touches the arena, matching `R_Images_
/// DeleteImage`'s (wave 1) identical two-registry cleanup (`R2-D3`/`R2-D4`).
/// The oracle's `bEraseOccured` erase-in-place iterator dance has no R2
/// carrier of its own (see `R_Images_StartIteration`'s PORT-NOTE) — the
/// collect-first shape sidesteps it entirely.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1006-1031
pub fn R_Images_DeleteLightMaps(sim: &mut RenderAssetsSim, gpu: &mut GpuResources) {
    let _ = R_Images_StartIteration(&sim.published);
    let mut cursor = 0usize;
    let mut targets = Vec::new();
    while let Some(handle) = R_Images_GetNextIteration(&sim.published, &mut cursor) {
        if let Some(image) = sim.published.images.get(handle) {
            // loose check, but should be ok
            if image.img_name.starts_with('*') && image.img_name.contains("lightmap") {
                targets.push((handle, image.img_name.clone()));
            }
        }
    }

    for (handle, name) in targets {
        R_Images_DeleteImageContents(sim, handle);
        Arc::make_mut(&mut sim.published).image_names.remove(&name);
    }

    GL_ResetBinds(gpu);
}

/// Raven `RE_RegisterImages_LevelLoadEnd`.
///
/// `qboolean` return preserved as `bool` (§C7, already out-param-free in the
/// oracle). Same collect-then-delete arena-mutation shape as `R_Images_
/// DeleteLightMaps` above — `R_Images_DeleteImageContents` empties the arena
/// slot, the matching `image_names` entry is removed alongside (`R_Images_
/// DeleteImage`'s two-registry cleanup precedent, `R2-D3`/`R2-D4`).
/// `RE_RegisterMedia_GetLevel()` reconciles to the already-live
/// `RenderModels::media_get_level` (`tr_model/cached_model_binary.rs`, per
/// this file's `tr_model` PORT-NOTE above — reconciled, not re-ported). The
/// commented-out `MAX_DRAWIMAGES` warning block (`tr_image.cpp:1134-1141`,
/// already dead in the oracle) is not transcribed.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1097-1148
pub fn RE_RegisterImages_LevelLoadEnd(
    sim: &mut RenderAssetsSim,
    gpu: &mut GpuResources,
    view: &mut EngineHostView,
    models: &RenderModels,
) -> bool {
    Com_DPrintf(
        view.common,
        &format!(
            "{}RE_RegisterImages_LevelLoadEnd():\n",
            S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII")
        ),
    );

    let mut erase_occured = false;

    let _ = R_Images_StartIteration(&sim.published);
    let mut cursor = 0usize;
    let mut targets = Vec::new();
    while let Some(handle) = R_Images_GetNextIteration(&sim.published, &mut cursor) {
        if let Some(image) = sim.published.images.get(handle) {
            // don't un-register system shaders (*fog, *dlight, *white,
            // *default), but DO de-register lightmaps
            // ("*<mapname>/lightmap%d")
            if !image.img_name.starts_with('*') || image.img_name.contains('/') {
                // image used on this level?
                if image.last_level_used_on != models.media_get_level() {
                    // nope, so dump it...
                    Com_DPrintf(
                        view.common,
                        &format!(
                            "{}Dumping image \"{}\"\n",
                            S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII"),
                            image.img_name
                        ),
                    );
                    targets.push((handle, image.img_name.clone()));
                }
            }
        }
    }

    for (handle, name) in targets {
        R_Images_DeleteImageContents(sim, handle);
        Arc::make_mut(&mut sim.published).image_names.remove(&name);
        erase_occured = true;
    }

    Com_DPrintf(
        view.common,
        &format!(
            "{}RE_RegisterImages_LevelLoadEnd(): Ok\n",
            S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII")
        ),
    );

    GL_ResetBinds(gpu);

    erase_occured
}

/// Raven `R_DeleteTextures`.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:2942-2946
pub fn R_DeleteTextures(
    sim: &mut RenderAssetsSim,
    state: &mut TrImageState,
    gpu: &mut GpuResources,
) {
    R_Images_Clear(sim, state);
    GL_ResetBinds(gpu);
}

// ============================================================================
// wave 3
// ============================================================================

/// Raven `R_CreateImage`.
///
/// Registration path per this packet's STATE HOMES table (`R2-D3`/`R2-D4`):
/// the oracle's `Z_Malloc`-then-`AllocatedImages[name] = image` sequence
/// becomes one `Arena<ImageAsset>::insert` + `image_names` map insert at the
/// end, through `Arc::make_mut(&mut sim.published)` (A9), matching every
/// other `RenderAssets` registry write in this file. The image arena is
/// unbounded (A5) so `insert` never fails — this fn's Rust return type is a
/// bare `ImageHandle`, not `Option`, matching the oracle's "always succeeds"
/// contract (`Z_Malloc` panics rather than returning NULL on OOM, oracle-side
/// only).
///
/// Two named constants block real CPU logic and are DEFERRED rather than
/// guessed (never-guess rule): `GL_CLAMP`/`GL_CLAMP_TO_EDGE` gate the
/// `glWrapClampMode` clamp-to-edge substitution at `:1214-1216` and again
/// (GL-call-only, see below) at `:1264`. Both are absent from this packet's
/// FILE-SCOPE CONSTANTS section and this fn's own oracle slice, and are the
/// same unresolved wrap-mode family `GL_TextureMode`/`R_ImageList_f` (wave 1,
/// same file) already flagged as unresolvable — `gl_wrap_clamp_mode` is
/// threaded through unmodified rather than fabricating either enum value.
/// `glConfig.clampToEdgeAvailable` — the substitution's other conjunct at
/// `:1214` — is a co-blocker in its own right: it has no R3 home (`glConfig`
/// is R4 GL-capability state, STATE HOMES table), so even with both enum
/// values in hand the `if` could not be evaluated.
/// This propagates to both the `R_FindImageFile_NoLoad` lookup key/warn
/// comparisons below and the final stored `ImageAsset::wrap_clamp_mode`
/// field — the oracle would apply the substitution before both uses.
///
/// The entire `qglActiveTextureARB`-gated `GL_SelectTexture`/`GL_Bind`/
/// `bRectangle` GL-target-selection block (`:1254-1270`) is DEFERRED: R4 —
/// `qglActiveTextureARB`'s presence has no R3 home (STATE HOMES table) so
/// even the guarding `if` cannot be evaluated, and `image->texnum` (the GL
/// bind target both branches address) has no R3 home either (`ImageAsset`'s
/// own doc comment). It has no effect on any `ImageAsset` field: the stored
/// `wrapClampMode` is already set from the un-substituted
/// `gl_wrap_clamp_mode` before this block runs in the oracle, and the
/// block's own `glWrapClampMode = GL_CLAMP_TO_EDGE` reassignment
/// (`:1264`) only feeds the deferred `qglTexParameterf` calls past it, never
/// the stored field.
///
/// Source: oracle/codemp/renderer/tr_image.cpp:1204-1298
pub fn R_CreateImage(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    sim: &mut RenderAssetsSim,
    models: &RenderModels,
    state: &mut TrImageState,
    gpu: &mut GpuResources,
    name: &str,
    pic: &[u8],
    width: i32,
    height: i32,
    format: i32,
    mipmap: bool,
    allow_picmip: bool,
    allow_tc: bool,
    gl_wrap_clamp_mode: i32,
    b_rectangle: bool,
) -> ImageHandle {
    if name.len() >= MAX_QPATH as usize {
        com_error(
            errorParm_t::ERR_DROP,
            format!("R_CreateImage: \"{}\" is too long\n", name),
        );
    }

    // DEFERRED: `glWrapClampMode == GL_CLAMP -> GL_CLAMP_TO_EDGE`
    // substitution not applied — see the doc comment above.
    // Source: oracle/codemp/renderer/tr_image.cpp:1214-1216

    // Raven: only images whose name starts with '*' and whose last path
    // component is "lightmapNNN" are lightmaps.
    let mut is_lightmap = false;
    if name.starts_with('*') {
        if let Some(slash) = name.rfind('/') {
            if name[slash + 1..].starts_with("lightmap") {
                is_lightmap = true;
            }
        }
    }

    if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
        com_error(
            errorParm_t::ERR_FATAL,
            format!(
                "R_CreateImage: {} dimensions ({} x {}) not power of 2!\n",
                name, width, height
            ),
        );
    }

    if let Some(handle) = R_FindImageFile_NoLoad(
        sim,
        view,
        models,
        Some(name),
        mipmap,
        allow_picmip,
        gl_wrap_clamp_mode,
    ) {
        return handle;
    }

    // Raven: `image = (image_t*) Z_Malloc(sizeof(image_t), TAG_IMAGE_T,
    // qtrue);` — replaced by the `Arena<ImageAsset>::insert` at the end of
    // this fn (owned Rust storage, porting-rules §C9), matching
    // `R_Images_DeleteImageContents`'s identical `Z_Free` reconciliation
    // above. `bZeroit = qtrue` zero-initializes every field this fn doesn't
    // explicitly set below (e.g. `frameUsed`), matching `ImageAsset::default`.
    // Source: oracle/codemp/renderer/tr_image.cpp:1236-1237

    // Raven: `image->texnum = 1024 + giTextureBindNum++;` — texnum's target
    // field has no R3 home (`ImageAsset`'s own doc comment: "lands with the
    // R4 GPU wave"), but the increment side effect still runs: "the ++ is of
    // course staggeringly important..." (Raven comment) — later images
    // depend on it having advanced.
    state.gi_texture_bind_num += 1;

    let last_level_used_on = models.media_get_level();

    // DEFERRED: R4 — the `qglActiveTextureARB`-gated GL-target-selection
    // block; see the doc comment above.
    // Source: oracle/codemp/renderer/tr_image.cpp:1254-1270

    // Raven: `Upload32((unsigned *)pic, format, …)` — the `(unsigned *)pic`
    // reinterpret-cast becomes an owned little-endian byte->u32 collection
    // (interior-safety law forbids a raw reinterpret cast), matching
    // `R_MipMap`'s identical `to_le_bytes`/`from_le_bytes` precedent in this
    // same file.
    //
    // PORT-NOTE: the owned copy also drops Raven's in-place write-back —
    // `Upload32` scales/mips through the caller's `pic` buffer — but no
    // oracle caller reads `pic` after the call, so the divergence is
    // unobservable.
    let pixel_count = (width * height).max(0) as usize;
    let mut data: Vec<u32> = pic[..pixel_count * 4]
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    let (internal_format, out_width, out_height) = Upload32(
        view,
        cvars,
        &*sim.published,
        &*state,
        gpu,
        &mut data,
        format,
        mipmap,
        allow_picmip,
        is_lightmap,
        allow_tc,
        width as u16,
        height as u16,
        b_rectangle,
    );

    // DEFERRED: R4 — `qglTexParameterf(uiTarget, GL_TEXTURE_WRAP_S/T, …)` x2,
    // `qglBindTexture(uiTarget, 0)`, and `glState.currenttextures
    // [glState.currenttmu] = 0` (`GpuResources::gl_state` is a named
    // placeholder with no `currenttextures`/`currenttmu` fields yet).
    // Source: oracle/codemp/renderer/tr_image.cpp:1281-1285

    // Raven: `Q_strncpyz(image->imgName, name, …)` at `:1248` is overwritten
    // by this second `Q_strncpyz(image->imgName, psNewName, …)` before
    // `image` is read again — only the final mapped name is transcribed.
    let p_name = GenerateImageMappingName(name);

    let assets = Arc::make_mut(&mut sim.published);
    let handle = assets.images.insert(ImageAsset {
        img_name: p_name.clone(),
        width: out_width as i32,
        height: out_height as i32,
        frame_used: 0,
        internal_format,
        wrap_clamp_mode: gl_wrap_clamp_mode,
        mipmap,
        allow_picmip,
        last_level_used_on,
    });
    assets.image_names.insert(p_name, handle);

    // DEFERRED: R4 — `if (bRectangle) { qglDisable(uiTarget);
    // qglEnable(GL_TEXTURE_2D); }` restore.
    // Source: oracle/codemp/renderer/tr_image.cpp:1291-1295

    handle
}
