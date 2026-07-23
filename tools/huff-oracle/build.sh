#!/bin/sh
# huff-oracle — differential golden harness for the adaptive-Huffman port
# (crates/mp/engine/qcommon/src/qcommon/huff.rs; porting-rules §F18). Compiles
# the UNMODIFIED oracle codemp/qcommon/huffman.cpp standalone against a minimal
# stub exe_headers.h, seeds the msgHuff tree the way MSG_initHuffman does with
# the LIVE msg_hData table (extracted programmatically from msg.cpp, never the
# stale Quake-3 table at :2696), and dumps per-symbol prefix codes plus two
# concatenated bitstreams. The Rust twin (tests/huff_golden.rs) reproduces the
# committed goldens byte-for-byte, so cargo test needs no C++ toolchain.
#
#   sh build.sh          build, run all dumpers, diff against goldens/
#   sh build.sh --regen  regenerate goldens/*
#
# oracle/ is never edited. huffman.cpp is COPIED into build/ next to the stub
# exe_headers.h so its `#include "../qcommon/exe_headers.h"` resolves to the stub.
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-g++-16}"
PY="${PY:-python3}"
ORACLE=../../oracle
QC=$ORACLE/codemp/qcommon
REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

# Oracle-parity flags shared with the other §F harnesses: signed char, no FP
# contraction / fast-math. The Huffman coder is pure integer/pointer work, but
# we keep the flag set uniform across harnesses.
FLAGS="-std=c++14 -w -fsigned-char -ffp-contract=off -fno-fast-math"

rm -rf build
mkdir -p build/codemp/qcommon

# Unmodified oracle TU.
cp "$QC/huffman.cpp" build/codemp/qcommon/
# Stub header placed where the relative #include resolves.
cp stubs/qcommon/exe_headers.h build/codemp/qcommon/

# Extract the LIVE freq. table into a header the dumper compiles in.
"$PY" extract_table.py "$QC/msg.cpp" build/msg_hdata.h

INC="-Ibuild -Ibuild/codemp/qcommon"

echo "compiling oracle TU + dumper..."
$CXX $FLAGS $INC -c build/codemp/qcommon/huffman.cpp -o build/huffman.o
$CXX $FLAGS $INC -c main.cpp                          -o build/main.o
$CXX $FLAGS $INC -o build/huffdump build/main.o build/huffman.o

run_or_check() { # $1 = golden basename ; $2 = dump kind
	name="$1"; kind="$2"
	if [ "$REGEN" -eq 1 ]; then
		build/huffdump "$kind" "goldens/$name"
		echo "  regenerated goldens/$name ($(wc -c < goldens/$name) bytes)"
	else
		build/huffdump "$kind" "build/out1.$name"
		build/huffdump "$kind" "build/out2.$name"
		cmp -s "build/out1.$name" "build/out2.$name" || { echo "NONDETERMINISTIC: $name"; STATUS=1; }
		diff -u "goldens/$name" "build/out1.$name" || { echo "MISMATCH: $name"; STATUS=1; }
	fi
}

mkdir -p goldens
STATUS=0
run_or_check codes.txt codes
run_or_check seq.txt   seq
run_or_check chat.txt  chat

[ "$STATUS" -eq 0 ] && echo "huff-oracle: OK"
exit "$STATUS"
