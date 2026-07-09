#!/bin/sh
# Build the jampgame q_math + bg_lib oracle dumpers from the UNMODIFIED Raven
# sources and check (or, with --regen, regenerate) the golden dumps under
# crates/mp/game/tests/oracle/golden/.
#
# The oracle .c files and their real header chain (q_shared.h + teams.h +
# bg_lib.h + surfaceflags.h + ../qcommon/{disablewarnings,tags}.h) are copied
# into build/ so their relative #includes resolve; oracle/ itself is never
# touched. Two functions are EXTRACTED verbatim from the copies into their own
# build files because they are otherwise unreachable on a native (no-Q3_VM)
# compile — see the heredocs below and README.md.
set -eu
cd "$(dirname "$0")"

# Committed parity data (fixtures + goldens) lives inside the mp_game crate so
# the crate is self-contained; this harness only generates/checks it.
DATA=../../crates/mp/game/tests/oracle

ORACLE=../../oracle/oracle
G=$ORACLE/codemp/game
Q=$ORACLE/codemp/qcommon

rm -rf build
mkdir -p build/codemp/game build/codemp/qcommon

cp "$G/q_math.c" "$G/bg_lib.c" "$G/q_shared.h" "$G/teams.h" "$G/bg_lib.h" \
   "$G/surfaceflags.h" build/codemp/game/
cp "$Q/disablewarnings.h" "$Q/tags.h" build/codemp/qcommon/

# --- bg_saberLoad slice extra sources/headers (all copied UNMODIFIED) ---
# The saberLoad dumper compiles the whole bg_saberLoad.c + q_shared.c (the
# COM_* parser + string tables) against the real bg header chain, so copy the
# full game/qcommon header closure plus the two namespace shims and the
# animtable.h that defines the animTable symbol.
cp "$G/bg_saberLoad.c" "$G/q_shared.c" build/codemp/game/
cp "$G"/*.h build/codemp/game/
cp "$Q"/*.h build/codemp/qcommon/
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" build/codemp/
cp "$ORACLE/codemp/cgame/animtable.h" build/codemp/game/

# --- pmove slice extra sources/headers (all copied UNMODIFIED) ---
# The pmove dumper compiles the on-foot movement closure with -DQAGAME, which
# pulls g_local.h + ghoul2/G2.h, so it needs the ghoul2/cgame/icarus header
# trees copied alongside game/qcommon.
mkdir -p build/codemp/ghoul2 build/codemp/cgame build/codemp/icarus
cp "$G/bg_pmove.c" "$G/bg_slidemove.c" "$G/bg_panimate.c" "$G/bg_saber.c" \
   "$G/bg_misc.c" "$G/bg_weapons.c" build/codemp/game/
cp "$ORACLE"/codemp/ghoul2/*.h build/codemp/ghoul2/ 2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h  build/codemp/cgame/  2>/dev/null || true
cp "$ORACLE"/codemp/icarus/*.h build/codemp/icarus/ 2>/dev/null || true

# shim.h (force-included first): pull real libm so Raven's 2-arg powf(float,int)
# in q_shared.h/q_math.c is renamed out of the way of libm's powf(float,float).
cat > build/codemp/game/shim.h <<'EOF'
#include <math.h>
#define powf raven_powf
EOF

# raven_rng.c: Raven's holdrand LCG (q_math.c:1432-1474) extracted verbatim,
# with `unsigned long holdrand` normalized to `unsigned int` (the 32-bit i686
# ship target the port models; on this LP64 host `unsigned long` is 64-bit and
# `>>17` would diverge). Functions renamed r_* so they don't clash with the
# 64-bit copies still living in q_math.c.
{
	echo '#include <assert.h>'
	sed -n '1432,1474p' build/codemp/game/q_math.c \
	  | sed -E -e 's/unsigned long[[:space:]]+holdrand/unsigned int holdrand/' \
	           -e 's/[[:<:]]Rand_Init[[:>:]]/r_Rand_Init/g' \
	           -e 's/[[:<:]]Q_flrand[[:>:]]/r_Q_flrand/g' \
	           -e 's/[[:<:]]Q_irand[[:>:]]/r_Q_irand/g' \
	           -e 's/[[:<:]]flrand[[:>:]]/r_flrand/g' \
	           -e 's/[[:<:]]irand[[:>:]]/r_irand/g'
} > build/codemp/game/raven_rng.c

# Q_rsqrt (q_math.c:616-636) reads a float through `*(long*)&y`; on this LP64
# host `long` is 64-bit so it reads 4 bytes past the float (UB) and `>>1`
# diverges from the 32-bit ship target. Normalize long->int (exactly the port's
# i32 model) and drop the __linux__ isnan assert; rename r_Q_rsqrt.
sed -n '616,636p' build/codemp/game/q_math.c \
  | sed -E -e 's/[[:<:]]Q_rsqrt[[:>:]]/r_Q_rsqrt/g' \
           -e 's/long( |\t)*i;/int i;/' \
           -e 's/\* \( long \* \)/* ( int * )/' \
           -e '/assert\(/d' \
  >> build/codemp/game/raven_rng.c

# raven_atoi.c: Raven's atoi (bg_lib.c:915-958) is guarded by #if defined(Q3_VM);
# on a native build the linker would otherwise bind libc's atoi (which does not
# do Raven's signed-char >0x7f whitespace skip). Extract it verbatim, renamed.
{
	echo '/* extracted verbatim from oracle bg_lib.c:915-958 (Raven Q3_VM atoi) */'
	sed -n '915,958p' build/codemp/game/bg_lib.c | sed -E 's/int atoi\(/int raven_atoi(/'
} > build/codemp/game/raven_atoi.c

