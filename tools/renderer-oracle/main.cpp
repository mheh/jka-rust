// renderer-oracle dumper (R3 shader-parse slice, DEC-37 ruling 15 / porting-rules
// §18): drives the UNMODIFIED oracle/codemp/renderer/tr_shader.cpp
// (R_InitShaders -> ScanAndLoadShaderFiles -> R_FindShader -> ParseShader ->
// FinishShader) over a fixture .shader file and dumps canonical shader_t /
// shaderStage_t state. See README.md for the full stub inventory this file
// implements; every stub's deterministic behavior is documented at its
// definition below AND in the README (the R3 Rust test must mirror both).
//
// Usage: rdump <shaders-dir> <names-file>
//   <shaders-dir>   a directory containing exactly the .shader file(s) for
//                    this run (ScanAndLoadShaderFiles scans it as "shaders").
//   <names-file>     shader identifiers to R_FindShader() + dump, one per
//                    line (blank lines and lines starting with '#' skipped).
#include "codemp/qcommon/exe_headers.h"
#include "codemp/renderer/tr_local.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <algorithm>
#include <dirent.h>
#include <stdexcept>

// ===========================================================================
// Globals tr_shader.cpp expects some OTHER tr_*.cpp TU to define (normally
// tr_init.cpp/tr_main.cpp). Zero-initialized static storage duration mirrors
// retail's own zero-initialized globals at process start.
// ===========================================================================
trGlobals_t tr;
glconfig_t glConfig;

// qgl* ARB/NV entry points CreateInternalShaders' glow-shader setup probes.
// Left null (never loaded): every call site is a `if (qglFoo)` guard (see
// stubs/qgl.h), so this deterministically takes the "extension unavailable"
// branch -- CollapseMultitexture also short-circuits on qglActiveTextureARB,
// so GL multitexture stage-collapsing never fires (documented in README).
void ( APIENTRY * qglActiveTextureARB )(GLenum) = nullptr;
void ( APIENTRY * qglGenProgramsARB )(GLsizei, GLuint *) = nullptr;
void ( APIENTRY * qglBindProgramARB )(GLenum, GLuint) = nullptr;
void ( APIENTRY * qglProgramStringARB )(GLenum, GLenum, GLsizei, const void *) = nullptr;
void ( APIENTRY * qglGetIntegerv )(GLenum, GLint *) = nullptr;
const GLubyte * ( APIENTRY * qglGetString )(GLenum) = nullptr;
void ( APIENTRY * qglCombinerParameteriNV )(GLenum, GLint) = nullptr;
GLuint ( APIENTRY * qglGenLists )(GLsizei) = nullptr;
void ( APIENTRY * qglNewList )(GLuint, GLenum) = nullptr;
void ( APIENTRY * qglEndList )(void) = nullptr;
void ( APIENTRY * qglCombinerInputNV )(GLenum, GLenum, GLenum, GLenum, GLenum, GLenum) = nullptr;
void ( APIENTRY * qglCombinerOutputNV )(GLenum, GLenum, GLenum, GLenum, GLenum, GLenum, GLenum, GLboolean, GLboolean, GLboolean) = nullptr;
void ( APIENTRY * qglFinalCombinerInputNV )(GLenum, GLenum, GLenum, GLenum) = nullptr;

// Cvars FinishShader reads. All fixed at their zero/off default -- no
// fixture depends on detail-texture culling, vertex-light stage collapsing,
// or the ui-fullscreen 2-pass-lightmap-stage suppression, so "always off" is
// a deterministic, documented stand-in (see README).
static cvar_t r_detailTextures_storage{};
static cvar_t r_vertexLight_storage{};
static cvar_t r_uiFullScreen_storage{};
cvar_t *r_detailTextures = &r_detailTextures_storage;
cvar_t *r_vertexLight = &r_vertexLight_storage;
cvar_t *r_uiFullScreen = &r_uiFullScreen_storage;

// Declared (non-static, external linkage) at the top of tr_shader.cpp itself;
// reused here exactly as CreateExternalShaders() reuses it, so the driver's
// R_FindShader() calls match retail's own calling convention.
extern const byte stylesDefault[MAXLIGHTMAPS];

