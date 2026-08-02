# SP tree (code/) — C++ throughout.

SPEC = dict(
    name="sp-game",
    lang="c++", entry=["code/game/b_local.h", "code/game/wp_saber.h",
                       "code/game/g_functions.h", "code/game/g_vehicles.h",
                       "code/game/g_roff.h", "code/game/objectives.h",
                       "code/game/g_items.h", "code/game/fields.h",
                       "code/game/characters.h", "code/game/hitlocs.h",
                       "code/game/events.h", "code/game/bg_local.h"],
    includes=["code/game", "code"],
    defines=["NDEBUG", "_IMMERSION"],
    srcglob=["code/game/*.cpp"],
)
