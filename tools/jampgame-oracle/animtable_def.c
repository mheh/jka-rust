// animtable_def.c — provides the `animTable` symbol the unmodified oracle
// bg_saberLoad.c extern-references (readyAnim/drawAnim/... token lookups via
// GetIDForString). Raven's animTable is defined in cgame/animtable.h via the
// ENUM2STRING macro; that header is normally compiled into exactly one TU
// (cg_players / ui_players). We compile it standalone here so the definition
// is byte-faithful to the oracle (and 1:1 with the port's
// crates/mp/game/src/anim_table.rs, which is generated from the same header).
// _XBOX is not defined, so animtable.h takes its definition branch.
#include "q_shared.h"
#include "bg_public.h" // pulls anims.h -> animNumber_t + MAX_ANIMATIONS
#include "animtable.h"  // copied flat into build/codemp/game by run.sh
