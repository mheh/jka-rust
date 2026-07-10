#!/bin/sh
# stringed-oracle — differential golden harness for the StringEd localization
# port (docs/subsystems/stringed.md § Verification strategy). Compiles the
# UNMODIFIED oracle codemp/qcommon/stringed_ingame.cpp + stringed_interface.cpp
# standalone under stub headers, drives them over hand-authored .str/.ste
# fixtures with a deterministic host, and checks (or regenerates) the three
# committed goldens. See README.md.
#
#   sh build.sh          build, run the three modes, diff against goldens/
#   sh build.sh --regen  rebuild and regenerate goldens/*
#
# oracle/ is never edited — the oracle .cpp/.h are COPIED into build/ so their
# relative #includes resolve to the stub headers; no normalizations are applied
# (both TUs compile clean under the stubs on this host).
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-g++-16}"
ORACLE=../../oracle
QC=$ORACLE/codemp/qcommon
REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

# Oracle-parity flags shared with tools/gp2-oracle & tools/icarus-oracle:
# signed char, no FP contraction / fast-math. Plus the WinDed Release macro set
# the ported build models (docs/subsystems/stringed.md appendix): -DNDEBUG makes
# the SE-V3 asserts no-ops (faithful fall-through returns), -DDEDICATED selects
# the dedicated link set; _STRINGED is left UNDEFINED (SE-V1 editor branches out).
FLAGS="-std=c++14 -w -fsigned-char -ffp-contract=off -fno-fast-math -DNDEBUG -DDEDICATED"

rm -rf build
mkdir -p build/codemp/game build/codemp/server build/codemp/qcommon

# unmodified oracle TUs + their in-scope headers
cp "$QC/stringed_ingame.cpp"    "$QC/stringed_ingame.h"    build/codemp/qcommon/
cp "$QC/stringed_interface.cpp" "$QC/stringed_interface.h" build/codemp/qcommon/
# harness stub headers (resolve the qcommon boilerplate #includes)
cp stubs/game/q_shared.h        build/codemp/game/
cp stubs/server/server.h        build/codemp/server/
cp stubs/qcommon/qcommon.h      build/codemp/qcommon/

echo "building dump..."
$CXX $FLAGS -o build/dump dump.cpp host.cpp

run_or_check() { # $1 = golden basename ; $2 = mode
    if [ "$REGEN" -eq 1 ]; then
        build/dump "$2" > "goldens/$1"
        echo "  regenerated goldens/$1"
    else
        build/dump "$2" | diff -u "goldens/$1" - || { echo "MISMATCH: $1"; STATUS=1; }
    fi
}

STATUS=0
run_or_check parse_lookup.txt        parse_lookup
run_or_check reference_stability.txt reference_stability
run_or_check filelist_scan.txt       filelist_scan

[ "$STATUS" -eq 0 ] && echo "stringed-oracle: OK"
exit "$STATUS"
