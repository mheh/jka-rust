#!/bin/sh
# Build the jampgame q_shared oracle dumper from the UNMODIFIED Raven
# codemp/game/q_shared.c and check (or, with --regen, regenerate) the golden
# dump golden/qshared.txt.
#
# Self-contained: uses its OWN build dir (build-qshared/) and does not touch
# run.sh or build/. The oracle q_shared.c + the full game/qcommon header
# closure (plus the two namespace shims) are copied into build-qshared/ so the
# copied source's relative #includes resolve; oracle/ itself is never touched.
#
# Compiled for the plain native target (no Q3_VM, no QAGAME) exactly like the
# bglib slice: q_shared.c is stateless support code needing only q_shared.h +
# libc + this TU's Com_Error/Com_Printf stubs. -D__linux__ selects q_shared.h's
# clean C branches; -ffp-contract=off keeps float eval non-contracted (matches
# Rust); shim.h renames Raven's 2-arg powf out of libm's way.
set -eu
cd "$(dirname "$0")"

ORACLE=../../oracle/oracle
G=$ORACLE/codemp/game
Q=$ORACLE/codemp/qcommon

rm -rf build-qshared
mkdir -p build-qshared/codemp/game build-qshared/codemp/qcommon

cp "$G/q_shared.c" build-qshared/codemp/game/
cp "$G"/*.h build-qshared/codemp/game/
cp "$Q"/*.h build-qshared/codemp/qcommon/
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" build-qshared/codemp/

# shim.h (force-included first): pull real libm so Raven's 2-arg powf(float,int)
# in q_shared.h is renamed out of the way of libm's powf(float,float).
cat > build-qshared/codemp/game/shim.h <<'EOF'
#include <math.h>
#define powf raven_powf
EOF

CFLAGS="-w -std=gnu11 -D__linux__ -D_FORTIFY_SOURCE=0 -ffp-contract=off \
        -include build-qshared/codemp/game/shim.h -I. -I build-qshared/codemp/game"

# shellcheck disable=SC2086
cc $CFLAGS -o build-qshared/qshared_dump main_qshared.c build-qshared/codemp/game/q_shared.c

mkdir -p golden
status=0
if [ "${1:-}" = "--regen" ]; then
	build-qshared/qshared_dump fixtures > golden/qshared.txt
	echo "regenerated qshared"
else
	build-qshared/qshared_dump fixtures | diff -u golden/qshared.txt - || status=1
fi

[ "$status" -eq 0 ] && echo "jampgame-oracle qshared: OK"
exit "$status"