CFLAGS="-w -std=gnu11 -D__linux__ -D_FORTIFY_SOURCE=0 -ffp-contract=off \
        -include build/codemp/game/shim.h -I. -I build/codemp/game"

# shellcheck disable=SC2086
cc $CFLAGS -o build/qmath_dump main_qmath.c \
   build/codemp/game/q_math.c build/codemp/game/raven_rng.c
# shellcheck disable=SC2086
cc $CFLAGS -o build/bglib_dump main_bglib.c \
   build/codemp/game/bg_lib.c build/codemp/game/raven_atoi.c

# bg_saberLoad dumper: compiled with -DQAGAME (jampgame == Raven's QAGAME),
# linking the unmodified bg_saberLoad.c + q_shared.c (COM_* parser + string
# tables) + animtable_def.c (defines the animTable symbol). All engine traps
# and the FPTable/G_SoundIndex externs are stubbed in main_saberload.c.
SABERCFLAGS="-w -std=gnu11 -D__linux__ -DQAGAME -D_FORTIFY_SOURCE=0 -ffp-contract=off \
        -include build/codemp/game/shim.h -I. -I build/codemp/game"
# shellcheck disable=SC2086
cc $SABERCFLAGS -o build/saberload_dump main_saberload.c animtable_def.c \
   build/codemp/game/bg_saberLoad.c build/codemp/game/q_shared.c

# --- pmove single-step slice ---------------------------------------------------
# The trace dumper (main_trace.c) proves the pmworld.h axial-brush trace stub in
# isolation; it links nothing else. The pmove dumper (main_pmove.c) links the
# UNMODIFIED on-foot movement closure with -DQAGAME (jampgame == QAGAME) plus
# -fgnu89-inline (Raven's non-static `inline` PM_* helpers need gnu89 external
# inline semantics, else clang emits no out-of-line symbol at -O0 -- this only
# affects symbol emission, never the IEEE math). q_math.c is recompiled here
# with its holdrand RNG functions RENAMED (-DQ_irand=o_Q_irand, etc.) so
# main_pmove.c's own 32-bit Q_irand + draw-counter tripwire wins the link
# without a duplicate symbol (the RNG is never drawn on the basic path).
# animtable.h is compiled INTO bg_panimate.c already, so -- unlike saberload --
# no animtable_def.c is linked (it would duplicate the animTable symbol).
PMCFLAGS="-w -std=gnu11 -fgnu89-inline -D__linux__ -DQAGAME -D_FORTIFY_SOURCE=0 \
        -ffp-contract=off -include build/codemp/game/shim.h -I. -I build/codemp/game"
