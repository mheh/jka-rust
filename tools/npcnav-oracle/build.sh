#!/bin/sh
# npcnav-oracle build+run. Compiles the UNMODIFIED oracle navigator.cpp TU from
# copies placed in build/ next to the stub headers (oracle/ is never touched,
# §18), then for each layouts/*.layout either CHECKS the emitted fixture/golden
# against the committed ones (default) or REGENERATES them (--regen).
#
#   sh build.sh            build, diff current output against fixtures/ + goldens/
#   sh build.sh --regen    rebuild fixtures/*.nav and goldens/*.txt
#
# The goldens + .nav fixtures are committed, so the Rust parity tests need no
# C++ toolchain; this script is only for regen / spot-check.
#
# Toolchain: Homebrew g++-16 (libstdc++) — pins the std::push_heap/pop_heap
# equal-cost tie order (NAV-D2 / RULING 45). 4-byte-long shim is in the stub
# header (NAV-D1 / RULING 44); -ftrivial-auto-var-init=zero makes the edge_t
# struct-padding written into the .nav deterministic (else uninitialised stack).
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-g++-16}"
ORACLE=../../oracle
NAV=$ORACLE/codemp/server/NPCNav

rm -rf build
mkdir -p build/server/NPCNav build/game build/check

# UNMODIFIED oracle TU next to the stubs (relative includes resolve to stubs).
cp "$NAV/navigator.cpp" "$NAV/navigator.h" build/server/NPCNav/
cp stubs/game/*   build/game/
cp stubs/server/* build/server/

FLAGS="-std=c++14 -w -O0 -fno-strict-aliasing -ftrivial-auto-var-init=zero -DNDEBUG -DDEDICATED -I build"
$CXX $FLAGS -o build/npcnav_oracle main.cpp build/server/NPCNav/navigator.cpp

mkdir -p fixtures goldens
status=0
for lf in layouts/*.layout; do
	name=$(basename "$lf" .layout)
	if [ "${1:-}" = "--regen" ]; then
		build/npcnav_oracle "$name" "$lf" fixtures >"goldens/$name.txt"
		echo "regenerated $name  ($(wc -c <"fixtures/$name.nav" | tr -d ' ') bytes)"
	else
		build/npcnav_oracle "$name" "$lf" build/check >"build/check/$name.txt"
		if ! cmp -s "build/check/$name.nav" "fixtures/$name.nav"; then
			echo "FIXTURE DIFF: $name.nav"; status=1
		fi
		if ! diff -u "goldens/$name.txt" "build/check/$name.txt"; then
			echo "GOLDEN DIFF: $name.txt"; status=1
		fi
	fi
done

[ "$status" -eq 0 ] && echo "npcnav-oracle: OK"
exit "$status"
