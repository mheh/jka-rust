# Multi-entry: tr_local.h pulls tr_public/qgl/ghoul2_shared/mdx_format.
# qgl.h/glext.h parse via the glshim include dir (GL scalar typedefs only);
# their own types are GL bindings — replaced, never swept. Skipped:
# qgl_console/glext_console (Xbox).
# cm_landscape.h precedes tr_landscape.h (HEIGHT_RESOLUTION array bound).
# The windows-type defines cover qgl.h's unguarded WGL pbuffer section and
# tr_local.h's HDC/HGLRC/USHORT fields (handles = pointer-size, layout-
# correct). -fdeclspec parses `__declspec(align(16))` on shaderCommands_t.
# srcglob is the jk2mp.vcproj client-renderer compiled set EXACTLY (R1,
# 2026-07-25), not `codemp/renderer/tr_*.cpp`: of the 32 tr_*.cpp files on
# disk, jk2mp.vcproj lists 28. Excluded (both confirmed absent from every
# oracle .vcproj — grep of tr_bsp_xbox/tr_curve_xbox/tr_image_xbox/
# tr_flares across every *.vcproj/*.vcxproj/*.mak in the oracle tree):
#   tr_bsp_xbox.cpp, tr_curve_xbox.cpp, tr_image_xbox.cpp — Xbox-platform
#     twins of tr_bsp/tr_curve/tr_image; PC MP never links them.
#   tr_flares.cpp — dead in MP: not in jk2mp.vcproj, and its only would-be
#     caller `RB_RenderFlares()` sits commented out at tr_backend.cpp:1244.
# matcomp.c also lives in codemp/renderer/ and IS in jk2mp.vcproj, but it
# is not a tr_* file and already has a srcglob home (mp-engine-ded); not
# duplicated here.

SPEC = dict(
    name="mp-renderer",
    lang="c++", entry=["codemp/game/q_shared.h", "codemp/qcommon/qcommon.h",
                       "codemp/qcommon/cm_landscape.h",
                       "codemp/renderer/tr_local.h", "codemp/renderer/tr_font.h",
                       "codemp/renderer/tr_quicksprite.h",
                       "codemp/renderer/tr_WorldEffects.h",
                       "codemp/renderer/tr_landscape.h",
                       "codemp/renderer/matcomp.h"],
    includes=["codemp/renderer", "codemp/game", "codemp/qcommon", "codemp",
              "../tools/closure-prototype/glshim"],
    defines=["NDEBUG", "MISSIONPACK", "_JK2",
             "USHORT=unsigned short", "BOOL=int", "UINT=unsigned int",
             "FLOAT=float", "HDC=void *", "HGLRC=void *",
             "DECLARE_HANDLE(name)=typedef void *name",
             # R1 (2026-07-25): the srcglob addition sweeps real fn bodies
             # (tr_font.cpp GDI-flavored font loading, tr_shader.cpp path
             # building) that mp-renderer's header-only predecessor never
             # reached. Same win32-type-spoof + MSVC str* alias pattern
             # mp-engine-ded already established; PATH_SEP is a genuine
             # gap shared by every un-platformed profile (q_shared.h only
             # defines it inside _WIN32/__MACOS__/__linux__/__FreeBSD__
             # blocks, no catch-all) — pinned to '/' for parse purposes.
             "LPCTSTR=const char *", "LPCSTR=const char *",
             "COLORREF=unsigned int", "DWORD=unsigned int",
             "WORD=unsigned short", "BYTE=unsigned char",
             "HANDLE=void *", "LPVOID=void *", "__int64=long long",
             "stricmp=strcasecmp", "strnicmp=strncasecmp",
             "strcmpi=strcasecmp", "PATH_SEP='/'",
             # Raven gates dead PowerPC/big-endian byte-swap fallbacks
             # (stale pre-refactor mdxmVertex_t field names — real source
             # bit-rot, `tr_model.cpp`/`tr_ghoul2.cpp`) behind `#ifndef
             # _M_IX86`; MSVC always auto-defines `_M_IX86` for x86
             # targets, which is every retail MP client config Raven
             # shipped. Defining it (mirroring the vcproj Release configs,
             # same policy as the rest of this profile) takes the live
             # (non-dead) branch and drops ~200 diagnostics from code that
             # never compiled on retail either.
             "_M_IX86=600"],
    flags=["-fdeclspec"],
    srcglob=["codemp/renderer/tr_animation.cpp", "codemp/renderer/tr_arioche.cpp",
             "codemp/renderer/tr_backend.cpp", "codemp/renderer/tr_bsp.cpp",
             "codemp/renderer/tr_cmds.cpp", "codemp/renderer/tr_curve.cpp",
             "codemp/renderer/tr_font.cpp", "codemp/renderer/tr_ghoul2.cpp",
             "codemp/renderer/tr_image.cpp", "codemp/renderer/tr_init.cpp",
             "codemp/renderer/tr_light.cpp", "codemp/renderer/tr_main.cpp",
             "codemp/renderer/tr_marks.cpp", "codemp/renderer/tr_mesh.cpp",
             "codemp/renderer/tr_model.cpp", "codemp/renderer/tr_noise.cpp",
             "codemp/renderer/tr_quicksprite.cpp", "codemp/renderer/tr_scene.cpp",
             "codemp/renderer/tr_shade.cpp", "codemp/renderer/tr_shade_calc.cpp",
             "codemp/renderer/tr_shader.cpp", "codemp/renderer/tr_shadows.cpp",
             "codemp/renderer/tr_sky.cpp", "codemp/renderer/tr_surface.cpp",
             "codemp/renderer/tr_surfacesprites.cpp", "codemp/renderer/tr_terrain.cpp",
             "codemp/renderer/tr_world.cpp", "codemp/renderer/tr_WorldEffects.cpp"],

    # ---- chain fields (enginesweep) ----
    label="renderer",
    sweep_title="mp-renderer (full MP client renderer)",
    sweep_desc=(
        "the `mp-renderer` profile (jk2mp.vcproj client renderer compile "
        "set: all 28 `tr_*.cpp` translation units, `-DMISSIONPACK`, no "
        "`-DDEDICATED` — the full frontend+backend renderer)"),
    # The full engine subsystem list keeps the historical sweep-report shape.
    subsystems=["qcommon", "server", "ghoul2", "botlib", "icarus", "RMG",
                "renderer"],
)
