# Same skip list as mp-engine; SP additionally has no vm_local (no QVM)
# and no GenericParser2/RoffSystem in qcommon (SP GP2 lives in game/).

SPEC = dict(
    name="sp-engine",
    lang="c++", entry=["code/game/q_shared.h",
                       "code/qcommon/qcommon.h", "code/qcommon/qfiles.h",
                       "code/qcommon/cm_local.h", "code/qcommon/cm_patch.h",
                       "code/qcommon/cm_landscape.h",
                       "code/qcommon/cm_randomterrain.h",
                       "code/qcommon/cm_terrainmap.h", "code/qcommon/files.h",
                       "code/qcommon/chash.h", "code/qcommon/fixedmap.h",
                       "code/qcommon/hstring.h", "code/qcommon/sstring.h",
                       "code/qcommon/MiniHeap.h",
                       "code/qcommon/stringed_ingame.h",
                       "code/qcommon/stringed_interface.h",
                       "code/qcommon/timing.h", "code/server/server.h"],
    includes=["code/qcommon", "code/game", "code/server", "code"],
    defines=["NDEBUG", "_IMMERSION"],
)
