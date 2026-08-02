#!/bin/sh
# cin-oracle — run every scenario and check (or regenerate) the committed
# goldens. build.sh must have built build/cin_dump first.
#
#   sh run.sh           run every scenario, diff against golden/
#   sh run.sh --regen   regenerate golden/ (after a scenario or fixture change)
#
# Each scenario writes one golden, golden/<name>.txt. The Rust decode core must
# reproduce it byte for byte through
# crates/mp/engine/client/tests/cin_oracle_goldens.rs, which needs no C++
# toolchain.
set -eu
cd "$(dirname "$0")"

if [ ! -x build/cin_dump ]; then
	echo "cin-oracle: build/cin_dump is missing. Run build.sh first."
	exit 1
fi

REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

# The scenario order is fixed so the Rust test can walk the same list.
SCENARIOS="quadinfo quadinfo_ragged codebook codebook_partial vq_frames vq_nonsquare sound_mono sound_stereo rll_direct"

mkdir -p golden
STATUS=0

for name in $SCENARIOS; do
	if [ "$REGEN" -eq 1 ]; then
		build/cin_dump "$name" > "golden/$name.txt"
		echo "  regenerated $name ($(wc -c < "golden/$name.txt") bytes)"
	else
		build/cin_dump "$name" > "build/$name.txt"
		diff -u "golden/$name.txt" "build/$name.txt" || { echo "MISMATCH: $name.txt"; STATUS=1; }
	fi
done

[ "$STATUS" -eq 0 ] && echo "cin-oracle: OK"
exit "$STATUS"