// ===========================================================================
// Hunk_Alloc / Z_Malloc-family stand-in: a bump allocator over malloc, zeroed
// (retail Hunk_Alloc hands back zero-filled memory), never freed -- this is a
// one-shot dumper process, so leaking for the process lifetime is simplest
// and deterministic (see also ui-oracle's Hunk_Alloc-equivalent).
// ===========================================================================
void *Hunk_Alloc(int size, ha_pref preference) {
    void *p = malloc(size > 0 ? size : 1);
    memset(p, 0, size > 0 ? size : 1);
    return p;
}

void Com_Memcpy(void *dest, const void *src, const size_t count) { memcpy(dest, src, count); }
void Com_Memset(void *dest, const int val, const size_t count) { memset(dest, val, count); }

// Com_Printf/Com_DPrintf: retail prints warnings to the console. The dumper
// mirrors them to stderr (never stdout, so golden diffs are unaffected) --
// visible during `--regen`/spot-checks, silent in `cargo test`-style diffing.
void QDECL Com_Printf(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    fputs("[Com_Printf] ", stderr);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
}
void QDECL Com_DPrintf(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    fputs("[Com_DPrintf] ", stderr);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
}

// Com_Error: retail longjmps to a per-frame safe point, aborting whatever
// call stack triggered it. This dumper is a flat per-shader loop (not the
// engine's Com_Frame), so it substitutes a C++ exception -- caught once per
// R_FindShader() call in main() below -- for the same "abort this shader,
// keep going" behavior, unwinding safely through ParseTexMod/ParseShader's
// C++ stack frames (RAII-safe, unlike a raw longjmp). Divergence documented
// here and in README; the only fixture-reachable call site is ParseTexMod's
// tcMod-overflow guard (see edge_cases.shader) -- this TU's other three
// Com_Error sites (FinishShader's lightstyle-without-lightmap guard,
// ScanAndLoadShaderFiles's two file-not-found guards) are unreachable under
// this harness's fixed calling convention; see README's Com_Error row for
// why each is unreachable.
struct ComErrorAbort {
    int code;
    std::string message;
};
void QDECL Com_Error(int code, const char *fmt, ...) {
    char buf[4096];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);
    throw ComErrorAbort{code, std::string(buf)};
}

// ===========================================================================
// FS_* stand-ins: ScanAndLoadShaderFiles("shaders") is real, unmodified
// oracle code -- the dumper backs it with a REAL directory (argv[1]),
// scanned deterministically (alphabetical order; retail's own order depends
// on pak mount order, which this harness has no equivalent of). Single-shot
// process: FS_Free* are no-ops (never freed, simplest + deterministic).
// ===========================================================================
static std::string g_shadersDir;

char **FS_ListFiles(const char *directory, const char *extension, int *numfiles) {
    std::vector<std::string> names;
    DIR *d = opendir(g_shadersDir.c_str());
    if (d) {
        struct dirent *ent;
        size_t elen = strlen(extension);
        while ((ent = readdir(d)) != nullptr) {
            std::string fn = ent->d_name;
            if (fn.size() > elen && fn.compare(fn.size() - elen, elen, extension) == 0) {
                names.push_back(fn);
            }
        }
        closedir(d);
    }
    std::sort(names.begin(), names.end());
    char **out = (char **)malloc(sizeof(char *) * (names.size() > 0 ? names.size() : 1));
    for (size_t i = 0; i < names.size(); i++) {
        out[i] = strdup(names[i].c_str());
    }
    *numfiles = (int)names.size();
    return out;
}
void FS_FreeFileList(char ** /*fileList*/) {}

int FS_ReadFile(const char *qpath, void **buffer) {
    const char *base = strrchr(qpath, '/');
    base = base ? base + 1 : qpath;
    std::string full = g_shadersDir + "/" + base;
    FILE *f = fopen(full.c_str(), "rb");
    if (!f) {
        if (buffer) *buffer = nullptr;
        return -1;
    }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *data = (char *)malloc(len + 1);
    size_t got = fread(data, 1, len, f);
    data[got] = '\0';
    fclose(f);
    if (buffer) *buffer = data;
    return (int)got;
}
void FS_FreeFile(void * /*buffer*/) {}

