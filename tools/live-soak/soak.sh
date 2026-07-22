#!/bin/zsh
# Live-bot soak gate: boots the Rust engine + Rust jampgame with bots and
# verifies the server survives bot join, combat, a kick/refill cycle (bot
# state recycle), and a map_restart. Exit 0 = pass, non-zero = fail.
#
# The A/B referee replays recorded traces that contain no bot syscalls, so
# botlib work is referee-blind; this soak is the mandatory gate for it
# (phase-3 ruling 2026-07-22). Bot join exercises BotAISetupClient + the
# botlib precompiler/character/weight loaders; play exercises chat/goal/
# weight; kick/refill exercises shutdown + in-place state reset.
#
# Usage: tools/live-soak/soak.sh [map] [bots] [soak-seconds]
#   env: JKA_BASEPATH (default ~/Developer/jka/rust-server)
#        SOAK_PORT    (default 29095)
set -u
MAP=${1:-mp/duel1}
BOTS=${2:-8}
SOAK=${3:-60}
BASEPATH=${JKA_BASEPATH:-$HOME/Developer/jka/rust-server}
PORT=${SOAK_PORT:-29095}
REPO=${0:A:h:h:h}
LOG=$(mktemp -t soak-log)
RCON=devmal

fail() { echo "SOAK FAIL: $1"; [ -n "${SRVPID:-}" ] && kill $SRVPID 2>/dev/null; tail -15 $LOG; exit 1; }

rcon() {
  python3 - "$PORT" "$RCON" "$@" <<'EOF'
import socket, sys
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); s.settimeout(3.0)
s.sendto(b"\xff\xff\xff\xffrcon " + sys.argv[2].encode() + b" " + " ".join(sys.argv[3:]).encode(), ("127.0.0.1", int(sys.argv[1])))
out = b""
try:
    while True:
        d, _ = s.recvfrom(65536); out += d.replace(b"\xff\xff\xff\xffprint\n", b"")
except socket.timeout: pass
sys.stdout.write(out.decode("utf-8", "replace"))
EOF
}

begins() { grep -c "ClientBegin" $LOG 2>/dev/null || echo 0 }
alive() { kill -0 $SRVPID 2>/dev/null }

[ -x "$REPO/target/debug/mp_app" ] || fail "mp_app not built (cargo build -p mp_app)"
cp "$REPO/target/debug/libjampgame.dylib" "$BASEPATH/base/jampgamei386.so" || fail "dylib install"

"$REPO/target/debug/mp_app" +set dedicated 1 +set fs_basepath "$BASEPATH" \
  +set net_port $PORT +set sv_hostname "rust-soak-gate" +set sv_maxclients 32 \
  +set bot_enable 1 +set bot_minplayers $BOTS +set g_gametype 0 \
  +set rconpassword $RCON +devmap $MAP > $LOG 2>&1 &
SRVPID=$!

# 1. All bots join.
n=0
while [ $(begins) -lt $BOTS ]; do
  alive || fail "server died during bot join ($(begins)/$BOTS begins)"
  n=$((n+1)); [ $n -gt 90 ] && fail "timeout waiting for $BOTS bots ($(begins) joined)"
  sleep 2
done

# 2. Kick a bot; minplayers must refill the slot (shutdown + re-setup path).
pre=$(begins)
rcon clientkick 1 >/dev/null
n=0
while [ $(begins) -le $pre ]; do
  alive || fail "server died on kick/refill"
  n=$((n+1)); [ $n -gt 30 ] && fail "kicked slot never refilled"
  sleep 2
done

# 3. map_restart.
rcon map_restart >/dev/null
sleep 5
alive || fail "server died on map_restart"

# 4. Soak.
n=0
while [ $n -lt $((SOAK / 2)) ]; do
  alive || fail "server died during soak (t=$((n*2))s)"
  n=$((n+1)); sleep 2
done
kills=$(grep -c "Kill:" $LOG)

rcon quit >/dev/null; sleep 2
alive && kill $SRVPID 2>/dev/null

echo "SOAK PASS: $MAP, $BOTS bots, kick/refill ok, map_restart ok, ${SOAK}s soak, $kills kills"
exit 0
