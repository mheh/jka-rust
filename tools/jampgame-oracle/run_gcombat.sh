#!/bin/sh
# Build the jampgame "gcombat" oracle dumper from the UNMODIFIED Raven sources
# and check (or, with --regen, regenerate) the golden dump
# crates/mp/game/tests/oracle/golden/gcombat.txt.
#
# The dumper drives THREE pure-ish leaf functions of g_combat.c —
# RaySphereIntersections, G_GetHitLocation, CheckArmor — linked against the
# UNMODIFIED q_shared.c + q_math.c. g_combat.c is a large TU that extern-
# references ~120 game/engine symbols across functions the dumper never calls;
# main_gcombat.c defines the zeroed data globals it reads, and stubs_gcombat.c
# (compiled WITHOUT the game headers so its argless abort() stubs never clash
# with the real prototypes — the C linker binds by name alone) satisfies every
# unreachable function. It is compiled with -DQAGAME (jampgame == Raven's QAGAME
# build), so g_combat.c pulls g_local.h + the ghoul2/cgame/icarus header closure
# exactly as the bgmisc/pmove slices do. Own build dir build-gcombat/; oracle/
# is never touched.
set -eu
cd "$(dirname "$0")"

# Committed parity data (fixtures + goldens) lives inside the mp_game crate so
# the crate is self-contained; this harness only generates/checks it.
DATA=../../crates/mp/game/tests/oracle

ORACLE=../../oracle/oracle
G=$ORACLE/codemp/game
Q=$ORACLE/codemp/qcommon
B=build-gcombat

rm -rf "$B"
mkdir -p "$B/codemp/game" "$B/codemp/qcommon" "$B/codemp/ghoul2" \
         "$B/codemp/cgame" "$B/codemp/icarus"

# Full game/qcommon header closure + the two namespace shims + animtable.h,
# copied UNMODIFIED (g_combat.c under -DQAGAME includes g_local.h which pulls
# the ghoul2/cgame/icarus trees).
cp "$G"/*.h "$B/codemp/game/"
cp "$Q"/*.h "$B/codemp/qcommon/"
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" "$B/codemp/"
cp "$ORACLE/codemp/cgame/animtable.h" "$B/codemp/game/"
cp "$ORACLE"/codemp/ghoul2/*.h "$B/codemp/ghoul2/" 2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h  "$B/codemp/cgame/"  2>/dev/null || true
cp "$ORACLE"/codemp/icarus/*.h "$B/codemp/icarus/" 2>/dev/null || true

cp "$G/g_combat.c" "$G/q_shared.c" "$G/q_math.c" "$B/codemp/game/"

# shim.h (force-included first): pull real libm so Raven's 2-arg powf(float,int)
# is renamed out of the way of libm's powf(float,float). Same as run_bgmisc.sh.
cat > "$B/codemp/game/shim.h" <<'EOF'
#include <math.h>
#define powf raven_powf
EOF

CFLAGS="-w -std=gnu11 -fgnu89-inline -D__linux__ -DQAGAME -D_FORTIFY_SOURCE=0 \
        -ffp-contract=off -include $B/codemp/game/shim.h -I. -I $B/codemp/game"

OBJS=""
# shellcheck disable=SC2086
for f in g_combat q_shared q_math; do
	cc $CFLAGS -c "$B/codemp/game/$f.c" -o "$B/$f.o"
	OBJS="$OBJS $B/$f.o"
done
# shellcheck disable=SC2086
cc $CFLAGS -c main_gcombat.c -o "$B/main.o"
# stubs_gcombat.c is compiled WITHOUT the game headers on purpose (argless
# K&R stubs; the linker binds by symbol name alone).
cc -w -std=gnu11 -c stubs_gcombat.c -o "$B/stubs.o"
# shellcheck disable=SC2086
cc "$B/main.o" $OBJS "$B/stubs.o" -lm -o "$B/gcombat_dump"

mkdir -p "$DATA/golden"
status=0
if [ "${1:-}" = "--regen" ]; then
	"$B/gcombat_dump" "$DATA/fixtures/gcombat" > "$DATA/golden/gcombat.txt"
	echo "regenerated gcombat"
else
	"$B/gcombat_dump" "$DATA/fixtures/gcombat" | diff -u "$DATA/golden/gcombat.txt" - || status=1
fi

[ "$status" -eq 0 ] && echo "jampgame-oracle gcombat: OK"
exit "$status"
