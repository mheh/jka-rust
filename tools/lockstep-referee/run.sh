#!/bin/bash
# Lockstep referee launcher (plan G3, docs/plans/2026-07-13-engine-lockstep-referee.md).
#
# Boots two mp_app engines coupled through the streaming referee tape:
#   PRIMARY   — our jampgame module, live bots, open human slot; ref_record
#               appends the event tape in real time (flushed per game frame).
#   SECONDARY — the patched oracle jampgame module (tools/referee-oracle);
#               ref_replay + ref_follow 1 tail-follows the growing tape,
#               stepping each frame as its block lands and comparing the
#               per-frame component state digests (REF DIVERGE lines).
#
# Usage:
#   tools/lockstep-referee/run.sh                 # run until Ctrl-C (or the
#                                                 # primary is quit by hand)
#   DURATION=120 tools/lockstep-referee/run.sh    # bounded session: rcon-quit
#                                                 # the primary after N seconds
#   SECONDARY_MODULE=ours tools/lockstep-referee/run.sh
#                                                 # our-vs-our sanity mode
#
# The tape, logs, and summary land in $OUT. Servers are killed BY PID only.
set -euo pipefail

cd "$(dirname "$0")/../.."

MAP=${MAP:-mp/ffa3}
BOTS=${BOTS:-4}
SEED=${SEED:-777777}
FPS=${FPS:-20}
MAXCLIENTS=${MAXCLIENTS:-16}
GAMETYPE=${GAMETYPE:-0}
PRIMARY_BASE=${PRIMARY_BASE:-$HOME/Developer/jka/g2-server}
SECONDARY_BASE=${SECONDARY_BASE:-$HOME/Developer/jka/lockstep-oracle}
PRIMARY_PORT=${PRIMARY_PORT:-29085}
SECONDARY_PORT=${SECONDARY_PORT:-29095}
RCON=${RCON:-jkarust}
REF_STATE=${REF_STATE:-1} # 1 = verbose V records (field-level attribution)
HALT=${HALT:-0}           # 1 = freeze both engines on divergence (step mode)
OUT=${OUT:-/tmp/lockstep-referee}
BIN=${BIN:-target/debug/mp_app}
ORACLE_DYLIB=${ORACLE_DYLIB:-tools/referee-oracle/build/liboraclejampgame.dylib}
SECONDARY_MODULE=${SECONDARY_MODULE:-oracle} # oracle | ours
DURATION=${DURATION:-}

[ -x "$BIN" ] || { echo "missing $BIN (cargo build --workspace)"; exit 1; }

# Stage the secondary basepath: shared assets, module per SECONDARY_MODULE.
mkdir -p "$SECONDARY_BASE/base"
for pk3 in "$PRIMARY_BASE"/base/assets*.pk3; do
    ln -sf "$pk3" "$SECONDARY_BASE/base/$(basename "$pk3")"
done
if [ "$SECONDARY_MODULE" = oracle ]; then
    [ -f "$ORACLE_DYLIB" ] || { echo "missing $ORACLE_DYLIB (tools/referee-oracle/build.sh)"; exit 1; }
    ln -sf "$(cd "$(dirname "$ORACLE_DYLIB")" && pwd)/$(basename "$ORACLE_DYLIB")" \
        "$SECONDARY_BASE/base/jampgamei386.so"
else
    ln -sf "$PRIMARY_BASE/base/jampgamei386.so" "$SECONDARY_BASE/base/jampgamei386.so"
fi

mkdir -p "$OUT"
TAPE=$OUT/tape.txt
rm -f "$TAPE" "$OUT"/primary.log "$OUT"/secondary.log

COMMON_ARGS=(+set dedicated 1 +set sv_maxclients "$MAXCLIENTS" +set bot_enable 1
    +set g_gametype "$GAMETYPE" +set sv_fps "$FPS" +set ref_seed "$SEED"
    +set rconpassword "$RCON" +set bot_minplayers "$BOTS"
    +set ref_haltOnDiverge "$HALT")

"./$BIN" "${COMMON_ARGS[@]}" +set fs_basepath "$PRIMARY_BASE" \
    +set net_port "$PRIMARY_PORT" +set ref_record "$TAPE" +set ref_state "$REF_STATE" \
    +map "$MAP" >"$OUT/primary.log" 2>&1 &
PRIMARY_PID=$!
echo "primary   pid=$PRIMARY_PID port=$PRIMARY_PORT module=ours log=$OUT/primary.log"

cleanup() {
    kill "$PRIMARY_PID" 2>/dev/null || true
    kill "${SECONDARY_PID:-0}" 2>/dev/null || true
}
trap cleanup INT TERM

# The follower bounded-waits for the H header itself; just require the file.
for _ in $(seq 1 300); do [ -s "$TAPE" ] && break; sleep 0.2; done
[ -s "$TAPE" ] || { echo "primary never wrote $TAPE"; cleanup; exit 1; }

"./$BIN" "${COMMON_ARGS[@]}" +set fs_basepath "$SECONDARY_BASE" \
    +set net_port "$SECONDARY_PORT" +set ref_replay "$TAPE" +set ref_follow 1 \
    +map "$MAP" >"$OUT/secondary.log" 2>&1 &
SECONDARY_PID=$!
echo "secondary pid=$SECONDARY_PID port=$SECONDARY_PORT module=$SECONDARY_MODULE log=$OUT/secondary.log"

if [ -n "$DURATION" ]; then
    sleep "$DURATION"
    python3 - "$PRIMARY_PORT" "$RCON" <<'EOF'
import socket, sys
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.sendto(b'\xff\xff\xff\xffrcon ' + sys.argv[2].encode() + b' quit', ('127.0.0.1', int(sys.argv[1])))
EOF
fi

wait "$PRIMARY_PID" 2>/dev/null || true
# The E end record lets the follower finish and self-quit.
wait "$SECONDARY_PID" 2>/dev/null || true

echo "--- secondary summary ---"
grep -E "REF (FOLLOW|REPLAY|DIVERGE)" "$OUT/secondary.log" | tail -20
