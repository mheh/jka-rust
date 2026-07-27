//! Raven `tr_image.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_image.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use std::f32::consts::PI;
use std::sync::Arc;

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::cvar_fns::Cvar_Set;
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Read, FS_ReadFileVec};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{fileHandle_t, MAX_QPATH};

use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::GlConfig;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::tr_globals_t::FOG_TABLE_SIZE;

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

/// Per-subsystem state for `tr_image.cpp`'s two 256-entry gamma/intensity
/// lookup tables — Raven file-scope statics `s_gammatable`/`s_intensitytable`
/// with no R2 carrier of their own (kind-3, genuine cross-frame state);
/// named by this wave per DEC-37 A13.3. Written by `R_SetColorMappings`,
/// read by `R_GammaCorrect`/`R_LightScaleTexture`. Zero-initialized to match
/// the oracle's zero-filled `static` arrays before the first
/// `R_SetColorMappings` call.
///
/// Source: `oracle/codemp/renderer/tr_image.cpp` (file-scope statics near
/// `R_SetColorMappings`, `:2847-2919`)
#[derive(Clone)]
pub struct TrImageState {
    /// Raven `byte s_gammatable[256]`.
    pub gamma_table: [u8; 256],
    /// Raven `byte s_intensitytable[256]`.
    pub intensity_table: [u8; 256],
}

impl Default for TrImageState {
    fn default() -> TrImageState {
        TrImageState {
            gamma_table: [0; 256],
            intensity_table: [0; 256],
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
