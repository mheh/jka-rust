#!/bin/sh
# Build the RMG differential-oracle dumper from the UNMODIFIED Raven TUs and
# check (or, with --regen, regenerate) the committed golden dumps under golden/.
#
# The oracle .cpp/.h files are copied into build/ mirroring the codemp/ tree so
# their relative #includes (../qcommon/, ../server/) resolve to the stub headers
# in stubs/; oracle/ itself is never touched (§18).
#
# CRITICAL (ruling 25 / RMG-D1): compiled WITH -DDEDICATED so the harness pins
# exactly the dedicated-server behavior — generation dead, LoadMission early-out.
set -eu
cd "$(dirname "$0")"
HERE=$(pwd)

CXX=${CXX:-g++-16}
ORACLE=../../oracle/codemp

rm -rf build
mkdir -p build/codemp/qcommon build/codemp/RMG build/codemp/server

# --- unmodified oracle TUs + the real headers under test ---
cp "$ORACLE/qcommon/cm_terrain.cpp"       \
   "$ORACLE/qcommon/cm_randomterrain.cpp" \
   "$ORACLE/qcommon/GenericParser2.cpp"   \
   "$ORACLE/qcommon/GenericParser2.h"     \
   "$ORACLE/qcommon/cm_landscape.h"       \
   "$ORACLE/qcommon/cm_randomterrain.h"   \
   "$ORACLE/qcommon/cm_patch.h"           build/codemp/qcommon/
cp "$ORACLE/RMG/RM_Manager.cpp" "$ORACLE/RMG/RM_Manager.h" build/codemp/RMG/

# --- stub headers (oracle never edited) ---
cp stubs/qcommon/* build/codemp/qcommon/
cp stubs/RMG/*     build/codemp/RMG/
cp stubs/server/*  build/codemp/server/

INC="-I build/codemp/qcommon -I build/codemp/RMG"
# oracle-parity flags (gp2-oracle precedent): C++14, warnings silenced (the
# unmodified Raven sources are not warning-clean). -DDEDICATED pins RMG-D1.
# -undefined dynamic_lookup leaves the unreferenced generation subtree's externs
# (CreateRandomTerrain's CRandomTerrain::Generate chain, the CM_* area wrappers,
# renderer R_* image loaders) unresolved rather than stubbing them: under
# DEDICATED none is ever CALLED, so they need no definition. (-dead_strip was
# rejected — it corrupts the RMG_CreateSeed static-table relocations.)
FLAGS="-std=c++14 -w -O1 -DDEDICATED -DFIXTURE_ROOT=\"$HERE/fixtures\" -Wl,-undefined,dynamic_lookup"

# shellcheck disable=SC2086
$CXX $FLAGS $INC -o build/rmg_dump \
	main.cpp \
	src/rmg_host_stubs.cpp \
	build/codemp/qcommon/cm_terrain.cpp \
	build/codemp/qcommon/cm_randomterrain.cpp \
	build/codemp/qcommon/GenericParser2.cpp \
	build/codemp/RMG/RM_Manager.cpp

mkdir -p golden
status=0
for mode in seed dedicated; do
	if [ "${1:-}" = "--regen" ]; then
		build/rmg_dump "$mode" >"golden/$mode.txt"
		echo "regenerated $mode"
	else
		build/rmg_dump "$mode" | diff -u "golden/$mode.txt" - || status=1
	fi
done

[ "$status" -eq 0 ] && echo "rmg-oracle: OK"
exit "$status"
