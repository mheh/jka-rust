SPEC = dict(
    name="mp-ghoul2",
    lang="c++", entry=["codemp/game/q_shared.h", "codemp/qcommon/qcommon.h",
                       "codemp/ghoul2/ghoul2_shared.h", "codemp/ghoul2/G2_local.h",
                       "codemp/ghoul2/G2_gore.h", "codemp/ghoul2/G2.h"],
    includes=["codemp/ghoul2", "codemp/game", "codemp/qcommon", "codemp"],
    defines=["NDEBUG", "MISSIONPACK", "_JK2"],
)