// ===========================================================================
// R_FindImageFile: always succeeds (never returns NULL -- unlike retail, which
// can fail to find a texture on disk), handing back a freshly allocated
// image_t with a monotonically increasing texnum (base 1000, one call = one
// increment, mirroring ui-oracle's registerShaderNoMip-style counters) and a
// fixed 64x64 size. No dedup-by-name (retail's image cache would return the
// SAME image_t for two stages referencing the same path); the golden dump
// only observes per-stage image content (name/dims/texnum/flags), never
// pointer identity across stages, so dedup would not change any dumped
// field -- see README.
// ===========================================================================
static int g_imageCounter = 1000;
image_t *R_FindImageFile(const char *name, qboolean mipmap, qboolean allowPicmip, qboolean allowTC, int glWrapClampMode) {
    image_t *img = (image_t *)malloc(sizeof(image_t));
    memset(img, 0, sizeof(*img));
    Q_strncpyz(img->imgName, name, sizeof(img->imgName));
    img->width = 64;
    img->height = 64;
    img->texnum = (GLuint)(g_imageCounter++);
    img->mipmap = mipmap ? true : false;
    img->allowPicmip = allowPicmip ? true : false;
    img->wrapClampMode = glWrapClampMode;
    (void)allowTC;
    return img;
}

// CIN_PlayCinematic: videoMap's cinematic handle. Retail plays a .roq and
// hands back a scratch-image slot in [0, NUM_SCRATCH_IMAGES); the stub hands
// out a monotonically increasing counter mod NUM_SCRATCH_IMAGES (main()
// pre-populates every tr.scratchImage[] slot below so the handle is always
// valid to dereference -- see ParseStage's `map` case at
// oracle/codemp/renderer/tr_shader.cpp:1444-1462).
static int g_cinCounter = 0;
int CIN_PlayCinematic(const char * /*arg0*/, int, int, int, int, int) {
    int handle = g_cinCounter % NUM_SCRATCH_IMAGES;
    g_cinCounter++;
    return handle;
}

// Reached only from ParseSkyParms (side effect on a module-private sky
// tex-coord table this harness never reads) and R_MergeShaders /
// R_ShaderList_f (RMG terrain-blend + console-command paths no fixture
// exercises) -- deterministic no-ops.
void R_InitSkyTexCoords(float /*cloudLayerHeight*/) {}
void R_SyncRenderThread(void) {}
int Cmd_Argc(void) { return 0; }

// ===========================================================================
// Canonical dump. Field-by-field, deterministic order (source declaration
// order), %.6f floats, symbolic names for the small closed enums (numeric
// value alongside every one, so a diff never hides behind a renamed label).
// ===========================================================================
static void DumpImage(const char *field, image_t *img) {
    if (!img) {
        printf("  %s = NULL\n", field);
        return;
    }
    printf("  %s = { name=\"%s\" width=%d height=%d texnum=%u mipmap=%d allowPicmip=%d wrapClampMode=%d }\n",
           field, img->imgName, img->width, img->height, (unsigned)img->texnum,
           img->mipmap ? 1 : 0, img->allowPicmip ? 1 : 0, img->wrapClampMode);
}

