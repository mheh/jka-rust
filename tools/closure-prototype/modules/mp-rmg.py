SPEC = dict(
    name="mp-rmg",
    lang="c++", entry=["codemp/game/q_shared.h", "codemp/qcommon/qcommon.h",
                       "codemp/RMG/RM_Headers.h"],
    includes=["codemp/RMG", "codemp/game", "codemp/qcommon", "codemp"],
    defines=["NDEBUG", "MISSIONPACK", "_JK2"],
)
