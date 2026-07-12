# B4 — Network: Ground-Truth Dossier

Scope: raw material for the design session on DEC-06 (full JKA 1.01 wire
compatibility, protocol 26) — this is the wire-compat backbone doc. Every
claim cites `oracle/<path>:<line>`. MP tree = `oracle/codemp/`,
SP tree = `oracle/code/`. Cross-ref: A2 dossier §1f has the netchan
globals census (`showpackets`/`showdrop`/`qport`/`net_killdroppedfragments`
cvars, `loopbacks[2]`).

Status: complete.

---

## 1. Wire format ground truth

### 1a. Connectionless packets (the `-1` marker)

Raven's own header comment states the rule (`oracle/codemp/qcommon/net_chan.cpp:15-16`):

> "if the sequence number is -1, the packet should be handled as an
> out-of-band message instead of as part of a netcon."

- `NET_SendPacket` treats a leading 4-byte `-1` specially only for log
  verbosity: `if ( showpackets->integer && *(int *)data == -1 )` —
  `net_chan.cpp:534`.
- Writers: `NET_OutOfBandPrint` writes four raw `-1` (0xFF) bytes then a
  `vsprintf`'d command string (`net_chan.cpp:565-571`); `NET_OutOfBandData`
  writes four `0xff` bytes, copies the payload starting at byte 4, then
  **huffman-compresses everything past byte 12** via `Huff_Compress(&mbuf,
  12)` before sending (`net_chan.cpp:591-602`).
- No separate `Netchan_OutOfBand*` wrapper exists — `NET_OutOfBandPrint`/
  `NET_OutOfBandData` (both in `net_chan.cpp`, declared `qcommon.h:138-139`)
  are the only implementations.
- The demux of `-1` vs. sequenced packets into the OOB-vs-netchan handler
  path itself happens in the caller (`SV_PacketEvent`/`CL_PacketEvent`,
  outside net_chan.cpp), not inside `Netchan_Process`.

### 1b. Netchan connected-packet header, bit-by-bit

Verbatim from source (`net_chan.cpp:6-27`):

```
4   outgoing sequence.  high bit will be set if this is a fragmented message
[2  qport (only for client to server)]
[2  fragment start byte]
[2  fragment length. if < FRAGMENT_SIZE, this is the last fragment]
```

No checksum/CRC field exists anywhere in this header.

Constants (`net_chan.cpp:29-38`, non-Xbox build):
| Name | Value | Source |
|---|---|---|
| `MAX_PACKETLEN` | 1400 | `net_chan.cpp:33` |
| `FRAGMENT_SIZE` | `MAX_PACKETLEN - 100` = 1300 | `net_chan.cpp:34` |
| `PACKET_HEADER` | 10 ("two ints and a short") | `net_chan.cpp:35` |
| `FRAGMENT_BIT` | `1<<31` | `net_chan.cpp:38` |
| `MAX_MSGLEN` | 49152 | `qcommon.h:150` |
| `PACKET_BACKUP` | 32 | `qcommon.h:98` |
| `PACKET_MASK` | `PACKET_BACKUP-1` | `qcommon.h:100` |

(Xbox build uses a different `MAX_PACKETLEN`=1359/`FRAGMENT_SIZE` formula
with a "needed due to huffman?" fudge-factor comment, `net_chan.cpp:30-31` —
not in scope for the PC-parity target but flags that Raven itself wasn't
fully certain of the margin.)

- **`Netchan_Transmit`** (`net_chan.cpp:145-194`): if `length >=
  FRAGMENT_SIZE`, buffers into `chan->unsentBuffer` and hands off to
  `Netchan_TransmitNextFragment` (`:159-169`); otherwise writes plain
  `outgoingSequence` long (`:174-175`, incrementing after), then `qport` as
  a **short**, only `if (chan->sock == NS_CLIENT)` (`:178-180`), then the
  raw payload.
- **`Netchan_TransmitNextFragment`** (`net_chan.cpp:88-134`): writes
  `chan->outgoingSequence | FRAGMENT_BIT` (`:96`), conditionally the qport
  short (`:99-101`), then `fragmentStart` short + `fragmentLength` short
  (`:109-110`), then the fragment bytes. `outgoingSequence` only increments
  once the **final** fragment (`fragmentLength != FRAGMENT_SIZE`) ships
  (`:130-133`) — all fragments of one message share one sequence number,
  matching the header comment (`:18`).
- **`Netchan_Process`** (`net_chan.cpp:208-366`): reads the sequence long,
  masks the fragment bit (`sequence &= ~FRAGMENT_BIT`, `:219-224`), reads
  qport only `if (chan->sock == NS_SERVER)` (`:227-229`), then (if
  fragmented) `fragmentStart`/`fragmentLength` as unsigned shorts
  (`:233-234`). Drops packets with `sequence <= chan->incomingSequence`
  (dedup/reorder guard, `:258-266`); records the gap in `chan->dropped`
  (`:271-279`).

**Fragmentation reassembly.** Bookkeeping fields on `netchan_t`
(`qcommon.h:163-186`): incoming — `fragmentSequence` (`:176`),
`fragmentLength` (`:177`), `fragmentBuffer[MAX_MSGLEN]` (`:178`); outgoing —
`unsentFragments` (`:182`), `unsentFragmentStart` (`:183`), `unsentLength`
(`:184`), `unsentBuffer[MAX_MSGLEN]` (`:185`). In `Netchan_Process:286-358`:
a `sequence` change from `chan->fragmentSequence` resets `fragmentLength` to
0 (new message, `:288-291`); a fragment whose `fragmentStart !=
chan->fragmentLength` (non-contiguous — dropped/duplicate/reordered) aborts
the whole reassembly and the packet is discarded (`:294-315` — source
comments show the original team argued over this strictness). Otherwise the
fragment is memcpy'd into `fragmentBuffer` at the current offset
(`:327-330`); a fragment shorter than `FRAGMENT_SIZE` signals "last
fragment" and triggers reconstruction into the local `msg` (`:344-353`).
**Net effect: fragment reassembly is strictly in-order — no true
out-of-order reassembly, only strict duplicate/gap rejection.**

