#!/bin/sh
# fx-oracle - run every scenario and check (or regenerate) the committed
# goldens. build.sh must have built build/fx_dump first.
#
#   sh run.sh           run every scenario, diff against golden/
#   sh run.sh --regen   regenerate golden/ (after a scenario or fixture change)
#
# Each scenario writes one golden, golden/<name>.txt, the emission stream in
# call order. The Rust FX port (gh#27) must reproduce it byte for byte.
set -eu
cd "$(dirname "$0")"

if [ ! -x build/fx_dump ]; then
	echo "fx-oracle: build/fx_dump is missing. Run build.sh first."
	exit 1
fi

REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

mkdir -p golden
STATUS=0

for f in scenarios/*.fxs; do
	name=$(basename "$f" .fxs)
	if [ "$REGEN" -eq 1 ]; then
		build/fx_dump "$f" > "golden/$name.txt"
		echo "  regenerated $name ($(wc -l < "golden/$name.txt" | tr -d ' ') records)"
	else
		build/fx_dump "$f" > "build/$name.txt"
		diff -u "golden/$name.txt" "build/$name.txt" || { echo "MISMATCH: $name"; STATUS=1; }
	fi
done

[ "$STATUS" -eq 0 ] && echo "fx-oracle: OK"
exit "$STATUS"
