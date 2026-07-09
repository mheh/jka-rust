#!/bin/sh
# Build the jampgame "wsaber" oracle dumper from the UNMODIFIED Raven sources
# and check (or, with --regen, regenerate) the golden dump
# crates/mp/game/tests/oracle/golden/wsaber.txt.
#
# The dumper drives TWO pure integer leaf functions of w_saber.c —
# G_SaberLockAnim and G_KnockawayForParry — linked against the UNMODIFIED
# q_shared.c + q_math.c. w_saber.c is a large TU that extern-references ~130
# game/engine symbols across functions the dumper never calls; main_wsaber.c
# defines the zeroed data globals it reads, and stubs_wsaber.c (compiled WITHOUT
# the game headers so its argless abort() stubs never clash with the real
# prototypes) satisfies every unreachable function. Compiled with -DQAGAME so
# w_saber.c pulls the full g_local.h + ghoul2/cgame/icarus header closure. Own
# build dir build-wsaber/; oracle/ is never touched.
set -eu
cd "$(dirname "$0")"

# Committed parity data (fixtures + goldens) lives inside the mp_game crate so
# the crate is self-contained; this harness only generates/checks it.
DATA=../../crates/mp/game/tests/oracle

ORACLE=../../oracle
G=$ORACLE/codemp/game
Q=$ORACLE/codemp/qcommon
B=build-wsaber

rm -rf "$B"
mkdir -p "$B/codemp/game" "$B/codemp/qcommon" "$B/codemp/ghoul2" \
         "$B/codemp/cgame" "$B/codemp/icarus"

cp "$G"/*.h "$B/codemp/game/"
cp "$Q"/*.h "$B/codemp/qcommon/"
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" "$B/codemp/"
cp "$ORACLE/codemp/cgame/animtable.h" "$B/codemp/game/"
cp "$ORACLE"/codemp/ghoul2/*.h "$B/codemp/ghoul2/" 2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h  "$B/codemp/cgame/"  2>/dev/null || true
cp "$ORACLE"/codemp/icarus/*.h "$B/codemp/icarus/" 2>/dev/null || true

cp "$G/w_saber.c" "$G/q_shared.c" "$G/q_math.c" "$B/codemp/game/"

cat > "$B/codemp/game/shim.h" <<'EOF'
#include <math.h>
#define powf raven_powf
EOF

CFLAGS="-w -std=gnu11 -fgnu89-inline -D__linux__ -DQAGAME -D_FORTIFY_SOURCE=0 \
        -ffp-contract=off -include $B/codemp/game/shim.h -I. -I $B/codemp/game"

OBJS=""
# shellcheck disable=SC2086
for f in w_saber q_shared q_math; do
	cc $CFLAGS -c "$B/codemp/game/$f.c" -o "$B/$f.o"
	OBJS="$OBJS $B/$f.o"
done
# shellcheck disable=SC2086
cc $CFLAGS -c main_wsaber.c -o "$B/main.o"
# stubs_wsaber.c is compiled WITHOUT the game headers on purpose (argless K&R
# stubs; the linker binds by symbol name alone).
cc -w -std=gnu11 -c stubs_wsaber.c -o "$B/stubs.o"
# shellcheck disable=SC2086
cc "$B/main.o" $OBJS "$B/stubs.o" -lm -o "$B/wsaber_dump"

mkdir -p "$DATA/golden"
status=0
if [ "${1:-}" = "--regen" ]; then
	"$B/wsaber_dump" "$DATA/fixtures/wsaber" > "$DATA/golden/wsaber.txt"
	echo "regenerated wsaber"
else
	"$B/wsaber_dump" "$DATA/fixtures/wsaber" | diff -u "$DATA/golden/wsaber.txt" - || status=1
fi

[ "$status" -eq 0 ] && echo "jampgame-oracle wsaber: OK"
exit "$status"
