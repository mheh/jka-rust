#!/bin/sh
# trmodel-oracle — differential golden harness for the tr_model loader + model
# cache port and the matcomp codec (docs/subsystems/tr-model.md § Verification
# strategy). Compiles the UNMODIFIED oracle codemp/renderer/tr_model.cpp +
# matcomp.c standalone against stub headers, drives them over hand-authored
# minimal .glm/.gla fixtures with a deterministic host, and checks (or
# regenerates) the committed goldens. See README.md.
#
#   sh build.sh          build, run all modes, diff dumps against goldens/
#   sh build.sh --regen  regenerate fixtures/* and goldens/*
#
# oracle/ is never edited. The oracle .cpp/.h are COPIED into build/ so their
# relative #includes resolve to the stub headers; no source normalizations are
# applied (both TUs compile clean under the stubs on this host — see README).
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-g++-16}"
ORACLE=../../oracle
RN=$ORACLE/codemp/renderer
REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

# Oracle-parity flags shared with tools/gp2-oracle / icarus-oracle / stringed-
# oracle: signed char, no FP contraction / fast-math. Plus the WinDed DEDICATED
# Release macro set this port models (docs/subsystems/tr-model.md § Raven ground
# truth): -DDEDICATED (headless model loader; GetRefAPI exports only RE_Shutdown),
# -DNDEBUG (asserts no-op — faithful fall-through), and -D_M_IX86 (the shipped x86
# target: every `#ifndef _M_IX86` big-endian swap block compiles OUT, so the
# LittleLong/Short/Float write-backs are identity on this LE host, TRM-D3).
# -fpermissive downgrades the one MSVC/LP64-ism to a warning: the surf-hierarchy
# stride `(int)(&((mdxmSurfHierarchy_t*)0)->childIndexes[n])` casts a pointer to
# int — exact on the 32-bit ship (pointer==int width, ruling 44); the computed
# offset is small so the value is identical here. oracle/ is untouched.
FLAGS="-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math -DDEDICATED -DNDEBUG -D_M_IX86"

rm -rf build
mkdir -p build/codemp/game build/codemp/qcommon build/codemp/renderer

# Unmodified oracle TUs + the two self-contained oracle headers the port parses.
cp "$RN/tr_model.cpp" "$RN/matcomp.c" "$RN/matcomp.h" "$RN/mdx_format.h" build/codemp/renderer/
cp "$ORACLE/codemp/qcommon/sstring.h"                                    build/codemp/qcommon/
# Harness stub headers, placed where tr_model.cpp's relative #includes resolve.
cp stubs/game/q_shared.h            build/codemp/game/
cp stubs/qcommon/qcommon.h          build/codemp/qcommon/
cp stubs/qcommon/exe_headers.h      build/codemp/qcommon/
cp stubs/qcommon/disablewarnings.h  build/codemp/qcommon/
cp stubs/renderer/tr_local.h        build/codemp/renderer/

INC="-Ibuild/codemp/renderer -Ibuild/codemp/qcommon -Ibuild/codemp/game -Istubs/compat -I."

echo "building modelgen (fixture generator)..."
$CXX -std=c++14 -w -o build/modelgen modelgen.cpp

if [ "$REGEN" -eq 1 ]; then
	mkdir -p fixtures/models fixtures/skeletons
	echo "generating fixtures..."
	build/modelgen
fi

echo "compiling oracle TUs..."
$CXX $FLAGS $INC -c build/codemp/renderer/tr_model.cpp -o build/tr_model.o
$CXX $FLAGS $INC -x c++ -c build/codemp/renderer/matcomp.c -o build/matcomp.o
$CXX $FLAGS $INC -c host.cpp -o build/host.o

echo "building dumpers..."
$CXX $FLAGS $INC -o build/dump_load    dump_load.cpp    build/tr_model.o build/host.o
$CXX $FLAGS $INC -o build/dump_cache   dump_cache.cpp   build/tr_model.o build/host.o
$CXX $FLAGS $INC -o build/dump_matcomp dump_matcomp.cpp build/matcomp.o

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
run_or_check load.txt              build/dump_load
run_or_check cache_hitmiss.txt     build/dump_cache hitmiss
run_or_check cache_evict.txt       build/dump_cache evict
run_or_check cache_dumpnonpure.txt build/dump_cache dumpnonpure
run_or_check matcomp.txt           build/dump_matcomp

[ "$STATUS" -eq 0 ] && echo "trmodel-oracle: OK"
exit "$STATUS"
