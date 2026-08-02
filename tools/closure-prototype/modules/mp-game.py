# MP tree (codemp/) — plain C module.

SPEC = dict(
    name="mp-game",
    lang="c", entry=["codemp/game/b_local.h", "codemp/game/ai_main.h",
                     "codemp/game/w_saber.h", "codemp/game/bg_local.h",
                     "codemp/game/botlib.h"],
    includes=["codemp/game"],
    defines=["NDEBUG", "MISSIONPACK", "QAGAME", "_JK2"],
    srcglob=["codemp/game/*.c"],
)
