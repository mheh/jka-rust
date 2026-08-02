# SP tr_local.h additionally includes glext.h directly. Skipped:
# tr_stl.h (C++ STL helpers), tr_jpeg_interface.h (vendored jpeg-6),
# amd3d.h (3DNow asm), qgl_linked.h (binding macros).

SPEC = dict(
    name="sp-renderer",
    lang="c++", entry=["code/game/q_shared.h", "code/qcommon/qcommon.h",
                       "code/qcommon/cm_landscape.h",
                       "code/renderer/tr_local.h", "code/renderer/tr_font.h",
                       "code/renderer/tr_quicksprite.h",
                       "code/renderer/tr_WorldEffects.h",
                       "code/renderer/tr_landscape.h",
                       "code/renderer/matcomp.h"],
    includes=["code/renderer", "code/game", "code/qcommon", "code",
              "../tools/closure-prototype/glshim"],
    defines=["NDEBUG", "_IMMERSION",
             "USHORT=unsigned short", "BOOL=int", "UINT=unsigned int",
             "FLOAT=float", "HDC=void *", "HGLRC=void *",
             "DECLARE_HANDLE(name)=typedef void *name"],
    # SP tr_local.h names fields `or` (orientationr_t or;) — MSVC treats
    # `or` as an identifier; -fno-operator-names matches that, else clang
    # silently drops the field from viewParms_t/trGlobals_t.
    flags=["-fdeclspec", "-fno-operator-names"],
)
