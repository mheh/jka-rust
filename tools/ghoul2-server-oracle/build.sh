#!/bin/sh
# ghoul2-server-oracle — differential golden harness for the mp_engine_ghoul2
# server-side bone/arena pipeline port (docs/subsystems/ghoul2-server.md, FROZEN,
# § Verification strategy). Compiles the UNMODIFIED oracle ghoul2 server TUs
# (codemp/ghoul2/G2_API.cpp, G2_bolts.cpp, G2_surfaces.cpp) standalone against
# stub headers, drives them over hand-authored, model-free fixtures, and checks
# (or regenerates) the committed goldens. See README.md.
#
#   sh build.sh          build, run all modes, diff dumps against goldens/
#   sh build.sh --regen  regenerate goldens/*
#
# oracle/ is never edited. The oracle .cpp/.h are COPIED into build/ so their
# relative #includes resolve to the stub headers; no source normalizations are
# applied (all three TUs compile clean under the stubs on this host — README).
#
# SCOPE: this harness covers the model-free, standalone-compilable islands of the
# subsystem — the arena/handle scheme, the bolt-list bookkeeping, and the
# generated-surface list. The bone-transform / bolt-matrix / collision / ragdoll
# / gore goldens named in the doc require the full renderer + collision + model-
# memory closure (tr_ghoul2.cpp, G2_bones.cpp, G2_misc.cpp) and are NOT yet
# standalone — see README § Uncovered (gaps). No silent coverage is claimed.
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-g++-16}"
ORACLE=../../oracle
G2=$ORACLE/codemp/ghoul2
RN=$ORACLE/codemp/renderer
QC=$ORACLE/codemp/qcommon
REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

# Oracle-parity flags shared with tools/gp2-oracle / icarus-oracle / trmodel-
# oracle: signed char, no FP contraction / fast-math. Plus the WinDed DEDICATED
# Release macro set this port models (docs/subsystems/ghoul2-server.md § Raven
# ground truth): -DDEDICATED, -DNDEBUG (asserts no-op), -D_M_IX86 (shipped x86
# target — the #ifndef _M_IX86 big-endian swaps compile OUT), -D_G2_GORE (ON in
# MP, q_shared.h:3110). -fpermissive downgrades the LP64 pointer-width warnings
# (exact on the 32-bit ship, never dumped). oracle/ is untouched.
FLAGS="-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math -DDEDICATED -DNDEBUG -D_M_IX86 -D_G2_GORE"

rm -rf build
mkdir -p build/codemp/ghoul2 build/codemp/qcommon build/codemp/game build/codemp/renderer

# Unmodified oracle TUs + the self-contained oracle headers they parse.
cp "$G2/G2_API.cpp" "$G2/G2_bolts.cpp" "$G2/G2_surfaces.cpp" build/codemp/ghoul2/
cp "$G2/G2_local.h" "$G2/ghoul2_shared.h" "$G2/G2.h" "$G2/G2_gore.h" build/codemp/ghoul2/
cp "$RN/mdx_format.h"  build/codemp/renderer/
cp "$QC/MiniHeap.h"    build/codemp/qcommon/
# Harness stub headers, placed where the TUs' relative #includes resolve.
cp stubs/game/q_shared.h            build/codemp/game/
cp stubs/qcommon/qcommon.h          build/codemp/qcommon/
cp stubs/qcommon/exe_headers.h      build/codemp/qcommon/
cp stubs/qcommon/disablewarnings.h  build/codemp/qcommon/
cp stubs/renderer/tr_local.h        build/codemp/renderer/

INC="-Ibuild/codemp/ghoul2 -Ibuild/codemp/renderer -Ibuild/codemp/qcommon -Ibuild/codemp/game -I. -Ibuild"

echo "compiling oracle TUs..."
$CXX $FLAGS $INC -c build/codemp/ghoul2/G2_API.cpp      -o build/G2_API.o
$CXX $FLAGS $INC -c build/codemp/ghoul2/G2_bolts.cpp    -o build/G2_bolts.o
$CXX $FLAGS $INC -c build/codemp/ghoul2/G2_surfaces.cpp -o build/G2_surfaces.o
$CXX $FLAGS $INC -c host.cpp                            -o build/host.o

echo "building dumpers..."
# -Wl,-dead_strip drops every function the dumper never reaches, so only the
# live-path engine-seam symbols (host.cpp) need a body.
$CXX $FLAGS $INC -Wl,-dead_strip -o build/dump_arena    dump_arena.cpp    build/G2_API.o      build/host.o
$CXX $FLAGS $INC -Wl,-dead_strip -o build/dump_bolts    dump_bolts.cpp    build/G2_bolts.o build/G2_surfaces.o build/host.o
$CXX $FLAGS $INC -Wl,-dead_strip -o build/dump_surfaces dump_surfaces.cpp build/G2_surfaces.o build/host.o

run_or_check() { # $1 = golden basename ; rest = command
	name="$1"; shift
	if [ "$REGEN" -eq 1 ]; then
		"$@" > "goldens/$name"
		echo "  regenerated goldens/$name ($(wc -c < goldens/$name) bytes)"
	else
		"$@" | diff -u "goldens/$name" - || { echo "MISMATCH: $name"; STATUS=1; }
	fi
}

[ "$REGEN" -eq 1 ] && mkdir -p goldens
STATUS=0
run_or_check arena.txt    build/dump_arena
run_or_check bolts.txt    build/dump_bolts
run_or_check surfaces.txt build/dump_surfaces

[ "$STATUS" -eq 0 ] && echo "ghoul2-server-oracle: OK"
exit "$STATUS"
