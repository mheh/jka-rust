#!/bin/sh
# Build the jampgame q_math + bg_lib oracle dumpers from the UNMODIFIED Raven
# sources and check (or, with --regen, regenerate) the golden dumps under
# golden/.
#
# The oracle .c files and their real header chain (q_shared.h + teams.h +
# bg_lib.h + surfaceflags.h + ../qcommon/{disablewarnings,tags}.h) are copied
# into build/ so their relative #includes resolve; oracle/ itself is never
# touched. Two functions are EXTRACTED verbatim from the copies into their own
# build files because they are otherwise unreachable on a native (no-Q3_VM)
# compile — see the heredocs below and README.md.
set -eu
cd "$(dirname "$0")"

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

mkdir -p golden
status=0
run_one() {
	name=$1; bin=$2
	if [ "${REGEN:-}" = "1" ]; then
		"$bin" fixtures > "golden/$name.txt"
		echo "regenerated $name"
	else
		"$bin" fixtures | diff -u "golden/$name.txt" - || status=1
	fi
}

if [ "${1:-}" = "--regen" ]; then REGEN=1; fi
run_one qmath build/qmath_dump
run_one bglib build/bglib_dump
run_one saberload build/saberload_dump

[ "$status" -eq 0 ] && echo "jampgame-oracle: OK"
exit "$status"