**qport rationale** (`net_chan.cpp:20-25`, verbatim): a workaround for NAT
routers that remap the client's source port mid-session; matching
base-address + qport lets the channel survive an IP-port change. Sent as a
**short**, client→server only.

### 1c. usercmd transmission (framing-level; see §6 for full mechanism)

`CL_WritePacket`'s own header comment gives the full connected-packet body
layout during gameplay (`cl_input.cpp:1594-1605`):

```
4   sequence number         (netchan header, not the payload proper)
2   qport
4   serverid
4   acknowledged sequence number
4   clc.serverCommandSequence
<optional reliable commands>
1   clc_move or clc_moveNoDelta
1   command count
<count * usercmds>
```

This body is written after `MSG_Bitstream(&buf)` (`cl_input.cpp:1632`), **not**
`MSG_InitOOB` — i.e. the payload after the netchan header goes through the
huffman path (§2), while the netchan header fields themselves (sequence,
qport, fragment info) are always written raw via `MSG_InitOOB`
(`net_chan.cpp:94,172`).

---

## 2. Huffman & MSG bit-packing

### 2a. Adaptive, but pre-seeded from a static frequency table

JKA's huffman is Sayood's adaptive-Huffman algorithm (`huffman.cpp:1-4`) —
`Huff_addRef`/tree-rebalance logic (`huffman.cpp:128-233`) does live FGK-style
updates per symbol seen, so it is **adaptive at runtime**, not static. But
both compressor and decompressor trees are pre-warmed at init from a fixed
frequency table so client and server start from an identical, non-uniform
tree rather than from scratch:

```c
// msg.cpp:3228-3232
for (i=0;i<256;i++) {
    for (j=0;j<msg_hData[i];j++) {
        Huff_addRef(&msgHuff.compressor, (byte)i);
        Huff_addRef(&msgHuff.decompressor, (byte)i);
    }
}
```
called from `MSG_initHuffman()` (`msg.cpp:3219-3234`, `#ifndef
_USINGNEWHUFFTABLE_`).

