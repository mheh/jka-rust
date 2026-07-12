#!/bin/sh
# Build the GP2 oracle dumpers from the UNMODIFIED Raven sources and check
# (or, with --regen, regenerate) the golden dumps under golden/.
#
# The oracle .cpp files are copied into build/ next to stub headers so their
# relative #includes resolve to the stubs; oracle/ itself is never touched.
set -eu
cd "$(dirname "$0")"

ORACLE=../../oracle

rm -rf build
mkdir -p build/mp/qcommon build/sp/game build/sp/qcommon

cp "$ORACLE/codemp/qcommon/GenericParser2.cpp" "$ORACLE/codemp/qcommon/GenericParser2.h" build/mp/qcommon/
cp stubs/mp/qcommon/* build/mp/qcommon/
cp "$ORACLE/code/game/genericparser2.cpp" "$ORACLE/code/game/genericparser2.h" build/sp/game/
cp stubs/sp/game/* build/sp/game/
cp stubs/sp/qcommon/* build/sp/qcommon/

c++ -std=c++14 -w -I build/mp/qcommon -o build/gp2_dump_mp main.cpp build/mp/qcommon/GenericParser2.cpp
c++ -std=c++14 -w -DGP2_SP -D_JK2EXE -I build/sp/game -o build/gp2_dump_sp main.cpp build/sp/game/genericparser2.cpp

mkdir -p golden
status=0
for f in fixtures/*.gp2; do
	b=$(basename "$f" .gp2)
	if [ "${1:-}" = "--regen" ]; then
		build/gp2_dump_mp "$f" >"golden/$b.mp.txt"
		build/gp2_dump_sp "$f" >"golden/$b.sp.txt"
		echo "regenerated $b"
	else
		build/gp2_dump_mp "$f" | diff -u "golden/$b.mp.txt" - || status=1
		build/gp2_dump_sp "$f" | diff -u "golden/$b.sp.txt" - || status=1
	fi
done

[ "$status" -eq 0 ] && echo "gp2-oracle: OK"
exit "$status"
