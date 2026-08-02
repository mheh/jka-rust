# srcglob is the ui.vcproj compiled set EXACTLY, not codemp/ui/*.c:
# ui_players.c and ui_util.c are NOT in ui.vcproj or ui.q3asm (vestigial
# Q3/JK2 surface — ui_players.c refs symbols absent from MP: ANIM_TOGGLEBIT,
# playerInfo_t.headModel; its only would-be caller UI_DrawOpponent is inside
# a /* */ block in ui_main.c). Dropping them removes 26 dead port-target fns
# and the semantic parse errors they caused. The six live UI sources +
# ui_syscalls.c (seam) are listed; bg_*.c ride in as satisfied-dep callee
# bodies (already ported → mp_bg). Found 2026-07-24 during U0.

SPEC = dict(
    name="mp-ui",
    lang="c", entry="codemp/ui/ui_local.h",
    includes=["codemp/ui", "codemp/game"],
    defines=["NDEBUG", "MISSIONPACK", "UI_EXPORTS", "_JK2"],
    srcglob=["codemp/ui/ui_main.c", "codemp/ui/ui_atoms.c",
             "codemp/ui/ui_force.c", "codemp/ui/ui_shared.c",
             "codemp/ui/ui_gameinfo.c", "codemp/ui/ui_saber.c",
             "codemp/ui/ui_syscalls.c", "codemp/game/bg_*.c"],
)
