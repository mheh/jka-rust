#!/bin/sh
# Build the terrainmap differential-oracle dumper from the UNMODIFIED Raven TUs
# and check (or, with --regen, regenerate) the committed golden dumps under
# golden/.
#
# The oracle .cpp/.h files are copied into build/ mirroring the codemp/ tree so
# their relative #includes (../qcommon/, ../png/) resolve to the stub headers in
# stubs/; oracle/ itself is never touched (porting-rules §F18).
#
# Compiled -DNDEBUG: Raven's asserts vanish in a release build, and the Rust
# port omits them, so the harness must too.
set -eu
cd "$(dirname "$0")"
HERE=$(pwd)

CXX=${CXX:-g++-16}
ORACLE=../../oracle/codemp

rm -rf build
mkdir -p build/codemp/qcommon build/codemp/png

# --- unmodified oracle TUs + the real headers under test ---
cp "$ORACLE/qcommon/cm_draw.cpp"       \
   "$ORACLE/qcommon/cm_draw.h"         \
   "$ORACLE/qcommon/cm_terrainmap.cpp" \
   "$ORACLE/qcommon/cm_terrainmap.h"   build/codemp/qcommon/

# MSVC functional-cast syntax: `unsigned short (expr)` is not valid C++ and only
# MSVC accepts it. Rewrite the two sites in the BUILD COPY to the C cast, which
# is the same conversion. Syntax only, no behavior change.
# Source: oracle/codemp/qcommon/cm_draw.cpp:553,585
sed -i '' 's/= unsigned short /= (unsigned short)/' build/codemp/qcommon/cm_draw.cpp
[ "$(grep -c '= (unsigned short)' build/codemp/qcommon/cm_draw.cpp)" = "2" ] || {
	echo "unsigned-short cast patch failed"
	exit 1
}

# --- stub headers (oracle never edited) ---
cp stubs/qcommon/* build/codemp/qcommon/
cp stubs/png/*     build/codemp/png/

INC="-I build/codemp/qcommon"
# oracle-parity flags (gp2-oracle / rmg-oracle precedent): C++14, warnings
# silenced (the unmodified Raven sources are not warning-clean).
FLAGS="-std=c++14 -w -O1 -DNDEBUG -DFIXTURE_ROOT=\"$HERE/fixtures\""

# shellcheck disable=SC2086
$CXX $FLAGS $INC -o build/terrainmap_dump \
	main.cpp \
	src/host_stubs.cpp \
	build/codemp/qcommon/cm_draw.cpp \
	build/codemp/qcommon/cm_terrainmap.cpp

mkdir -p golden
status=0
for mode in draw terrainmap; do
	if [ "${1:-}" = "--regen" ]; then
		build/terrainmap_dump "$mode" >"golden/$mode.txt"
		echo "regenerated $mode"
	else
		build/terrainmap_dump "$mode" | diff -u "golden/$mode.txt" - || status=1
	fi
done

[ "$status" -eq 0 ] && echo "terrainmap-oracle: OK"
exit "$status"
