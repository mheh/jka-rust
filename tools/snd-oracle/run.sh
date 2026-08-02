#!/bin/sh
# snd-oracle — run every scenario and check (or regenerate) the committed
# goldens. build.sh must have built build/snd_dump first.
#
#   sh run.sh           run every scenario, diff against golden/
#   sh run.sh --regen   regenerate golden/ (after a scenario or fixture change)
#
# Each scenario writes two goldens: <name>.txt, the state and ring digests, and
# <name>.bin, the final dma_t ring bytes. The Rust mixer (gh#24) must reproduce
# both byte for byte.
set -eu
cd "$(dirname "$0")"

if [ ! -x build/snd_dump ]; then
	echo "snd-oracle: build/snd_dump is missing. Run build.sh first."
	exit 1
fi

REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

mkdir -p golden
STATUS=0

for f in scenarios/*.snd; do
	name=$(basename "$f" .snd)
	if [ "$REGEN" -eq 1 ]; then
		build/snd_dump "$f" "golden/$name.bin" > "golden/$name.txt"
		echo "  regenerated $name ($(wc -c < "golden/$name.txt") text, $(wc -c < "golden/$name.bin") ring)"
	else
		build/snd_dump "$f" "build/$name.bin" > "build/$name.txt"
		diff -u "golden/$name.txt" "build/$name.txt" || { echo "MISMATCH: $name.txt"; STATUS=1; }
		cmp "golden/$name.bin" "build/$name.bin" || { echo "MISMATCH: $name.bin"; STATUS=1; }
	fi
done

[ "$STATUS" -eq 0 ] && echo "snd-oracle: OK"
exit "$STATUS"