static const char *NameCullType(cullType_t t) {
    switch (t) {
        case CT_FRONT_SIDED: return "CT_FRONT_SIDED";
        case CT_BACK_SIDED: return "CT_BACK_SIDED";
        case CT_TWO_SIDED: return "CT_TWO_SIDED";
    }
    return "CT_?";
}
static const char *NameFogPass(fogPass_t t) {
    switch (t) {
        case FP_NONE: return "FP_NONE";
        case FP_EQUAL: return "FP_EQUAL";
        case FP_LE: return "FP_LE";
        case FP_GLFOG: return "FP_GLFOG";
    }
    return "FP_?";
}
static const char *NameColorGen(colorGen_t g) {
    switch (g) {
        case CGEN_BAD: return "CGEN_BAD";
        case CGEN_IDENTITY_LIGHTING: return "CGEN_IDENTITY_LIGHTING";
        case CGEN_IDENTITY: return "CGEN_IDENTITY";
        case CGEN_ENTITY: return "CGEN_ENTITY";
        case CGEN_ONE_MINUS_ENTITY: return "CGEN_ONE_MINUS_ENTITY";
        case CGEN_EXACT_VERTEX: return "CGEN_EXACT_VERTEX";
        case CGEN_VERTEX: return "CGEN_VERTEX";
        case CGEN_ONE_MINUS_VERTEX: return "CGEN_ONE_MINUS_VERTEX";
        case CGEN_WAVEFORM: return "CGEN_WAVEFORM";
        case CGEN_LIGHTING_DIFFUSE: return "CGEN_LIGHTING_DIFFUSE";
        case CGEN_LIGHTING_DIFFUSE_ENTITY: return "CGEN_LIGHTING_DIFFUSE_ENTITY";
        case CGEN_FOG: return "CGEN_FOG";
        case CGEN_CONST: return "CGEN_CONST";
        case CGEN_LIGHTMAPSTYLE: return "CGEN_LIGHTMAPSTYLE";
    }
    return "CGEN_?";
}
static const char *NameAlphaGen(alphaGen_t g) {
    switch (g) {
        case AGEN_IDENTITY: return "AGEN_IDENTITY";
        case AGEN_SKIP: return "AGEN_SKIP";
        case AGEN_ENTITY: return "AGEN_ENTITY";
        case AGEN_ONE_MINUS_ENTITY: return "AGEN_ONE_MINUS_ENTITY";
        case AGEN_VERTEX: return "AGEN_VERTEX";
        case AGEN_ONE_MINUS_VERTEX: return "AGEN_ONE_MINUS_VERTEX";
        case AGEN_LIGHTING_SPECULAR: return "AGEN_LIGHTING_SPECULAR";
        case AGEN_WAVEFORM: return "AGEN_WAVEFORM";
        case AGEN_PORTAL: return "AGEN_PORTAL";
        case AGEN_BLEND: return "AGEN_BLEND";
        case AGEN_CONST: return "AGEN_CONST";
        case AGEN_DOT: return "AGEN_DOT";
        case AGEN_ONE_MINUS_DOT: return "AGEN_ONE_MINUS_DOT";
    }
    return "AGEN_?";
}
static const char *NameTcGen(texCoordGen_t g) {
    switch (g) {
        case TCGEN_BAD: return "TCGEN_BAD";
        case TCGEN_IDENTITY: return "TCGEN_IDENTITY";
        case TCGEN_LIGHTMAP: return "TCGEN_LIGHTMAP";
        case TCGEN_LIGHTMAP1: return "TCGEN_LIGHTMAP1";
        case TCGEN_LIGHTMAP2: return "TCGEN_LIGHTMAP2";
        case TCGEN_LIGHTMAP3: return "TCGEN_LIGHTMAP3";
        case TCGEN_TEXTURE: return "TCGEN_TEXTURE";
        case TCGEN_ENVIRONMENT_MAPPED: return "TCGEN_ENVIRONMENT_MAPPED";
        case TCGEN_FOG: return "TCGEN_FOG";
        case TCGEN_VECTOR: return "TCGEN_VECTOR";
    }
    return "TCGEN_?";
}
static const char *NameTexMod(texMod_t t) {
    switch (t) {
        case TMOD_NONE: return "TMOD_NONE";
        case TMOD_TRANSFORM: return "TMOD_TRANSFORM";
        case TMOD_TURBULENT: return "TMOD_TURBULENT";
        case TMOD_SCROLL: return "TMOD_SCROLL";
        case TMOD_SCALE: return "TMOD_SCALE";
        case TMOD_STRETCH: return "TMOD_STRETCH";
        case TMOD_ROTATE: return "TMOD_ROTATE";
        case TMOD_ENTITY_TRANSLATE: return "TMOD_ENTITY_TRANSLATE";
    }
    return "TMOD_?";
}
static const char *NameDeform(deform_t d) {
    switch (d) {
        case DEFORM_NONE: return "DEFORM_NONE";
        case DEFORM_WAVE: return "DEFORM_WAVE";
        case DEFORM_NORMALS: return "DEFORM_NORMALS";
        case DEFORM_BULGE: return "DEFORM_BULGE";
        case DEFORM_MOVE: return "DEFORM_MOVE";
        case DEFORM_PROJECTION_SHADOW: return "DEFORM_PROJECTION_SHADOW";
        case DEFORM_AUTOSPRITE: return "DEFORM_AUTOSPRITE";
        case DEFORM_AUTOSPRITE2: return "DEFORM_AUTOSPRITE2";
        case DEFORM_TEXT0: return "DEFORM_TEXT0";
        case DEFORM_TEXT1: return "DEFORM_TEXT1";
        case DEFORM_TEXT2: return "DEFORM_TEXT2";
        case DEFORM_TEXT3: return "DEFORM_TEXT3";
        case DEFORM_TEXT4: return "DEFORM_TEXT4";
        case DEFORM_TEXT5: return "DEFORM_TEXT5";
        case DEFORM_TEXT6: return "DEFORM_TEXT6";
        case DEFORM_TEXT7: return "DEFORM_TEXT7";
    }
    return "DEFORM_?";
}
static const char *NameGenFunc(genFunc_t f) {
    switch (f) {
        case GF_NONE: return "GF_NONE";
        case GF_SIN: return "GF_SIN";
        case GF_SQUARE: return "GF_SQUARE";
        case GF_TRIANGLE: return "GF_TRIANGLE";
        case GF_SAWTOOTH: return "GF_SAWTOOTH";
        case GF_INVERSE_SAWTOOTH: return "GF_INVERSE_SAWTOOTH";
        case GF_NOISE: return "GF_NOISE";
        case GF_RAND: return "GF_RAND";
    }
    return "GF_?";
}
static const char *NameAcff(acff_t a) {
    switch (a) {
        case ACFF_NONE: return "ACFF_NONE";
        case ACFF_MODULATE_RGB: return "ACFF_MODULATE_RGB";
        case ACFF_MODULATE_RGBA: return "ACFF_MODULATE_RGBA";
        case ACFF_MODULATE_ALPHA: return "ACFF_MODULATE_ALPHA";
    }
    return "ACFF_?";
}
static const char *NameFogOverride(EGLFogOverride o) {
    switch (o) {
        case GLFOGOVERRIDE_NONE: return "GLFOGOVERRIDE_NONE";
        case GLFOGOVERRIDE_BLACK: return "GLFOGOVERRIDE_BLACK";
        case GLFOGOVERRIDE_WHITE: return "GLFOGOVERRIDE_WHITE";
        case GLFOGOVERRIDE_MAX: return "GLFOGOVERRIDE_MAX";
    }
    return "GLFOGOVERRIDE_?";
}
static const char *NameSurfSpriteType(int t) {
    switch (t) {
        case SURFSPRITE_NONE: return "SURFSPRITE_NONE";
        case SURFSPRITE_VERTICAL: return "SURFSPRITE_VERTICAL";
        case SURFSPRITE_ORIENTED: return "SURFSPRITE_ORIENTED";
        case SURFSPRITE_EFFECT: return "SURFSPRITE_EFFECT";
        case SURFSPRITE_WEATHERFX: return "SURFSPRITE_WEATHERFX";
    }
    return "SURFSPRITE_?";
}
static const char *NameSurfSpriteFacing(int f) {
    switch (f) {
        case SURFSPRITE_FACING_NORMAL: return "SURFSPRITE_FACING_NORMAL";
        case SURFSPRITE_FACING_UP: return "SURFSPRITE_FACING_UP";
        case SURFSPRITE_FACING_DOWN: return "SURFSPRITE_FACING_DOWN";
        case SURFSPRITE_FACING_ANY: return "SURFSPRITE_FACING_ANY";
    }
    return "SURFSPRITE_FACING_?";
}