PM_RNG_RENAME="-DRand_Init=o_Rand_Init -Dflrand=o_flrand -DQ_flrand=o_Q_flrand \
        -Dirand=o_irand -DQ_irand=o_Q_irand"

PMOBJS=""
# shellcheck disable=SC2086
for f in bg_pmove bg_slidemove bg_panimate bg_saber bg_saberLoad bg_misc bg_weapons q_shared; do
	cc $PMCFLAGS -c build/codemp/game/$f.c -o build/pm_$f.o
	PMOBJS="$PMOBJS build/pm_$f.o"
done
# shellcheck disable=SC2086
cc $PMCFLAGS $PM_RNG_RENAME -c build/codemp/game/q_math.c -o build/pm_q_math.o
# shellcheck disable=SC2086
cc $PMCFLAGS -c main_pmove.c -o build/pm_main.o
# shellcheck disable=SC2086
cc build/pm_main.o $PMOBJS build/pm_q_math.o -lm -o build/pmove_dump

# trace dumper: self-contained (only pmworld.h + fixture parsing), plain native.
TRACECFLAGS="-w -std=gnu11 -D__linux__ -D_FORTIFY_SOURCE=0 -ffp-contract=off \
        -include build/codemp/game/shim.h -I. -I build/codemp/game"
# shellcheck disable=SC2086
cc $TRACECFLAGS -o build/trace_dump main_trace.c -lm

mkdir -p "$DATA/golden"
status=0
run_one() {
	name=$1; bin=$2
	if [ "${REGEN:-}" = "1" ]; then
		"$bin" "$DATA/fixtures" > "$DATA/golden/$name.txt"
		echo "regenerated $name"
	else
		"$bin" "$DATA/fixtures" | diff -u "$DATA/golden/$name.txt" - || status=1
	fi
}

# A dumper taking a single fixture FILE argument (trace slice).
run_file() {
	name=$1; bin=$2; fix=$3
	if [ "${REGEN:-}" = "1" ]; then
		"$bin" "$fix" > "$DATA/golden/$name.txt"
		echo "regenerated $name"
	else
		"$bin" "$fix" | diff -u "$DATA/golden/$name.txt" - || status=1
	fi
}

# The pmove dumper runs over all six on-foot scenarios, concatenated (with a
# per-scenario banner) into one golden. It takes <fixture-file> <fixture-dir>;
# the dir holds the shared synthetic animation.cfg.
PMOVE_SCENARIOS="idle walk-fwd strafe-turn jump-land fall-onto-box wall-step"
run_pmove() {
	out=$( for s in $PMOVE_SCENARIOS; do
		echo "-- scenario $s --"
		build/pmove_dump "$DATA/fixtures/pmove/$s.txt" "$DATA/fixtures/pmove"
	done )
	if [ "${REGEN:-}" = "1" ]; then
		printf '%s\n' "$out" > "$DATA/golden/pmove.txt"
		echo "regenerated pmove"
	else
		printf '%s\n' "$out" | diff -u "$DATA/golden/pmove.txt" - || status=1
	fi
}

if [ "${1:-}" = "--regen" ]; then REGEN=1; fi
run_one qmath build/qmath_dump
run_one bglib build/bglib_dump
run_one saberload build/saberload_dump
run_file pmove_trace build/trace_dump "$DATA/fixtures/pmove/trace.txt"
run_pmove

[ "$status" -eq 0 ] && echo "jampgame-oracle: OK"
exit "$status"
