# Multi-entry: qcommon owns many headers unreachable from qcommon.h
# (qfiles/cm_*/files/vm_local/containers); server.h rides this TU too.
# Skipped on purpose: unzip.h (vendored minizip), platform.h/sparc.h
# (replaced), INetProfile.h/CNetProfile (win32 net profiling, C++ track).

SPEC = dict(
    name="mp-engine",
    lang="c++", entry=["codemp/qcommon/qcommon.h", "codemp/qcommon/qfiles.h",
                       "codemp/qcommon/cm_local.h", "codemp/qcommon/cm_patch.h",
                       "codemp/qcommon/cm_landscape.h",
                       "codemp/qcommon/cm_randomterrain.h",
                       "codemp/qcommon/cm_terrainmap.h", "codemp/qcommon/files.h",
                       "codemp/qcommon/vm_local.h", "codemp/qcommon/chash.h",
                       "codemp/qcommon/fixedmap.h", "codemp/qcommon/hstring.h",
                       "codemp/qcommon/sstring.h", "codemp/qcommon/MiniHeap.h",
                       "codemp/qcommon/GenericParser2.h",
                       "codemp/qcommon/RoffSystem.h",
                       "codemp/qcommon/stringed_ingame.h",
                       "codemp/qcommon/stringed_interface.h",
                       "codemp/qcommon/timing.h", "codemp/server/server.h"],
    includes=["codemp/qcommon", "codemp/game", "codemp/server", "codemp"],
    defines=["NDEBUG", "MISSIONPACK", "_JK2"],
)
