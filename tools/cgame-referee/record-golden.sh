#!/bin/sh
# Records one demo-referee golden journal from the ORACLE engine (ticket gh#30).
#
# The oracle engine plays a committed demo with the standalone probe module in
# the cgame seat. The probe journals one bracket per snapshot and quits the
# engine at the bracket cap, so a run needs no operator input.
#
#   ./record-golden.sh ffa1
#   ./record-golden.sh                      # all four committed demos
#   JKA_PROBE_BRACKET_CAP=0 ./record-golden.sh swoop1   # whole demo
#
# A cap of 0 means no cap, which is how the DEC-62.2 extended check mints its
# local full-length goldens. Point JKA_REF_FULL_GOLDENS at the output directory
# and run `full_demos_match_local_goldens`. Full goldens stay out of git.
#
# Nothing here writes to the retail install or to the operator's own
# ~/Library/Application Support/OpenJK tree. The run gets a private fs_homepath.
set -eu
cd "$(dirname "$0")"

HERE="$(pwd -P)"
ENGINE="${JKA_ORACLE_CLIENT:-$HOME/Developer/jka/seam-test/client/openjk.arm64.app/Contents/MacOS/openjk.arm64}"
ASSETS="${JKA_REF_BASEPATH:-$HOME/Developer/jka/jka_server}"
UI_DYLIB="${JKA_PROBE_UI_DYLIB:-$HOME/Developer/jka/seam-test/client/base/uiarm64.dylib}"
# A path with a double slash in it loses everything after the `//`, because the
# engine tokenizes its command line with COM_Parse and `//` starts a comment.
HOME_DIR="${JKA_PROBE_HOME:-$HOME/Developer/jka/cgame-probe-home}"
OUT_DIR="${JKA_PROBE_OUT:-$HERE/goldens}"
CAP="${JKA_PROBE_BRACKET_CAP:-400}"

[ -x "$ENGINE" ] || { echo "error: no oracle client at $ENGINE (set JKA_ORACLE_CLIENT)" >&2; exit 1; }
[ -f "$ASSETS/base/assets0.pk3" ] || { echo "error: no retail paks at $ASSETS (set JKA_REF_BASEPATH)" >&2; exit 1; }
[ -f "$UI_DYLIB" ] || { echo "error: no ui module at $UI_DYLIB (set JKA_PROBE_UI_DYLIB)" >&2; exit 1; }

echo "record-golden: building the probe cdylib"
( cd probe && cargo build --release )

mkdir -p "$HOME_DIR/base/demos" "$OUT_DIR"
cp probe/target/release/libcgamearm64.dylib "$HOME_DIR/base/cgamearm64.dylib"
cp "$UI_DYLIB" "$HOME_DIR/base/uiarm64.dylib"

record_one() {
	demo="$1"
	[ -f "fixtures/$demo.dm_26" ] || { echo "error: no fixture fixtures/$demo.dm_26" >&2; exit 1; }
	cp "fixtures/$demo.dm_26" "$HOME_DIR/base/demos/$demo.dm_26"
	echo "record-golden: $demo, cap $CAP -> $OUT_DIR/$demo.journal.gz"
	JKA_PROBE_JOURNAL="$OUT_DIR/$demo.journal.gz" \
	JKA_PROBE_MANIFESTS="$HERE" \
	JKA_PROBE_BRACKET_CAP="$CAP" \
	"$ENGINE" \
		+set fs_basepath "$ASSETS" \
		+set fs_homepath "$HOME_DIR" \
		+set sv_pure 0 \
		+set vm_cgame 0 \
		+set vm_ui 0 \
		+set com_maxfps 0 \
		+set s_initsound 0 \
		+set r_fullscreen 0 \
		+set r_mode 3 \
		+set r_swapInterval 0 \
		+demo "$demo" \
		> "$HOME_DIR/$demo-console.log" 2>&1
	grep -E "cgame-probe|ERROR|Error" "$HOME_DIR/$demo-console.log" || true
	ls -l "$OUT_DIR/$demo.journal.gz"
}

if [ $# -gt 0 ]; then
	for d in "$@"; do record_one "$d"; done
else
	for d in ffa1 sabers1 spectator swoop1; do record_one "$d"; done
fi
