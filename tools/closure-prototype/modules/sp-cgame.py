SPEC = dict(
    name="sp-cgame",
    lang="c++", entry=["code/cgame/cg_local.h", "code/cgame/cg_media.h",
                       "code/cgame/cg_lights.h"],
    includes=["code/cgame", "code/game", "code"],
    defines=["NDEBUG", "_IMMERSION"],
)