**The active `msg_hData[256]` table is at `msg.cpp:2958-3215`, labeled "//
Q3 TA freq. table."** (`:2957`) — i.e. it is the **unmodified Quake III:
Team Arena** frequency table, not something JKA-tuned. A second table of the
same name at `msg.cpp:2696-2954` ("New data gathered to tune Q3 to JK2MP...
gain was minimal") is **entirely inside a `/* ... */` block comment**
(opened before `:2696`, closed `:2955`) — dead, never compiled. **A
faithful port must seed from the Q3TA table at :2958, not the commented-out
one** — an easy fidelity trap since both are named identically and adjacent.

### 2b. `oob` toggle and the core bit-stuffing loop

`msg->oob` (field on `msg_t`, `qcommon.h:20`) selects raw-byte vs.
huffman-bitpacked mode inside `MSG_WriteBits`/`MSG_ReadBits`. `MSG_InitOOB`
sets `oob = qtrue` (`msg.cpp:91`, used for OOB/connectionless packets and
netchan headers); `MSG_Bitstream` clears it to `qfalse` (`msg.cpp:101-103`,
used for connected-message payloads — see §1c). Read-side mirrors:
`MSG_BeginReadingOOB`/`MSG_BeginReading` (`msg.cpp:105-115`).

Non-oob (huffman) write path (`msg.cpp:187-208`):
```c
value &= (0xffffffff>>(32-bits));
if (bits&7) {
    int nbits = bits&7;
    for(i=0;i<nbits;i++) {
        Huff_putBit((value&1), msg->data, &msg->bit);
        value = (value>>1);
    }
    bits = bits - nbits;
}
if (bits) {
    for(i=0;i<bits;i+=8) {
        Huff_offsetTransmit(&msgHuff.compressor, (value&0xff), msg->data, &msg->bit);
        value = (value>>8);
    }
}
msg->cursize = (msg->bit>>3)+1;
```
i.e. leftover sub-byte bits go through raw bit-stuffing (`Huff_putBit`),
full bytes go through the adaptive huffman tree (`Huff_offsetTransmit`).
Read mirror at `msg.cpp:243-260`. Single-bit primitives `Huff_putBit`/
`Huff_getBit` are at `huffman.cpp:12-29`, packing into `fout[bloc>>3]` at bit
`bloc&7`.

### 2c. `netField_t` delta tables — the wire contract

Struct (`msg.cpp:833-844`): `{ char *name; int offset; int bits; /* 0 =
float */ }` (Xbox variant adds a `realSize`/`mCount`). Built via a
null-pointer-cast offsetof macro, `NETF(x)` for entities
(`msg.cpp:848-850`): `#define NETF(x) #x,(int)&((entityState_t*)0)->x`
(non-Xbox; Xbox variant also captures `sizeof(...)`).

- **`entityStateFields[]`**: `msg.cpp:859-1051`, **132 entries**. Count is
  runtime-asserted: `numFields =
  sizeof(entityStateFields)/sizeof(entityStateFields[0])` (`:1078`), and
  `assert( numFields + 1 == sizeof(*from)/4 )` (`:1085`) — the `+1` accounts
  for `number`, transmitted separately from the table. **This assert is
  itself a golden fact**: every 4-byte word of `entityState_t` must appear
  exactly once in the table or the oracle build itself would assert-fail.
- **`playerStateFields[]`**: **two mutually-exclusive definitions**, gated
  by `#ifdef _OPTIMIZED_VEHICLE_NETWORKING` (`msg.cpp:1404`, table at
  `:1410-1568`) vs. `#else` (table at `:1829-1972`, 140 entries).
  **`_OPTIMIZED_VEHICLE_NETWORKING` is unconditionally `#define`d at
  `oracle/codemp/game/q_shared.h:2154`** — so the shipped 1.01 build
  uses the `#ifdef` branch (`:1410`) plus the companion
  `pilotPlayerStateFields[]` (`:1570`) and `vehPlayerStateFields[]`
  (`:1736`), **not** the `:1829` table. A port that transcribes the wrong
  (`#else`) table would silently diverge from the real wire format.
- Entity delta algorithm, `MSG_WriteDeltaEntity`/`MSG_ReadDeltaEntity`
  (`msg.cpp:1069-1213` / `1231-1390`): scans all fields for `*fromF !=
  *toF` to find a "last changed" index `lc` (`:1106-1124`); writes entity
  number (`GENTITYNUM_BITS`), a "removed" bit, a "has-delta" bit, then byte
  `lc` (`:1132-1142`). For each field `i < lc`: a 1-bit changed flag
  (`:1159-1165`); floats (`field->bits == 0`) get a nonzero-bit then an
  integer-vs-full-float choice bit (small ints biased/packed into
  `FLOAT_INT_BITS`, `:1167-1187`); integer fields get a nonzero-bit then a
  `field->bits`-wide value (`:1202-1210`). `to == NULL` means "delta
  remove": just entity number + 1 remove-bit (`:1088-1096`); on read, a
  removed entity's number equals `MAX_GENTITIES-1` and the caller drops it
  (`cl_parse.cpp:82-84`). `GENTITYNUM_BITS` is build-config-dependent: 9, 10,
  or 11 depending on `#if` branch (`q_shared.h:1992/1994/2000`) —
  `MAX_GENTITIES = 1<<GENTITYNUM_BITS`.
- Playerstate delta (`MSG_WriteDeltaPlayerstate`, `msg.cpp:2211+` /
  `MSG_ReadDeltaPlayerstate`, `msg.cpp:2460+`) follows the identical
  per-field changed-bit `netField_t` pattern, selecting among
  `vehPlayerStateFields`/`pilotPlayerStateFields`/`playerStateFields`
  by vehicle/pilot state (`:2244-2265`, `:2495-2513`).

### 2d. usercmd delta — hand-coded, not table-driven

Two variants exist, both in `msg.cpp`, neither uses `netField_t`:
- **`MSG_WriteDeltaUsercmd`/`MSG_ReadDeltaUsercmd`** (`msg.cpp:685-733`):
  serverTime as a 1-bit flag + 8-bit delta (if `<256`) else 32-bit absolute
  (`:686-692`); then each field (`angles[0..2]`, `forwardmove`,
  `rightmove`, `upmove`, `buttons`, `weapon`, `forcesel`, `invensel`,
  `generic_cmd`) individually via `MSG_WriteDelta`/`MSG_ReadDelta`
  (`:567-581`) — **one changed-bit per field**, not a grouped bitmask.
- **`MSG_WriteDeltaUsercmdKey`/`MSG_ReadDeltaUsercmdKey`**
  (`msg.cpp:740-778`+) — the one actually used by `CL_WritePacket`/
  `SV_UserMove` (§6). Same serverTime handling; first tests **all** fields
  for equality, and if none changed, writes a single "no change" bit and
  returns early (`:748-762`). Otherwise XORs `key ^= to->serverTime`
  (`:763`) and writes each field via `MSG_WriteDeltaKey` (`:621-630`), which
  XOR-obfuscates the value with `key` before writing — an anti-spoofing/
  anti-sniffing measure, masked with a `kbitmask[32]` table on read
  (`:610-619`, `:632-636`). Old `CM_ANGLE1`..`CM_INVEN` bitmask `#define`s
  exist (`:668-679`) but are **unused by either function** — dead leftovers
  from an earlier bitmask-based encoding; do not port them as live logic.

---

## 3. Delta snapshot flow

### 3a. Server: `SV_WriteSnapshotToClient` (`sv_snapshot.cpp:103-215`)

- **Baselines**: `sv.svEntities[entnum].baseline` (an `entityState_t`),
  populated once per level in `SV_CreateBaseline`
  (`sv_init.cpp:209-225`, assignment at `:223`). Used as the delta source
  for entities newly entering a client's PVS:
  `MSG_WriteDeltaEntity(msg, &sv.svEntities[newnum].baseline, newent,
  qtrue)` (`sv_snapshot.cpp:80`), and for the full initial gamestate
  (delta'd from an all-zero `nullstate`, `sv_client.cpp:749-758`).
- **Delta-from-frame selection**: picks `oldframe` from `client->frames[]`,
  a `PACKET_BACKUP`(32)-sized ring, indexed `client->deltaMessage &
  PACKET_MASK` (`sv_snapshot.cpp:125`). Falls back to a full (non-delta)
  snapshot (`oldframe = NULL`) when: `deltaMessage <= 0 || state !=
  CS_ACTIVE` (retransmit request, `:113-116`); `outgoingSequence -
  deltaMessage >= (PACKET_BACKUP - 3)` (delta too old, `:117-122`); or
  `oldframe->first_entity <= svs.nextSnapshotEntities -
  svs.numSnapshotEntities` (referenced entities already evicted from the
  shared circular entity buffer, `:129-133`).
- **Areabits**: computed per recursive portal-visible viewpoint via
  `frame->areabytes = CM_WriteAreaBits(frame->areabits, clientarea)`
  (`sv_snapshot.cpp:332`, buffer `MAX_MAP_AREA_BYTES`, `server.h:96`); after
  OR-ing all viewpoints together, `SV_BuildClientSnapshot` **inverts** the
  bits: `((int *)frame->areabits)[i] ^= -1` over `MAX_MAP_AREA_BYTES/4` ints
  (`:591-595`).
- **Message order**: `SV_SendClientSnapshot` (`sv_snapshot.cpp:719-798`) →
  `lastClientCommand` long (`:777`) → `SV_UpdateServerCommandsToClient`
  reliable-command replay (`:780`) → `SV_WriteSnapshotToClient` (`:784`,
  internally: serverTime `:144` → lastframe byte `:147` → snapFlags `:157`
  → areabytes+areabits `:160-161` → delta playerstate (+vehicle ps)
  `:164-204` → delta packet entities `:207`) → download data (`:788`).

### 3b. Client: `cl_parse.cpp`

- **`CL_ParseSnapshot`** (`:207-328`): reads `deltaNum = MSG_ReadByte`
  (`:231`), computes `newSnap.deltaNum = newSnap.messageNum - deltaNum`
  (`:235`), looks up `old = &cl.snapshots[newSnap.deltaNum & PACKET_MASK]`
  — a `PACKET_BACKUP`(32)-sized ring (`client.h:130`). Validity gated by
  `old->valid`, `old->messageNum == newSnap.deltaNum`, and
  `cl.parseEntitiesNum - old->parseEntitiesNum > MAX_PARSE_ENTITIES-128`
  (`:251-262`) — mirrors the server's staleness guard. Snapshots between
  last-received and new are invalidated to prevent stale-buffer deltas
  after a drop (`:299-306`).
- **`CL_ParseGamestate`** (`:533-662`, invoked on `case svc_gamestate:`,
  `:906-907`): `CL_ClearState()` (`:546`) → reads
  `clc.serverCommandSequence` (`:554`) → loops `svc_configstring` entries
  (`MSG_ReadShort` index + `MSG_ReadBigString` value into
  `cl.gameState.stringData`/`stringOffsets`, `:565-620`) and
  `svc_baseline` entries (`cl.entityBaselines[newnum]` via
  `MSG_ReadDeltaEntity(msg, &nullstate, es, newnum)`, `:621-628`) until
  `svc_EOF` (`:561-563`).
- **`CL_DeltaEntity`** (`:65-87`): writes into circular
  `cl.parseEntities[cl.parseEntitiesNum & (MAX_PARSE_ENTITIES-1)]`
  (`:71`). `unchanged=qtrue` just copies `*old`, no wire read (`:75`);
  otherwise `MSG_ReadDeltaEntity(msg, old, state, newnum)` (`:79`).
  Removal signaled by `state->number == (MAX_GENTITIES-1)`, entity dropped
  from the frame, not added (`:82-84`). New-vs-baseline distinction is made
  by the caller `CL_ParsePacketEntities` (`:95-195`): `oldnum < newnum` →
  re-emit unchanged (`:130-146`); matching numbers → delta from old frame
  (`:147-164`); `oldnum > newnum` → genuinely new, delta'd from
  `&cl.entityBaselines[newnum]` (`:166-172`). End-of-list sentinel
  `newnum == (MAX_GENTITIES-1)` (`:120-124`) matches the server's
  `MSG_WriteBits(msg, (MAX_GENTITIES-1), GENTITYNUM_BITS)` end marker
  (`sv_snapshot.cpp:93`).

### 3c. Configstrings

Storage: `char *configstrings[MAX_CONFIGSTRINGS]` in `server_t`
(`server.h:67`; `MAX_CONFIGSTRINGS` = 1700, `q_shared.h:2037`). Incremental
update: `SV_SetConfigstring` (`sv_init.cpp:25-96`) — no-op if unchanged
(`:39-41`), else `Z_Free`+`CopyString` (`:44-45`); if `sv.state == SS_GAME
|| sv.restarting` (`:49`), broadcasts to every client `>= CS_PRIMED` as a
reliable `"cs %i \"%s\""` command, or, for long strings (`len >=
maxChunkSize`, `maxChunkSize = MAX_STRING_CHARS - 24`, `:27`), chunked
`"bcs0"/"bcs1"/"bcs2"` commands (`:62-81`). Full-gamestate path:
`SV_SendClientGameState` (`sv_client.cpp:697-780`) iterates all
`MAX_CONFIGSTRINGS` slots, writing non-empty ones as `svc_configstring` +
short index + `MSG_WriteBigString` (`:741-747`), then baselines
(`:749-758`), then `svc_EOF` (`:760`) — called on initial connect and
gamestate resend (e.g. `:1032`, `:1821`).

### 3d. Reliable command windows (both directions)

`MAX_RELIABLE_COMMANDS = 128` (`qcommon.h:106`).

**Server→client**: storage `char
reliableCommands[MAX_RELIABLE_COMMANDS][MAX_STRING_CHARS]` in `client_t`
(`server.h:130`). Enqueue: `SV_AddServerCommand` (`sv_main.cpp:116-141`) —
`reliableSequence++` then write at `index = reliableSequence &
(MAX_RELIABLE_COMMANDS-1)` (`:125,139-140`); overflow (`reliableSequence -
reliableAcknowledge == MAX_RELIABLE_COMMANDS + 1`) → `SV_DropClient(client,
"Server command overflow")` (`:130-138`). Retransmit-until-acked: every
outgoing message, `SV_UpdateServerCommandsToClient`
(`sv_snapshot.cpp:225-235`) re-sends everything from
`reliableAcknowledge+1` through `reliableSequence` as `svc_serverCommand` +
sequence long + string. Client's ack: raw `MSG_ReadLong` every incoming
packet (`sv_client.cpp:1789`, clamped if wildly stale `:1794-1799`).

**Client→server**: storage `reliableCommands[MAX_RELIABLE_COMMANDS][...]`
in `clientConnection_t` (`client.h:183`; separate received-command ring
`serverCommands[...]` at `:196`). Enqueue: `CL_AddReliableCommand`
(`cl_main.cpp:156-167`) — errors (`Com_Error(ERR_DROP, "Client command
overflow")`) if backlog `> MAX_RELIABLE_COMMANDS` (`:161-163`).
Retransmit-until-acked: every outgoing packet, `CL_WritePacket`
(`cl_input.cpp:1646-1650`) re-sends unacked commands as `clc_clientCommand`
+ sequence long + string. Server's ack: raw `MSG_ReadLong` every incoming
server message (`cl_parse.cpp:866`, clamped `:868-870`).

**Mechanism**: both directions use the same non-NACK, plain
monotonically-increasing-sequence scheme — the sender re-embeds every
unacknowledged command in **every** outgoing packet (full-window resend,
not selective retransmit) until the peer's next incoming packet reports a
`reliableAcknowledge` that has caught up. Hard drop/error if the backlog
would exceed the 128-slot window.

---

## 4. Connection flow

### 4a. Challenge/connect handshake

- **`SV_GetChallenge`** (`sv_client.cpp:31-130`). Storage: `challenge_t
  challenges[MAX_CHALLENGES]` in `svs` (`server.h:220`, struct at
  `:194-201`: `netadr_t adr; int challenge; int time; int pingTime; int
  firstTime; qboolean connected;`). `MAX_CHALLENGES = 1024`
  (`server.h:190`) — comment: "made large to prevent a denial of service
  attack that could cycle all of them out before legitimate users
  connected" (`:187-190`). Reuses a slot for the same un-connected IP
  (`:53-56`) else evicts the oldest by `time` (`:57-72`); challenge value =
  `((rand()<<16)^rand())^svs.time` (`:67`). LAN clients get an immediate
  `"challengeResponse %i"` (`:76-79`); non-LAN may go through an
  authorize-server round trip gated by `AUTHORIZE_TIMEOUT`=5000ms
  (`server.h:192`, `sv_client.cpp:100-107`).
- **`SV_DirectConnect`** (`sv_client.cpp:221-568`), validation order:
  protocol version (`version != PROTOCOL_VERSION` →
  `"print\nServer uses protocol version %i.\n"`, `:241-246`) → reconnect-
  flood "quick reject" via address+qport/port + `sv_reconnectlimit`
  (`:252-274`) → challenge validity, linear scan of `svs.challenges[]`
  (`:277-290`) → ping-gating via `sv_minPing`/`sv_maxPing` (`:299-314`) →
  slot allocation incl. `sv_privateClients`/`sv_privatePassword`
  (`:343-373`). On success: `Netchan_Setup(NS_SERVER, &newcl->netchan, from,
  qport)` (`:514`) → `GAME_CLIENT_CONNECT` mod hook (`:520`) →
  `NET_OutOfBandPrint(NS_SERVER, from, "connectResponse")` (`:540`) →
  `state = CS_CONNECTED` (`:544`).
- **Client**: `CL_Connect_f` (`cl_main.cpp:1141-1206`) sets `cls.state =
  CA_CHALLENGING` for local addresses else `CA_CONNECTING`
  (`:1197-1201`), primes `clc.connectTime = -99999` so
  `CL_CheckForResend` fires immediately (`:1204`). `CL_CheckForResend`
  (`:1641-1725`) only acts in `CA_CONNECTING`/`CA_CHALLENGING` (`:1654`);
  fixed-interval resend gate `cls.realtime - clc.connectTime <
  RETRANSMIT_TIMEOUT` (`:1658`; `RETRANSMIT_TIMEOUT = 3000`,
  `client.h:19`) — **no exponential backoff**, flat 3s retry. In
  `CA_CONNECTING` sends `"getchallenge"` (`:1674`); in `CA_CHALLENGING`
  builds userinfo (`protocol`/`qport`/`challenge` keys) and sends
  `"connect \"%s\""` (`:1679-1715`).
- **`PROTOCOL_VERSION`**: MP = **26**, `oracle/codemp/qcommon/qcommon.h:205`.
  Server checks the client's connect request at `sv_client.cpp:242`. Client
  embeds it in the connect userinfo (`cl_main.cpp:1682`) and separately
  checks a server's info-broadcast reply for server-browser pings
  (`CL_ServerInfoPacket`, `cl_main.cpp:2848`) — **not** re-checked in the
  `connectResponse`/`challengeResponse` handler itself
  (`CL_ConnectionlessPacket`, `:2044-2081`), since the server already
  gates mismatches at connect time.
- SP's equivalent define is **`PROTOCOL_VERSION 40`**
  (`oracle/code/qcommon/qcommon.h:199`) — confirms DEC-06's "protocol
  26" target is MP-specific; SP has its own (unreachable over real UDP, see
  §7) value.

### 4b. qport (cross-ref §1b for the netchan-header half)

Rationale per Raven's comment (`net_chan.cpp:20-25`): NAT/router source-port
remapping workaround. Cvar: `qport = Cvar_Get("net_qport",
va("%i",port), CVAR_INIT)` (`net_chan.cpp:60`, seeded random in
`Netchan_Init`). Client sends it as a `qport` key in the connect-string
userinfo (`cl_main.cpp:1683`, value from `Cvar_VariableValue("net_qport")`,
`:1679`); server reads it back (`sv_client.cpp:249`), uses it for
reconnect-flood matching (`:263-265,328-330`), and stores it via
`Netchan_Setup` (`:514`). On established connections the server also reads
qport straight off every sequenced packet header to re-match clients behind
translating routers: `qport = MSG_ReadShort(msg) & 0xffff;` then `if
(cl->netchan.qport != qport) continue;` in `SV_PacketEvent`
(`sv_main.cpp:609,621-623`, with port-fixup logic at `:625-631`).

### 4c. Disconnect paths & timeouts

- **Server timeout**: `SV_CheckTimeouts` (`sv_main.cpp:719-745`). Cvars
  `sv_timeout` (default 200s, `sv_init.cpp:853`), `sv_zombietime` (default
  2s, `:854`). `droppoint = svs.time - 1000*sv_timeout->integer`
  (`:725`); a client past `droppoint` accrues `timeoutCount`, and after
  `>5` frames triggers `SV_DropClient(cl, "timed out")` (`:740-744`).
  Dropped clients go to `CS_ZOMBIE` (`:714-716`) and free once past
  `zombiepoint = svs.time - 1000*sv_zombietime->integer` (`:726,734-738`)
  — kept alive briefly so a final reliable disconnect ack isn't lost.
- **Client timeout**: `CL_CheckTimeout` (`cl_main.cpp:2212-2229`), gated by
  `cl_timeout->value*1000` vs. `cls.realtime - clc.lastPacketTime`
  (`:2218`); `>5` consecutive timeout frames → `Com_Error(ERR_DROP, ...)`
  (`:2222`).
- **Explicit disconnect**: server `SV_DropClient` (`sv_client.cpp:580`+,
  re-entrancy guard `:584-585`). Client `CL_Disconnect`
  (`cl_main.cpp:837`+): when `cls.state >= CA_CONNECTED`, sends the
  reliable `"disconnect"` command and flushes via `CL_WritePacket()`
  **three times** "in case one is dropped" (`:880-886`) before resetting.
  Out-of-band disconnect notices also travel outside the netchan: server
  sends `NET_OutOfBandPrint(NS_SERVER, from, "disconnect")` for
  unrecognized senders (`sv_main.cpp:648`); client's
  `CL_DisconnectPacket` (`cl_main.cpp:1738-1760`) **ignores** an inbound OOB
  disconnect if fewer than 3000ms have passed since the last real packet
  ("might be a malicious spoof", `:1748-1751`) — this exists because a
  netchan-based disconnect can get lost (`:1730-1736`).

---

## 5. Sockets

- **Setup**: `NET_IPSocket` (`win_net.cpp:588-645`) — `SOCK_DGRAM`/
  `IPPROTO_UDP`, non-blocking via `ioctlsocket(..., FIONBIO, ...)`
  (`:611`), `SO_BROADCAST` (`:617`), bind to `net_interface`/`INADDR_ANY` +
  `port` (`:622-642`). `NET_OpenIP` (`:911-929`) reads `net_ip` (default
  `"localhost"`) and `net_port` (default `PORT_SERVER`), retrying with
  incremented `net_port` on bind failure (`:925-929`). Unix equivalent:
  `NET_OpenIP`/`NET_IPSocket` (`unix_net.c:457,499`), same `net_port` cvar
  (`:465`), non-blocking via `ioctl(..., FIONBIO, ...)` (`:519`).
- **Send/receive dispatch**: generic `NET_SendPacket`
  (`net_chan.cpp:531-550`) special-cases `to.type == NA_LOOPBACK` →
  `NET_SendLoopPacket` (`:538-541`, impl `:514-526`), drops
  `NA_BOT`/`NA_BAD` (`:542-547`), else calls platform `Sys_SendPacket`
  (`:549`, win32 impl `win_net.cpp:393-466`). Receive: platform
  `Sys_GetPacket` (`win_net.cpp:250-321`, non-blocking `recvfrom`,
  `WSAEWOULDBLOCK` skip at `:278`) feeds the platform event queue
  (`win_main.cpp:1251`); loopback packets are drained separately via
  `NET_GetLoopPacket` (`net_chan.cpp:489-511`) called from
  `common.cpp:935,939`.
- **Loopback buffers**: `loopback_t loopbacks[2]` (`net_chan.cpp:486`, ring
  of `loopmsg_t[MAX_LOOPBACK]` + `get`/`send` cursors, struct
  `:481-484`). `NET_SendLoopPacket` writes to `loopbacks[sock^1]`
  (opposite side) (`:514-526`); `NET_GetLoopPacket` reads from
  `loopbacks[sock]`, dropping backlog beyond `MAX_LOOPBACK`
  (`:489-511`).
- **`NET_Sleep`**: **Windows is a no-op stub** — `void NET_Sleep( int msec )
  { }` (`win_net.cpp:1211-1212`). The real dedicated-server blocking wait
  is **Unix-only**: `select(ip_socket+1, &fdset, NULL, NULL, &timeout)` on
  the UDP socket + stdin fd, guarded by `if (!ip_socket ||
  !com_dedicated->integer) return;` (`unix_net.c:582-598`).

---

## 6. usercmd transmission

- **Client**: `CL_WritePacket` (`cl_input.cpp:1608-1706+`; layout doc
  `:1594-1605`, quoted §1c). Backup-cmd window: `oldPacketNum =
  (clc.netchan.outgoingSequence - 1 - cl_packetdup->integer) &
  PACKET_MASK; count = cl.cmdNumber - cl.outPackets[oldPacketNum].p_cmdNumber;`
  clamped to `MAX_PACKET_USERCMDS` (`:1660-1665`). `cl_packetdup` clamped
  to `[0,5]` (`:1655-1659`), defaults to `"1"` (`cl_main.cpp:2589`).
  `MAX_PACKET_USERCMDS = 32`, `PACKET_MASK = PACKET_BACKUP-1` (32)
  (`qcommon.h:98,100,102`). Each cmd written via
  `MSG_WriteDeltaUsercmdKey(&buf, key, oldcmd, cmd); oldcmd = cmd;` —
  **delta-chained against the previous cmd within the same packet**
  (`:1693-1698`), not against the last-acked cmd.
- **Server**: `SV_ExecuteClientMessage` dispatches `clc_move` →
  `SV_UserMove(cl, msg, qtrue)`, `clc_moveNoDelta` → `SV_UserMove(cl, msg,
  qfalse)` (`sv_client.cpp:1844-1847`). `SV_UserMove`
  (`:1674-1755`): reads `cmdCount = MSG_ReadByte(msg)`, rejects `<1` or
  `> MAX_PACKET_USERCMDS` (`:1687-1697`); reads each cmd via
  `MSG_ReadDeltaUsercmdKey(msg, key, oldcmd, cmd); oldcmd = cmd;`, same
  chaining as the client (`:1706-1712`). Duplicate/stale rejection is
  explicit (comment `:1736`: "usually, the first couple commands will be
  duplicates... included when cl_packetdup > 0"): a cmd is skipped if
  `cmds[i].serverTime > cmds[cmdCount-1].serverTime` (post-restart stale)
  or `cmds[i].serverTime <= cl->lastUsercmd.serverTime` (already executed);
  only surviving cmds reach `SV_ClientThink(cl, &cmds[i])`
  (`:1736-1754`).

---

## 7. SP scope confirmation

**Confirmed: SP's netchan/msg/socket code is a byte-similar fork of MP's,
kept for engine uniformity, but is structurally unreachable over real UDP
— every SP session is loopback-only.** Evidence:

- `SV_DirectConnect` unconditionally rejects non-local connectors and skips
  the challenge/response handshake entirely: `if ( !NET_IsLocalAddress
  (from) ) { NET_OutOfBandPrint(...,"print\nNo challenge for
  address.\n"); return; } else { Info_SetValueForKey( userinfo, "ip",
  "localhost" ); }` (`oracle/code/server/sv_client.cpp:46-51`).
- The client-slot loop is hardcoded to **one** slot: `for
  (i=0,cl=svs.clients ; i < 1 ; i++,cl++)` (`code/server/sv_client.cpp:58`,
  again `:81`); `sv_maxclients` is not a live cvar in SP at all — the only
  reference is a hardcoded literal `Info_SetValueForKey( infostring,
  "sv_maxclients", va("%i", 1) )` (`code/server/sv_main.cpp:221`), no
  `Cvar_Get("sv_maxclients", ...)` exists under `code/`.
- **No `SV_GetChallenge`/challenge table exists in SP** — `challenge_t` is
  still declared (`code/server/server.h:135-139`) but unused; no
  `MAX_CHALLENGES`/`svs.challenges[]` implementation exists (vs. MP's full
  `sv_client.cpp:31-130`).
- **No master-server/heartbeat code exists in SP** — zero hits for
  `Master|Heartbeat` in `code/server/sv_main.cpp`, vs. MP's
  `SV_MasterHeartbeat` (`codemp/server/sv_main.cpp:211-295`).
- Client always forces `"localhost"`: `Q_strncpyz( cls.servername,
  "localhost", ... ); ... // we don't need a challenge on the localhost`
  (`code/client/cl_main.cpp:225-234`).
- No Windows socket layer exists for SP at all — no `win_net.cpp`
  equivalent under `code/win32/`, and `WSAStartup` appears nowhere under
  `code/` (vs. MP's `win_net.cpp:1171`). SP's Windows build presumably
  never opens a real socket, relying purely on the `NA_LOOPBACK` path.
- SP's **Unix** build does retain a genuine UDP socket layer
  (`code/unix/unix_net.c`, real `socket()`/`bind()` at `:356,388`) — but
  it's unreachable in practice given the gates above; `code/null/null_net.c`
  (the stub target) recognizes only `"localhost"`
  (`:15-20`) with no-op send/recv (`:27-38`).
- SP's `PROTOCOL_VERSION = 40` (`code/qcommon/qcommon.h:199`) vs. MP's 26 —
  the field is still checked in SP's `SV_DirectConnect`
  (`code/server/sv_client.cpp:33-38`), confirming it's vestigial-but-wired,
  not dead code.
- `net_chan.cpp` is structurally near-identical between trees (same
  function set/order); MP's copy has one extra function
  (`Netchan_TransmitNextFragment`, `codemp/qcommon/net_chan.cpp:88`) for
  its fragmentation-retransmit feature that SP's older copy lacks.

**Scoping implication**: SP net_chan/msg/huffman code should be ported (or
explicitly stub-marked) as a **loopback-only, single-slot** engine path —
DEC-06's protocol-26 wire-compat target does not apply to SP at all (SP's
own protocol constant, 40, is never exercised against a real peer).

---

## 8. TU-harness candidates (DEC-09)

Per DEC-09, huffman + msg bit-packing + delta-entity tables are proposed as
standalone golden-test harnesses. Header-dependency and global-state survey:

- **`huffman.cpp`** (417 lines): single include,
  `#include "../qcommon/exe_headers.h"` (`:7`), which itself only pulls
  `../game/q_shared.h` + `../qcommon/qcommon.h`
  (`codemp/qcommon/exe_headers.h:4-5`) — no engine-singleton includes. One
  file-static beyond the `huff_t`/`msg_t` context: `static int bloc = 0;`
  (`:10`), read/written by `add_bit`/`get_bit` (`:13-27,33-44`) — but every
  public entry point (`Huff_Compress`/`Huff_Decompress`, `:253,299,330,383`)
  re-seeds `bloc` from the caller's `*offset` argument, so results stay
  **deterministic per call** despite the file-static. **Good standalone
  candidate.**
- **`msg.cpp`**: includes `exe_headers.h` (`:2`), conditionally
  `INetProfile.h` (`:4`, `_DONETPROFILE_` only), and unconditionally
  `../game/g_public.h` + `../server/server.h` (`:8-9`) — the file's own
  comment explains why: `// rjr: this is only used when cl_shownet is
  turned on and the server and client are in the same session` (`:7`). The
  **only** live use of that dependency is a debug print inside
  `MSG_ReadDeltaEntity` gated by `cl_shownet->integer >= 2`
  (`:1248,1266-1272,2484`, `extern cvar_t *cl_shownet;` at `:12`).
  `MSG_WriteBits`/`MSG_ReadBits`/`MSG_WriteDeltaEntity`/`MSG_ReadDeltaEntity`
  otherwise only call `Com_Error` on malformed input (`:141,185,241,1099,1239`)
  — no `Cvar_VariableValue` calls exist in the file at all. **For a
  harness: stub `cl_shownet->integer = 0` (or provide a trivial
  cvar/`Com_Error`/`Com_Printf`/`SV_GentityNum` stub) and no real
  server/game state is needed for correctness testing.** Good candidate.
- **`netField_t` tables**: reference offsets into `entityState_t`
  (`codemp/game/q_shared.h`, `entityState_s` at `:2670-2832` for the
  `#ifndef _XBOX` PC build, alternate packed Xbox layout `:2841-2985`) and
  `playerState_t` (`playerState_s` at `:2169-2435`). A harness must mirror
  these exact struct layouts for offsets to line up — this is the same
  `entityState_t`/`playerState_t` already ported at the qshared tier
  (`docs/type-port-todo.md` line ~154/248), so the harness's Rust side gets
  this for free if it reuses those types directly rather than re-deriving
  layout. Offset macro `NETF`/`PSF` is a null-pointer-cast `offsetof` idiom
  local to `msg.cpp` (`:846-849` / `:1395-1398`), not shared from
  `q_shared.h`.
- **`net_chan.cpp` framing (`Netchan_Transmit`/`Netchan_Process`) is
  *not* a pure function** — it calls `NET_SendPacket` directly (`:114,185`),
  reads cvars `showpackets`/`showdrop`/`net_killdroppedfragments`
  (`Cvar_Get` at `:58-61`, read at `:116,187,240,259,273,295,320`), and
  calls `Com_Error(ERR_DROP, ...)` on oversized packets (`:150`) plus
  numerous `Com_Printf` traces. A "feed bytes in, assert header bytes out"
  golden test needs either (a) to mock `NET_SendPacket` to capture the
  outgoing buffer and stub the three cvars, or (b) to test the
  fragmentation/sequencing logic at a lower level than the full
  `Netchan_Transmit`/`Process` entry points.

**Verdict**: huffman.cpp and msg.cpp's bit-packing/delta-table routines are
clean, near-pure-function golden-test candidates once the `entityState_t`/
`playerState_t` header dependency is satisfied by reusing the already-ported
qshared types and 2-3 trivial function stubs (`Com_Error`, `Com_Printf`,
`cl_shownet` cvar read) are provided. Netchan framing golden tests are
feasible but need a `NET_SendPacket`-capturing shim, not a pure function
call.

---

## Design forks

1. **`msg_t` buffer ownership.** Raven's `msg_t` (`qcommon.h`) is a raw
   `byte* data` + cursor/bit-position fields, written in place by
   `MSG_Write*`/read in place by `MSG_Read*` — no allocation inside the hot
   path. Rust options: (a) keep the same shape, a struct wrapping
   `&mut [u8]`/cursor state, faithful to the seam and zero-copy; (b) an
   owned `Vec<u8>`-backed writer for ergonomics at the cost of an
   allocation per message. Given `msg_t`/`netchan_t` are already ported at
   the engine tier (`docs/type-port-todo.md`, Wave 5), this fork is really
   "does the already-chosen layout support borrow-based zero-copy
   `MSG_Write*`/`MSG_Read*` methods" — needs checking against the existing
   port rather than re-deciding from scratch.
2. **Huffman table init strategy.** The oracle seeds from a 256-entry
   `int[256]` table (`msg.cpp:2958`) by literally calling `Huff_addRef`
   thousands of times per symbol at startup (§2a) — slow but simple, and
   the *order* of `Huff_addRef` calls affects the resulting tree shape
   (adaptive Huffman's tree depends on insertion history, not just final
   counts), so a naive "build tree from these 256 frequencies" reimplementation
   could diverge from the real per-symbol-insertion-order tree. Fork:
   faithfully replay the same `for i in 0..256 { for j in 0..freq[i] {
   add_ref(i) } }` loop (slow, ~sum(freq) calls, but byte-exact) vs. derive
   an equivalent tree analytically (risks wire divergence — must be
   differential-tested against the oracle bit-for-bit, not just "looks like
   a valid Huffman tree").
3. **Static frequency-table sourcing: hand-transcribe vs. codegen.** The
   256-entry `msg_hData` table (`msg.cpp:2958-3215`) is a flat literal with
   no structure to exploit — hand-transcription risk is a single
   fat-fingered entry silently breaking wire compatibility with real JKA
   1.01 clients. Fork: hand-transcribe with a checksum/hash const-asserted
   against a value computed once from the oracle source, vs. a small
   build-time codegen step that greps the exact `msg_hData` block out of
   `msg.cpp` (guarding against the dead duplicate at `:2696-2954` by
   anchoring on the `// Q3 TA freq. table.` comment or line range) and
   emits the Rust array — the latter also protects against wave-of-the-hand
   errors if the oracle source is ever re-diffed.
4. **`netField_t` tables: codegen from oracle vs. hand transcription with
   count asserts.** 132 entities + 140 (or 3-table split: `playerStateFields`
   + `pilotPlayerStateFields` + `vehPlayerStateFields`) playerstate fields,
   each a `(name, offset, bits)` triple that must exactly match
   `entityState_t`/`playerState_t`'s Rust layout. The oracle itself
   enforces this with `assert(numFields+1 == sizeof(*from)/4)`
   (`msg.cpp:1085`) — the Rust port needs an equivalent static assert (a
   `const _: () = assert!(...)` over `size_of::<EntityState>()`). Fork: (a)
   hand-transcribe field name/bit-width tuples but generate offsets via
   `offset_of!` macros against the real Rust struct (self-correcting for
   layout, but the *name*→bits mapping and field *order* are still
   hand-copied and must be checked against `msg.cpp:859-1051` /
   `:1410-1568` line-by-line); (b) write a small oracle-source scraper that
   parses the `NETF(x), bits` lines directly out of `msg.cpp` and emits the
   Rust table (removes hand-transcription risk entirely, but is one more
   piece of tooling coupled to the oracle's exact macro-call formatting).
   Given porting-rules' general codegen skepticism elsewhere in this repo
   (manual per-type ports are the norm), (a) with a byte-exact
   differential-test harness (§8) as the real safety net is probably
   closer to house style than (b).
5. **Socket layer: `std::net::UdpSocket` vs. raw platform sockets.**
   Raven's win32/unix backends differ only in a handful of syscalls
   (`ioctlsocket` vs `ioctl` for non-blocking, `WSAStartup`, etc. —
   §5) around an otherwise-identical `sendto`/`recvfrom` core. `std::net::UdpSocket`
   (with `set_nonblocking`) covers the real cross-platform surface faithfully;
   the two custom bits to reproduce explicitly are (a) the bind-retry
   loop incrementing `net_port` on failure (`win_net.cpp:925-929`) and (b)
   `NET_Sleep`'s platform asymmetry — a genuine no-op on Windows but a real
   `select()`-based block on Unix dedicated servers (§5) — which needs a
   conscious Rust decision (mio/`std::net` readiness poll vs. a
   Windows-faithful busy-loop) rather than silently "fixing" the asymmetry.
6. **Loopback transport as a real backend, not a special case.**
   `NA_LOOPBACK` is handled by dedicated ring buffers
   (`loopbacks[2]`, §5) parallel to the real socket path, and SP (§7)
   *only* ever uses this path. Fork: model loopback as a distinct
   `NetTransport` impl behind the same trait as the real UDP socket (clean
   separation, and gives SP a transport that structurally can't touch a
   real socket, matching its actual capability) vs. one `NetTransport` with
   an `if addr.is_loopback()` branch mirroring Raven's `NET_SendPacket`
   shape exactly (closer to a literal line-for-line port, cheaper to
   verify against the oracle diff).
7. **`_OPTIMIZED_VEHICLE_NETWORKING` — compile-time config vs. runtime
   flag.** Raven picks the `playerStateFields` variant via `#ifdef` (§2c),
   unconditionally defined in the shipped build. Since jka-rust ports one
   faithful behavior (porting-rules §A.2: no speculative behavior), the
   dead `#else` branch (`msg.cpp:1829-1972`, never compiled in 1.01) should
   simply not be ported at all — only the pilot/vehicle-split
   tables (`:1410`, `:1570`, `:1736`) reflect real 1.01 wire traffic. Flag
   this explicitly so a future porter doesn't "helpfully" transcribe the
   unused table thinking it's the live one (an easy trap given both are
   named `playerStateFields` and the unused one is textually simpler).
8. **Reliable-command windows: full-resend fidelity vs. a NACK-based
   redesign.** §3d's mechanism (full-window resend every packet, no
   selective retransmit) is bandwidth-inefficient by modern standards but
   is exactly what a protocol-26-compatible peer expects to send/receive.
   Given DEC-06 is byte-for-byte wire compatibility, this is not really a
   fork — the oracle's exact scheme must be kept — but it's worth recording
   explicitly here as a "looks inefficient, is load-bearing for compat" trap
   so a future refactor pass doesn't reflexively "fix" it.
