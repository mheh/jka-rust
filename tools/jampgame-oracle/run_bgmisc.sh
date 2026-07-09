#!/bin/sh
# Build the jampgame "bgmisc" oracle dumper from the UNMODIFIED Raven sources
# and check (or, with --regen, regenerate) the golden dump
# crates/mp/game/tests/oracle/golden/bgmisc.txt.
#
# The dumper drives bg_misc.c + bg_weapons.c (the trajectory evaluators, the
# bg_itemlist / weaponData / ammoData tables, the item lookups, item-grab
# rules, and BG_PlayerStateToEntityState) linked against the UNMODIFIED
# q_shared.c + q_math.c. It is compiled with -DQAGAME (jampgame == Raven's
# QAGAME build), so bg_misc.c pulls g_local.h + the ghoul2/cgame/icarus
# header closure exactly as the pmove slice does. Every game/engine extern the
# TUs reference is stubbed in main_bgmisc.c (all abort()ing except the tiny
# handful the tested functions actually reach). Own build dir build-bgmisc/;
# oracle/ is never touched.
set -eu
cd "$(dirname "$0")"

# Committed parity data (fixtures + goldens) lives inside the mp_game crate so
# the crate is self-contained; this harness only generates/checks it.
DATA=../../crates/mp/game/tests/oracle

ORACLE=../../oracle/oracle
G=$ORACLE/codemp/game
Q=$ORACLE/codemp/qcommon
B=build-bgmisc

rm -rf "$B"
mkdir -p "$B/codemp/game" "$B/codemp/qcommon" "$B/codemp/ghoul2" \
         "$B/codemp/cgame" "$B/codemp/icarus"

# Full game/qcommon header closure + the two namespace shims + animtable.h,
# copied UNMODIFIED (bg_misc.c under -DQAGAME includes g_local.h which pulls
# the ghoul2/cgame/icarus trees).
cp "$G"/*.h "$B/codemp/game/"
cp "$Q"/*.h "$B/codemp/qcommon/"
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" "$B/codemp/"
cp "$ORACLE/codemp/cgame/animtable.h" "$B/codemp/game/"
cp "$ORACLE"/codemp/ghoul2/*.h "$B/codemp/ghoul2/" 2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h  "$B/codemp/cgame/"  2>/dev/null || true
cp "$ORACLE"/codemp/icarus/*.h "$B/codemp/icarus/" 2>/dev/null || true

cp "$G/bg_misc.c" "$G/bg_weapons.c" "$G/q_shared.c" "$G/q_math.c" "$B/codemp/game/"

# shim.h (force-included first): pull real libm so Raven's 2-arg powf(float,int)
# is renamed out of the way of libm's powf(float,float). Same as run.sh.
cat > "$B/codemp/game/shim.h" <<'EOF'
#include <math.h>
#define powf raven_powf
EOF

CFLAGS="-w -std=gnu11 -fgnu89-inline -D__linux__ -DQAGAME -D_FORTIFY_SOURCE=0 \
        -ffp-contract=off -include $B/codemp/game/shim.h -I. -I $B/codemp/game"

# q_math.c holds its own copies of the holdrand RNG (irand/Q_irand/flrand/...)
# which main_bgmisc.c does NOT redefine, so no rename is needed here — the only
# RNG reference on the tested paths is TranslateSaberColor's "random" (never hit).

OBJS=""
# shellcheck disable=SC2086
for f in bg_misc bg_weapons q_shared q_math; do
	cc $CFLAGS -c "$B/codemp/game/$f.c" -o "$B/bg_$f.o"
	OBJS="$OBJS $B/bg_$f.o"
done
# shellcheck disable=SC2086
cc $CFLAGS -c main_bgmisc.c -o "$B/bg_main.o"
# shellcheck disable=SC2086
cc "$B/bg_main.o" $OBJS -lm -o "$B/bgmisc_dump"

mkdir -p "$DATA/golden"
status=0
if [ "${1:-}" = "--regen" ]; then
	"$B/bgmisc_dump" "$DATA/fixtures/bgmisc" > "$DATA/golden/bgmisc.txt"
	echo "regenerated bgmisc"
else
	"$B/bgmisc_dump" "$DATA/fixtures/bgmisc" | diff -u "$DATA/golden/bgmisc.txt" - || status=1
fi

[ "$status" -eq 0 ] && echo "jampgame-oracle bgmisc: OK"
exit "$status"
