# botlib headers assume the classic Q3 include order (q_shared -> l_* ->
# aasfile -> botlib -> be_*); the entry list reproduces it.
# NB: the botlib interface header is codemp/game/botlib.h (there is no
# codemp/botlib/botlib.h) — Raven's be_* files include it cross-dir.

SPEC = dict(
    name="mp-botlib",
    lang="c", entry=["codemp/game/q_shared.h", "codemp/botlib/l_crc.h",
                     "codemp/botlib/l_libvar.h", "codemp/botlib/l_log.h",
                     "codemp/botlib/l_memory.h", "codemp/botlib/l_script.h",
                     "codemp/botlib/l_precomp.h", "codemp/botlib/l_struct.h",
                     "codemp/botlib/l_utils.h", "codemp/botlib/aasfile.h",
                     "codemp/game/botlib.h",
                     # game-side interface headers must precede be_aas_def.h
                     # (aas_entity_s embeds aas_entityinfo_t from be_aas.h)
                     "codemp/game/be_aas.h", "codemp/game/be_ai_char.h",
                     "codemp/game/be_ai_chat.h", "codemp/game/be_ai_gen.h",
                     "codemp/game/be_ai_goal.h", "codemp/game/be_ai_move.h",
                     "codemp/game/be_ai_weap.h", "codemp/game/be_ea.h",
                     "codemp/botlib/be_aas_def.h",
                     "codemp/botlib/be_aas_funcs.h", "codemp/botlib/be_aas_bsp.h",
                     "codemp/botlib/be_aas_cluster.h", "codemp/botlib/be_aas_debug.h",
                     "codemp/botlib/be_aas_entity.h", "codemp/botlib/be_aas_file.h",
                     "codemp/botlib/be_aas_main.h", "codemp/botlib/be_aas_move.h",
                     "codemp/botlib/be_aas_optimize.h", "codemp/botlib/be_aas_reach.h",
                     "codemp/botlib/be_aas_route.h", "codemp/botlib/be_aas_routealt.h",
                     "codemp/botlib/be_aas_sample.h", "codemp/botlib/be_ai_weight.h",
                     "codemp/botlib/be_interface.h"],
    includes=["codemp/botlib", "codemp/game", "codemp"],
    defines=["NDEBUG", "MISSIONPACK", "BOTLIB", "_JK2"],
)
