SPEC = dict(
    name="sp-bg",
    lang="c++", entry=["code/game/g_local.h", "code/game/bg_local.h"],
    includes=["code/game", "code"],
    defines=["NDEBUG", "_IMMERSION"],
    srcglob=["code/game/bg_*.cpp"],
)
