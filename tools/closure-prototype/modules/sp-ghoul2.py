# SP ghoul2_shared.h lives in code/game/ (engine-linked), not code/ghoul2/.

SPEC = dict(
    name="sp-ghoul2",
    lang="c++", entry=["code/game/q_shared.h",
                       "code/game/ghoul2_shared.h", "code/ghoul2/ghoul2_gore.h",
                       "code/ghoul2/G2.h"],
    includes=["code/ghoul2", "code/game", "code/qcommon", "code"],
    defines=["NDEBUG", "_IMMERSION"],
)