static void DumpWaveForm(const char *field, const waveForm_t &w) {
    printf("    %s = { func=%s(%d) base=%.6f amplitude=%.6f phase=%.6f frequency=%.6f }\n",
           field, NameGenFunc(w.func), (int)w.func, w.base, w.amplitude, w.phase, w.frequency);
}

static void DumpBundle(int idx, const textureBundle_t &b) {
    printf("  bundle[%d]:\n", idx);
    // animMap/clampanimMap/oneshotanimMap (ParseStage,
    // oracle/codemp/renderer/tr_shader.cpp:1400-1443) repurpose `image` as an
    // `image_t **` of numImageAnimations frames instead of a single
    // `image_t *` -- dereferencing it as a scalar image_t* here would read
    // garbage. Every other keyword (map/clampmap/videoMap/$lightmap/
    // $whiteimage) leaves numImageAnimations at 0 and `image` as a normal
    // single pointer.
    if (b.numImageAnimations > 0) {
        image_t **frames = (image_t **)b.image;
        printf("    image = image_t*[%d] {\n", (int)b.numImageAnimations);
        for (int i = 0; i < b.numImageAnimations; i++) {
            char field[32];
            snprintf(field, sizeof(field), "[%d]", i);
            DumpImage(field, frames[i]);
        }
        printf("    }\n");
    } else {
        DumpImage("image", b.image);
    }
    printf("    tcGen = %s(%d)\n", NameTcGen(b.tcGen), (int)b.tcGen);
    if (b.tcGen == TCGEN_VECTOR && b.tcGenVectors) {
        printf("    tcGenVectors = [ (%.6f,%.6f,%.6f) (%.6f,%.6f,%.6f) ]\n",
               b.tcGenVectors[0][0], b.tcGenVectors[0][1], b.tcGenVectors[0][2],
               b.tcGenVectors[1][0], b.tcGenVectors[1][1], b.tcGenVectors[1][2]);
    }
    printf("    numTexMods = %d\n", (int)b.numTexMods);
    for (int i = 0; i < b.numTexMods; i++) {
        const texModInfo_t &tm = b.texMods[i];
        printf("    texMods[%d] = { type=%s(%d) wave={func=%s base=%.6f amp=%.6f phase=%.6f freq=%.6f} matrix=[[%.6f,%.6f],[%.6f,%.6f]] translate=[%.6f,%.6f] }\n",
               i, NameTexMod(tm.type), (int)tm.type, NameGenFunc(tm.wave.func),
               tm.wave.base, tm.wave.amplitude, tm.wave.phase, tm.wave.frequency,
               tm.matrix[0][0], tm.matrix[0][1], tm.matrix[1][0], tm.matrix[1][1],
               tm.translate[0], tm.translate[1]);
    }
    printf("    numImageAnimations = %d\n", (int)b.numImageAnimations);
    printf("    imageAnimationSpeed = %.6f\n", b.imageAnimationSpeed);
    printf("    isLightmap = %d\n", b.isLightmap ? 1 : 0);
    printf("    oneShotAnimMap = %d\n", b.oneShotAnimMap ? 1 : 0);
    printf("    vertexLightmap = %d\n", b.vertexLightmap ? 1 : 0);
    printf("    isVideoMap = %d\n", b.isVideoMap ? 1 : 0);
    printf("    videoMapHandle = %d\n", b.videoMapHandle);
}

