# bg_* compiles into game/cgame/ui; parse it through the game TU. Alias
# so tier-named callers (crate mp_bg) resolve without knowing that.

SPEC = dict(
    name="mp-bg",
    lang="c", entry=["codemp/game/g_local.h", "codemp/game/bg_local.h",
                     "codemp/game/bg_saga.h"],
    includes=["codemp/game"],
    defines=["NDEBUG", "MISSIONPACK", "QAGAME", "_JK2"],
    srcglob=["codemp/game/bg_*.c"],
)
