//! Standard OpenGL enum values used as CPU-visible image metadata.
//!
//! These are Khronos-registry constants (stable ABI, identical in every
//! `GL/gl.h`), not oracle-derived values — Raven never defined them, it
//! included the platform header. They are homed here so the frontend port can
//! carry the pixel-format and wrap-mode metadata Raven stores on `image_t`
//! (`internalFormat`, `wrapClampMode`) and compares in
//! `R_FindImageFile_NoLoad`'s cache-hit test.
//!
//! Scope: CPU-visible metadata only. Every actual GL call
//! (`qglTexParameter*`/`qglBindTexture`/`qglTexImage2D`/…) remains
//! DEFERRED: R4 (DEC-01/DEC-37 A13.2) — this module does not open the GL
//! surface, it only supplies the numbers those metadata fields hold.

/// `GL_RGBA` — the pixel format every `R_LoadImage` path yields
/// (`oracle/codemp/renderer/tr_image.cpp:2235`) and the `format` argument of
/// every `R_CreateImage` call in the file.
pub const GL_RGBA: i32 = 0x1908;

/// `GL_REPEAT` — the default texture wrap mode (`image_t::wrapClampMode`).
pub const GL_REPEAT: i32 = 0x2901;

/// `GL_CLAMP` — the legacy clamp wrap mode; substituted for
/// `GL_CLAMP_TO_EDGE` when `glConfig.clampToEdgeAvailable`
/// (`oracle/codemp/renderer/tr_image.cpp:1214-1216,2547-2550`).
pub const GL_CLAMP: i32 = 0x2900;

/// `GL_CLAMP_TO_EDGE` — OpenGL 1.2's clamp mode, which excludes the border
/// color at the edges (Raven's comment at
/// `oracle/codemp/renderer/tr_image.cpp:2712-2714`).
pub const GL_CLAMP_TO_EDGE: i32 = 0x812F;

/// `GL_NEAREST` — `modes[0]` minify/magnify filter
/// (`oracle/codemp/renderer/tr_image.cpp:61`).
pub const GL_NEAREST: i32 = 0x2600;

/// `GL_LINEAR` — `modes[1]` minify/magnify filter
/// (`oracle/codemp/renderer/tr_image.cpp:62`).
pub const GL_LINEAR: i32 = 0x2601;

/// `GL_NEAREST_MIPMAP_NEAREST` — `modes[2]` minify filter
/// (`oracle/codemp/renderer/tr_image.cpp:63`).
pub const GL_NEAREST_MIPMAP_NEAREST: i32 = 0x2700;

/// `GL_LINEAR_MIPMAP_NEAREST` — `modes[3]` minify filter, and
/// `gl_filter_min`'s initial value (`oracle/codemp/renderer/tr_image.cpp:38`).
pub const GL_LINEAR_MIPMAP_NEAREST: i32 = 0x2701;

/// `GL_NEAREST_MIPMAP_LINEAR` — `modes[4]` minify filter
/// (`oracle/codemp/renderer/tr_image.cpp:65`).
pub const GL_NEAREST_MIPMAP_LINEAR: i32 = 0x2702;

/// `GL_LINEAR_MIPMAP_LINEAR` — `modes[5]` minify filter
/// (`oracle/codemp/renderer/tr_image.cpp:66`).
pub const GL_LINEAR_MIPMAP_LINEAR: i32 = 0x2703;

/// `GL_RGBA4` — a `R_BytesPerTex`/`R_ImageFormatName`-cased internal format
/// (`oracle/codemp/renderer/tr_image.cpp:165`), also a `*pformat` pick in the
/// 16-bit compression paths (`oracle/codemp/renderer/tr_image.cpp:713`).
pub const GL_RGBA4: i32 = 0x8056;

/// `GL_RGB5` — a `R_BytesPerTex`/`R_ImageFormatName`-cased internal format
/// (`oracle/codemp/renderer/tr_image.cpp:169`), also a `*pformat` pick in the
/// 16-bit compression paths (`oracle/codemp/renderer/tr_image.cpp:685`).
pub const GL_RGB5: i32 = 0x8050;

/// `GL_RGBA8` — a `R_BytesPerTex`/`R_ImageFormatName`-cased internal format
/// (`oracle/codemp/renderer/tr_image.cpp:174`), also a `*pformat` pick in the
/// 32-bit paths (`oracle/codemp/renderer/tr_image.cpp:717`).
pub const GL_RGBA8: i32 = 0x8058;

/// `GL_RGB8` — a `R_BytesPerTex`/`R_ImageFormatName`-cased internal format
/// (`oracle/codemp/renderer/tr_image.cpp:178`), also a `*pformat` pick in the
/// 32-bit paths (`oracle/codemp/renderer/tr_image.cpp:689`).
pub const GL_RGB8: i32 = 0x8051;

/// `GL_RGB4_S3TC` — the pre-EXT S3TC internal format id
/// (`oracle/codemp/renderer/tr_image.cpp:183`), the `*pformat` pick ahead of
/// the `GL_COMPRESSED_RGB_S3TC_DXT1_EXT` fallback
/// (`oracle/codemp/renderer/tr_image.cpp:669`).
pub const GL_RGB4_S3TC: i32 = 0x83A1;

/// `GL_COMPRESSED_RGB_S3TC_DXT1_EXT` — the `GL_EXT_texture_compression_s3tc`
/// DXT1 internal format, a `R_BytesPerTex`/`R_ImageFormatName`-cased case
/// (`oracle/codemp/renderer/tr_image.cpp:187`) and `*pformat` pick
/// (`oracle/codemp/renderer/tr_image.cpp:674`).
pub const GL_COMPRESSED_RGB_S3TC_DXT1_EXT: i32 = 0x83F0;

/// `GL_COMPRESSED_RGBA_S3TC_DXT5_EXT` — the `GL_EXT_texture_compression_s3tc`
/// DXT5 internal format, a `R_BytesPerTex`/`R_ImageFormatName`-cased case
/// (`oracle/codemp/renderer/tr_image.cpp:191`) and `*pformat` pick
/// (`oracle/codemp/renderer/tr_image.cpp:677`).
pub const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT: i32 = 0x83F3;
