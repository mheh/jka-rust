#!/bin/sh
# roff-oracle — differential golden harness for the CROFFSystem port
# (docs/subsystems/roff.md § Verification strategy). Compiles the UNMODIFIED
# oracle codemp/qcommon/RoffSystem.cpp standalone against stub headers, drives it
# over hand-authored minimal .rof fixtures with a deterministic host, and checks
# (or, with --regen, regenerates) the committed goldens. See README.md.
#
#   sh build.sh          build, run both dumpers, diff against goldens/
#   sh build.sh --regen  regenerate fixtures/* and goldens/*
#
# oracle/ is never edited. RoffSystem.{cpp,h} are COPIED into build/ next to the
# stub headers so their relative #includes resolve to the stubs.
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-g++-16}"
ORACLE=../../oracle
QC=$ORACLE/codemp/qcommon
REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

# Oracle-parity flags shared with tools/gp2-oracle / trmodel-oracle / icarus-
# oracle: signed char, no FP contraction / fast-math. Plus the WinDed DEDICATED
# Release macro set this port models (ROFF-D3): -DDEDICATED (the #ifndef DEDICATED
# client branches compile OUT) and -DNDEBUG (the #ifdef _DEBUG Com_Printf lines
# compile OUT — the shipped Release build). FINAL_BUILD undefined.
FLAGS="-std=c++14 -w -fsigned-char -ffp-contract=off -fno-fast-math -DDEDICATED -DNDEBUG"

rm -rf build
mkdir -p build/codemp/qcommon build/codemp/game build/codemp/server build/codemp/client

# Unmodified oracle TU + its header.
cp "$QC/RoffSystem.cpp" "$QC/RoffSystem.h" build/codemp/qcommon/
# Stub headers placed where the relative #includes resolve.
cp stubs/qcommon/exe_headers.h stubs/qcommon/qcommon.h build/codemp/qcommon/
cp stubs/game/q_shared.h                               build/codemp/game/
cp stubs/server/server.h                               build/codemp/server/
cp stubs/client/client.h                               build/codemp/client/

INC="-Ibuild/codemp/qcommon -Ibuild/codemp -I."

echo "building roffgen (fixture generator)..."
$CXX -std=c++14 -w -o build/roffgen roffgen.cpp

if [ "$REGEN" -eq 1 ]; then
	mkdir -p fixtures/scripts
	echo "generating fixtures..."
	build/roffgen
fi

echo "compiling oracle TU + host..."
$CXX $FLAGS $INC -c build/codemp/qcommon/RoffSystem.cpp -o build/RoffSystem.o
$CXX $FLAGS $INC -c host.cpp                            -o build/host.o

echo "building dumpers..."
$CXX $FLAGS $INC -o build/dump_cache dump_cache.cpp build/RoffSystem.o build/host.o
$CXX $FLAGS $INC -o build/dump_play  dump_play.cpp  build/RoffSystem.o build/host.o

run_or_check() { # $1 = golden basename ; rest = command
	name="$1"; shift
	if [ "$REGEN" -eq 1 ]; then
		"$@" > "goldens/$name"
		echo "  regenerated goldens/$name ($(wc -c < goldens/$name) bytes)"
	else
		# Determinism guard: two independent runs must be byte-identical (compare
		# files directly so trailing newlines are preserved).
		"$@" > "build/out1.$name"
		"$@" > "build/out2.$name"
		cmp -s "build/out1.$name" "build/out2.$name" || { echo "NONDETERMINISTIC: $name"; STATUS=1; }
		diff -u "goldens/$name" "build/out1.$name" || { echo "MISMATCH: $name"; STATUS=1; }
	fi
}

[ "$REGEN" -eq 1 ] && mkdir -p goldens
STATUS=0
run_or_check cache.txt build/dump_cache
run_or_check play.txt  build/dump_play

[ "$STATUS" -eq 0 ] && echo "roff-oracle: OK"
exit "$STATUS"