static void DumpStage(int idx, const shaderStage_t &s) {
    printf(" stage[%d]:\n", idx);
    printf("  active = %d\n", s.active ? 1 : 0);
    printf("  isDetail = %d\n", s.isDetail ? 1 : 0);
    printf("  index = %d\n", (int)s.index);
    printf("  lightmapStyle = %d\n", (int)s.lightmapStyle);
    DumpBundle(0, s.bundle[0]);
    DumpBundle(1, s.bundle[1]);
    DumpWaveForm("rgbWave", s.rgbWave);
    printf("    rgbGen = %s(%d)\n", NameColorGen(s.rgbGen), (int)s.rgbGen);
    DumpWaveForm("alphaWave", s.alphaWave);
    printf("    alphaGen = %s(%d)\n", NameAlphaGen(s.alphaGen), (int)s.alphaGen);
    printf("  constantColor = [%d,%d,%d,%d]\n", s.constantColor[0], s.constantColor[1], s.constantColor[2], s.constantColor[3]);
    printf("  stateBits = 0x%08x\n", (unsigned)s.stateBits);
    printf("  adjustColorsForFog = %s(%d)\n", NameAcff(s.adjustColorsForFog), (int)s.adjustColorsForFog);
    printf("  mGLFogColorOverride = %s(%d)\n", NameFogOverride(s.mGLFogColorOverride), (int)s.mGLFogColorOverride);
    if (s.ss) {
        printf("  ss = { type=%s(%d) width=%.6f height=%.6f density=%.6f wind=%.6f windIdle=%.6f fadeDist=%.6f fadeMax=%.6f fadeScale=%.6f fxAlphaStart=%.6f fxAlphaEnd=%.6f fxDuration=%.6f vertSkew=%.6f variance=[%.6f,%.6f] fxGrow=[%.6f,%.6f] facing=%s(%d) }\n",
               NameSurfSpriteType(s.ss->surfaceSpriteType), s.ss->surfaceSpriteType,
               s.ss->width, s.ss->height, s.ss->density, s.ss->wind, s.ss->windIdle,
               s.ss->fadeDist, s.ss->fadeMax, s.ss->fadeScale, s.ss->fxAlphaStart, s.ss->fxAlphaEnd,
               s.ss->fxDuration, s.ss->vertSkew, s.ss->variance[0], s.ss->variance[1],
               s.ss->fxGrow[0], s.ss->fxGrow[1], NameSurfSpriteFacing(s.ss->facing), s.ss->facing);
    } else {
        printf("  ss = NULL\n");
    }
    printf("  glow = %d\n", s.glow ? 1 : 0);
}

