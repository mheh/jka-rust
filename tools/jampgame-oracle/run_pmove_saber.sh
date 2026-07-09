#!/bin/sh
# Build the pmove SABER differential dumper from the UNMODIFIED Raven sources and
# check (or, with --regen, regenerate)
# crates/mp/game/tests/oracle/golden/pmove_saber.txt.
#
# This is a self-contained sibling of run.sh's pmove slice: it uses its OWN build
# directory (build-pmove-saber/) and touches nothing run.sh owns. It follows the
# pmove build model VERBATIM -- -D__linux__ -DQAGAME -ffp-contract=off
# -fgnu89-inline, the q_math holdrand RNG rename, and the same UNMODIFIED TU link
# list (bg_pmove bg_slidemove bg_panimate bg_saber bg_saberLoad bg_misc bg_weapons
# q_shared + q_math with renamed RNG) -- but links main_pmove_saber.c instead of
# main_pmove.c. oracle/ is never edited (sources are copied into build-pmove-saber/).
set -eu
cd "$(dirname "$0")"

# Committed parity data (fixtures + goldens) lives inside the mp_game crate so
# the crate is self-contained; this harness only generates/checks it.
DATA=../../crates/mp/game/tests/oracle

ORACLE=../../oracle
G=$ORACLE/codemp/game
Q=$ORACLE/codemp/qcommon
B=build-pmove-saber

rm -rf "$B"
mkdir -p "$B/codemp/game" "$B/codemp/qcommon" "$B/codemp/ghoul2" \
         "$B/codemp/cgame" "$B/codemp/icarus"

# Full game/qcommon header + source closure (all copied UNMODIFIED), plus the
# ghoul2/cgame/icarus header trees the -DQAGAME closure pulls in.
cp "$G"/*.h "$B/codemp/game/"
cp "$Q"/*.h "$B/codemp/qcommon/"
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" "$B/codemp/"
cp "$ORACLE/codemp/cgame/animtable.h" "$B/codemp/game/"
cp "$G/bg_pmove.c" "$G/bg_slidemove.c" "$G/bg_panimate.c" "$G/bg_saber.c" \
   "$G/bg_saberLoad.c" "$G/bg_misc.c" "$G/bg_weapons.c" "$G/q_shared.c" \
   "$G/q_math.c" "$B/codemp/game/"
cp "$ORACLE"/codemp/ghoul2/*.h "$B/codemp/ghoul2/" 2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h  "$B/codemp/cgame/"  2>/dev/null || true
cp "$ORACLE"/codemp/icarus/*.h "$B/codemp/icarus/" 2>/dev/null || true

# shim.h (force-included first): pull real libm so Raven's 2-arg powf(float,int)
# is renamed out of the way of libm's powf(float,float).
cat > "$B/codemp/game/shim.h" <<'EOF'
#include <math.h>
#define powf raven_powf
EOF

PMCFLAGS="-w -std=gnu11 -fgnu89-inline -D__linux__ -DQAGAME -D_FORTIFY_SOURCE=0 \
        -ffp-contract=off -include $B/codemp/game/shim.h -I. -I $B/codemp/game"
# q_math.c recompiled with its holdrand RNG functions RENAMED so
# main_pmove_saber.c's own 32-bit Q_irand + draw-counter tripwire wins the link.
PM_RNG_RENAME="-DRand_Init=o_Rand_Init -Dflrand=o_flrand -DQ_flrand=o_Q_flrand \
        -Dirand=o_irand -DQ_irand=o_Q_irand"

PMOBJS=""
# shellcheck disable=SC2086
for src in bg_pmove bg_slidemove bg_panimate bg_saber bg_saberLoad bg_misc bg_weapons q_shared; do
	cc $PMCFLAGS -c "$B/codemp/game/$src.c" -o "$B/pm_$src.o"
	PMOBJS="$PMOBJS $B/pm_$src.o"
done
# shellcheck disable=SC2086
cc $PMCFLAGS $PM_RNG_RENAME -c "$B/codemp/game/q_math.c" -o "$B/pm_q_math.o"
# shellcheck disable=SC2086
cc $PMCFLAGS -c main_pmove_saber.c -o "$B/pm_main.o"
# shellcheck disable=SC2086
cc "$B/pm_main.o" $PMOBJS "$B/pm_q_math.o" -lm -o "$B/pmove_saber_dump"

mkdir -p "$DATA/golden"
status=0

# The dumper runs over all saber scenarios, concatenated (with a per-scenario
# banner) into one golden. It takes <fixture-file> <fixture-dir>; the dir holds
# the shared synthetic animation.cfg.
SABER_SCENARIOS="saber-idle saber-walk saber-attack-stand saber-attack-run saber-attack-strafe saber-jump"
out=$( for s in $SABER_SCENARIOS; do
	echo "-- scenario $s --"
	"$B/pmove_saber_dump" "$DATA/fixtures/pmove_saber/$s.txt" "$DATA/fixtures/pmove_saber"
done )

if [ "${1:-}" = "--regen" ]; then
	printf '%s\n' "$out" > "$DATA/golden/pmove_saber.txt"
	echo "regenerated pmove_saber"
else
	printf '%s\n' "$out" | diff -u "$DATA/golden/pmove_saber.txt" - || status=1
	[ "$status" -eq 0 ] && echo "pmove-saber-oracle: OK"
fi

exit "$status"
