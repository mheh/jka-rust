SPEC = dict(
    name="sp-ui",
    lang="c++", entry=["code/ui/ui_local.h", "code/ui/gameinfo.h"],
    includes=["code/ui", "code/game", "code"],
    defines=["NDEBUG", "_IMMERSION"],
)