static void DumpShader(const char *requestedName, shader_t *sh) {
    printf("=== %s ===\n", requestedName);
    printf("name = \"%s\"\n", sh->name);
    printf("lightmapIndex = [%d,%d,%d,%d]\n", sh->lightmapIndex[0], sh->lightmapIndex[1], sh->lightmapIndex[2], sh->lightmapIndex[3]);
    printf("styles = [%d,%d,%d,%d]\n", sh->styles[0], sh->styles[1], sh->styles[2], sh->styles[3]);
    printf("sort = %.6f\n", sh->sort);
    printf("surfaceFlags = 0x%08x\n", (unsigned)sh->surfaceFlags);
    printf("contentFlags = 0x%08x\n", (unsigned)sh->contentFlags);
    printf("defaultShader = %d\n", sh->defaultShader ? 1 : 0);
    printf("explicitlyDefined = %d\n", sh->explicitlyDefined ? 1 : 0);
    printf("entityMergable = %d\n", sh->entityMergable ? 1 : 0);
    printf("isBumpMap = %d\n", sh->isBumpMap ? 1 : 0);
    if (sh->sky) {
        printf("sky = { cloudHeight=%.6f outerbox=[\n", sh->sky->cloudHeight);
        for (int i = 0; i < 6; i++) {
            printf("  [%d]", i);
            DumpImage("outerbox", sh->sky->outerbox[i]);
        }
        printf("] }\n");
    } else {
        printf("sky = NULL\n");
    }
    if (sh->fogParms) {
        printf("fogParms = { color=(%.6f,%.6f,%.6f) depthForOpaque=%.6f }\n",
               sh->fogParms->color[0], sh->fogParms->color[1], sh->fogParms->color[2], sh->fogParms->depthForOpaque);
    } else {
        printf("fogParms = NULL\n");
    }
    printf("portalRange = %.6f\n", sh->portalRange);
    printf("multitextureEnv = 0x%x\n", (unsigned)sh->multitextureEnv);
    printf("cullType = %s(%d)\n", NameCullType(sh->cullType), (int)sh->cullType);
    printf("polygonOffset = %d\n", sh->polygonOffset ? 1 : 0);
    printf("noMipMaps = %d\n", sh->noMipMaps ? 1 : 0);
    printf("noPicMip = %d\n", sh->noPicMip ? 1 : 0);
    printf("noTC = %d\n", sh->noTC ? 1 : 0);
    printf("fogPass = %s(%d)\n", NameFogPass(sh->fogPass), (int)sh->fogPass);
    printf("bumpVector = (%.6f,%.6f,%.6f)\n", sh->bumpVector[0], sh->bumpVector[1], sh->bumpVector[2]);
    printf("numDeforms = %d\n", (int)sh->numDeforms);
    for (int i = 0; i < sh->numDeforms; i++) {
        deformStage_t *ds = sh->deforms[i];
        printf(" deform[%d] = { deformation=%s(%d) moveVector=(%.6f,%.6f,%.6f) deformationSpread=%.6f bulgeWidth=%.6f bulgeHeight=%.6f bulgeSpeed=%.6f\n",
               i, NameDeform(ds->deformation), (int)ds->deformation,
               ds->moveVector[0], ds->moveVector[1], ds->moveVector[2],
               ds->deformationSpread, ds->bulgeWidth, ds->bulgeHeight, ds->bulgeSpeed);
        DumpWaveForm("deformationWave", ds->deformationWave);
        printf(" }\n");
    }
    printf("clampTime = %.6f\n", sh->clampTime);
    printf("timeOffset = %.6f\n", sh->timeOffset);
    printf("hasGlow = %d\n", sh->hasGlow ? 1 : 0);
    printf("remappedShader = %s\n", sh->remappedShader ? sh->remappedShader->name : "NULL");
    printf("numUnfoggedPasses = %d\n", (int)sh->numUnfoggedPasses);
    for (int i = 0; i < sh->numUnfoggedPasses; i++) {
        DumpStage(i, sh->stages[i]);
    }
    printf("\n");
}

// ===========================================================================
// Driver
// ===========================================================================
// A names-file line is either a bare shader identifier (driven with
// lightmapIndex = lightmapsNone, matching CreateExternalShaders' own
// convention -- see README) or `name:<lm0>`, overriding lightmapIndex[0] to
// test the `map $lightmap` "lightmap available" branch (lm0 in
// [0, tr.numLightmaps)) against the default "not available" branch (any
// other bare name).
struct NamesEntry {
    std::string name;
    int lm0 = LIGHTMAP_NONE;
};

static std::vector<NamesEntry> ReadNamesFile(const char *path) {
    std::vector<NamesEntry> names;
    FILE *f = fopen(path, "r");
    if (!f) {
        fprintf(stderr, "rdump: cannot open names file '%s'\n", path);
        exit(1);
    }
    char line[512];
    while (fgets(line, sizeof(line), f)) {
        size_t len = strlen(line);
        while (len > 0 && (line[len - 1] == '\n' || line[len - 1] == '\r')) line[--len] = '\0';
        if (len == 0 || line[0] == '#') continue;
        NamesEntry e;
        const char *colon = strchr(line, ':');
        if (colon) {
            e.name.assign(line, colon - line);
            e.lm0 = atoi(colon + 1);
        } else {
            e.name = line;
        }
        names.push_back(e);
    }
    fclose(f);
    return names;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: %s <shaders-dir> <names-file>\n", argv[0]);
        return 1;
    }
    g_shadersDir = argv[1];

    // Pre-populate the fixed-size tr globals R_InitShaders/R_FindShader/
    // ParseStage's `map` keyword read directly (never allocated lazily by
    // the real registration path, which this harness doesn't run).
    tr.defaultImage = R_FindImageFile("$oracle_default", qtrue, qtrue, qtrue, GL_REPEAT);
    tr.whiteImage = R_FindImageFile("$oracle_white", qtrue, qtrue, qtrue, GL_REPEAT);
    // One lightmap present, deterministically, so `map $lightmap` fixtures
    // exercise the "lightmap available" branch (see README).
    tr.numLightmaps = 1;
    tr.lightmaps[0] = R_FindImageFile("$oracle_lightmap0", qfalse, qfalse, qfalse, GL_CLAMP);
    for (int i = 0; i < NUM_SCRATCH_IMAGES; i++) {
        char nm[32];
        snprintf(nm, sizeof(nm), "$oracle_scratch%d", i);
        tr.scratchImage[i] = R_FindImageFile(nm, qfalse, qfalse, qfalse, GL_CLAMP);
    }

    try {
        R_InitShaders(qfalse);
    } catch (const ComErrorAbort &e) {
        fprintf(stderr, "rdump: R_InitShaders: Com_Error(code=%d, \"%s\")\n", e.code, e.message.c_str());
        return 1;
    }

    std::vector<NamesEntry> names = ReadNamesFile(argv[2]);
    for (const NamesEntry &entry : names) {
        int lightmapIndex[MAXLIGHTMAPS] = {entry.lm0, LIGHTMAP_NONE, LIGHTMAP_NONE, LIGHTMAP_NONE};
        try {
            shader_t *sh = R_FindShader(entry.name.c_str(), lightmapIndex, stylesDefault, qtrue);
            DumpShader(entry.name.c_str(), sh);
        } catch (const ComErrorAbort &e) {
            printf("=== %s ===\n", entry.name.c_str());
            printf("ERROR: Com_Error(code=%d, \"%s\")\n\n", e.code, e.message.c_str());
        }
    }
    return 0;
}
