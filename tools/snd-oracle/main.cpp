// snd-oracle — the scripted driver over the unmodified Raven sound TUs.
//
//   snd_dump <script.snd> <ring.bin>
//
// The driver reads a scenario script, calls the Raven sound API in the scripted
// order, writes the canonical text dump to stdout, and writes the final `dma_t`
// ring bytes to <ring.bin>. README.md holds the command schema.
//
// The Rust mixer port (wayfinder ticket gh#24) must reproduce both goldens.
#include "../qcommon/exe_headers.h"
#include "snd_local.h"
#include "snd_music.h"
#include "snd_ambient.h"
#include "host.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>

// Raven's sound state. snd_local.h declares the mixer half. The rest lives in
// snd_dma.cpp, which declares none of it in a header.
// Source: `oracle/codemp/client/snd_dma.cpp:129-194`
extern int s_soundStarted;
extern qboolean s_soundMuted;
extern int s_soundtime;
extern int listener_number;
extern vec3_t listener_axis[3];
extern sfx_t s_knownSfx[];
extern int s_numSfx;
extern int numLoopSounds;
extern int s_entityWavVol[];
extern int s_entityWavVol_back[];

// The background-music state. `MusicInfo_t` and the four state globals are
// file-local to snd_dma.cpp, so the driver reads them through the two
// accessors build.sh appends to that copy.
// Source: `oracle/codemp/client/snd_dma.cpp:38-109`
extern "C" void snd_oracle_music_track(int track, int *out);
extern "C" void snd_oracle_music_state(int *out, const char **loop, const char **set);

// `Music_GetLevelSetName` is declared inside snd_dma.cpp, not in snd_music.h.
// Source: `oracle/codemp/client/snd_music.cpp:1140`
extern const char *Music_GetLevelSetName(void);

// `S_StartBackgroundTrack` and `S_RestartMusic` live in snd_public.h.
extern void S_StartBackgroundTrack(const char *intro, const char *loop, int bCalledByCGameStart);
extern void S_RestartMusic(void);
extern void S_StopBackgroundTrack(void);

#define SND_ORACLE_MAX_SLOTS 16
static sfxHandle_t s_slotHandles[SND_ORACLE_MAX_SLOTS];

// The device has consumed this many stereo frames since S_Init. The clock reads
// off this counter, so the scripted clock never drifts from the scripted cursor.
static long long s_framesConsumed = 0;

// --- dump helpers -----------------------------------------------------------

// FNV-1a over a byte range. The digest localises a mismatch without printing
// 64 KB of hex into every golden.
static unsigned int snd_oracle_fnv1a(const void *data, size_t len)
{
	const unsigned char *p = (const unsigned char *)data;
	unsigned int hash = 2166136261u;
	for (size_t i = 0; i < len; i++) {
		hash ^= p[i];
		hash *= 16777619u;
	}
	return hash;
}

static void snd_oracle_dump_state(const char *tag)
{
	printf("STATE %s\n", tag);
	printf("  clock %u dmapos %d frames %lld\n", snd_oracle_clock_ms, snd_oracle_dma_pos, s_framesConsumed);
	printf("  started %d muted %d soundtime %d paintedtime %d rawend %d\n",
		s_soundStarted, (int)s_soundMuted, s_soundtime, s_paintedtime, s_rawend);
	printf("  dma channels %d samples %d samplebits %d speed %d chunk %d\n",
		dma.channels, dma.samples, dma.samplebits, dma.speed, dma.submission_chunk);
	printf("  listener %d org %.6f %.6f %.6f\n",
		listener_number, listener_origin[0], listener_origin[1], listener_origin[2]);
	for (int a = 0; a < 3; a++) {
		printf("  axis%d %.6f %.6f %.6f\n", a, listener_axis[a][0], listener_axis[a][1], listener_axis[a][2]);
	}
	printf("  loops %d prints %d dprints %d\n", numLoopSounds, snd_oracle_print_count, snd_oracle_dprint_count);

	for (int i = 0; i < MAX_CHANNELS; i++) {
		channel_t *ch = &s_channels[i];
		if (!ch->thesfx) {
			continue;
		}
		printf("  ch %d ent %d chan %d lv %d rv %d mv %d start %u fixed %d loop %d org %.6f %.6f %.6f sfx %s\n",
			i, ch->entnum, ch->entchannel, ch->leftvol, ch->rightvol, ch->master_vol,
			ch->startSample, ch->fixed_origin, ch->loopSound,
			ch->origin[0], ch->origin[1], ch->origin[2], ch->thesfx->sSoundName);
	}
}

static void snd_oracle_dump_sfx(const char *tag)
{
	printf("SFX %s count %d\n", tag, s_numSfx);
	for (int i = 0; i < s_numSfx; i++) {
		sfx_t *sfx = &s_knownSfx[i];
		unsigned int digest = 0;
		if (sfx->pSoundData && sfx->iSoundLengthInSamples > 0) {
			digest = snd_oracle_fnv1a(sfx->pSoundData, (size_t)sfx->iSoundLengthInSamples * 2);
		}
		printf("  sfx %d name %s samples %d volrange %.6f method %d default %d inmem %d data %08x\n",
			i, sfx->sSoundName, sfx->iSoundLengthInSamples, sfx->fVolRange,
			(int)sfx->eSoundCompressionMethod, sfx->bDefaultSound, sfx->bInMemory, digest);
	}
}

// The lip-sync dump covers the low entity slots only. A scenario that wants a
// higher entity number raises this and re-records.
#define SND_ORACLE_LIPSYNC_ENTS 8

// S_DoLipSynchs writes the amplitude bucket of every voice channel into
// s_entityWavVol, and S_CheckAmplitude keeps the previous value in the backup
// table. Both are the only observable output of the lip-sync path.
static void snd_oracle_dump_lipsync(const char *tag)
{
	printf("LIPSYNC %s\n", tag);
	for (int i = 0; i < SND_ORACLE_LIPSYNC_ENTS; i++) {
		printf("  ent %d vol %d back %d\n", i, s_entityWavVol[i], s_entityWavVol_back[i]);
	}
}

// The background-music state block. The nine per-track fields come through the
// accessor build.sh appends to snd_dma.cpp, because MusicInfo_t has no header.
static void snd_oracle_dump_music(const char *tag)
{
	int state[3];
	const char *loop = NULL;
	const char *set = NULL;
	snd_oracle_music_state(state, &loop, &set);

	printf("MUSIC %s\n", tag);
	printf("  dynamic %d actual %d request %d loop %s\n", state[0], state[1], state[2], loop);
	printf("  set %s\n", set);
	for (int track = 0; track < eBGRNDTRACK_NUMBEROF; track++) {
		int f[9];
		snd_oracle_music_track(track, f);
		printf("  track %d mp3 %d file %d active %d exists %d xfade %d samples %d rate %d chans %d width %d\n",
			track, f[0], f[1], f[2], f[3], f[4], f[5], f[6], f[7], f[8]);
	}
}

// The ring is 64 KB. The dump carries one digest per 4 KB block plus the sample
// extremes, so a mismatch names the block it lives in.
#define SND_ORACLE_RING_BLOCK 4096
#define SND_ORACLE_RING_MAX 0x10000

// S_Shutdown frees the ring, so every dump keeps a copy. The driver writes the
// last copy to the .bin golden after the script ends.
static unsigned char s_ringSnapshot[SND_ORACLE_RING_MAX];
static int s_ringSnapshotBytes = 0;

static void snd_oracle_dump_ring(const char *tag)
{
	int bytes = dma.samples * (dma.samplebits / 8);
	if (!dma.buffer || bytes <= 0 || bytes > SND_ORACLE_RING_MAX) {
		printf("RING %s bytes 0 whole 00000000\n", tag);
		return;
	}
	memcpy(s_ringSnapshot, dma.buffer, (size_t)bytes);
	s_ringSnapshotBytes = bytes;

	printf("RING %s bytes %d whole %08x\n", tag, bytes, snd_oracle_fnv1a(dma.buffer, (size_t)bytes));

	for (int off = 0; off < bytes; off += SND_ORACLE_RING_BLOCK) {
		int len = bytes - off;
		if (len > SND_ORACLE_RING_BLOCK) {
			len = SND_ORACLE_RING_BLOCK;
		}
		const short *s = (const short *)(dma.buffer + off);
		int count = len / 2;
		int lo = 0;
		int hi = 0;
		int nonzero = 0;
		for (int i = 0; i < count; i++) {
			if (s[i] < lo) {
				lo = s[i];
			}
			if (s[i] > hi) {
				hi = s[i];
			}
			if (s[i]) {
				nonzero++;
			}
		}
		printf("  blk %d crc %08x min %d max %d nonzero %d\n",
			off / SND_ORACLE_RING_BLOCK, snd_oracle_fnv1a(dma.buffer + off, (size_t)len), lo, hi, nonzero);
	}
}

// --- script parsing ---------------------------------------------------------

static char *snd_oracle_next(char **cursor)
{
	char *p = *cursor;
	while (*p == ' ' || *p == '\t') {
		p++;
	}
	if (!*p) {
		*cursor = p;
		return NULL;
	}
	char *start = p;
	while (*p && *p != ' ' && *p != '\t') {
		p++;
	}
	if (*p) {
		*p++ = 0;
	}
	*cursor = p;
	return start;
}

static const char *snd_oracle_word(char **cursor, const char *command)
{
	const char *w = snd_oracle_next(cursor);
	if (!w) {
		fprintf(stderr, "snd-oracle: %s wants more arguments\n", command);
		exit(1);
	}
	return w;
}

static float snd_oracle_float(char **cursor, const char *command) { return (float)atof(snd_oracle_word(cursor, command)); }
static int snd_oracle_int(char **cursor, const char *command) { return atoi(snd_oracle_word(cursor, command)); }

static void snd_oracle_vec(char **cursor, const char *command, vec3_t out)
{
	out[0] = snd_oracle_float(cursor, command);
	out[1] = snd_oracle_float(cursor, command);
	out[2] = snd_oracle_float(cursor, command);
}

static sfxHandle_t snd_oracle_slot(char **cursor, const char *command)
{
	int slot = snd_oracle_int(cursor, command);
	if (slot < 0 || slot >= SND_ORACLE_MAX_SLOTS) {
		fprintf(stderr, "snd-oracle: %s names slot %d, which is out of range\n", command, slot);
		exit(1);
	}
	return s_slotHandles[slot];
}

// The device consumed `frames` stereo frames. The cursor moves in the units
// SNDDMA_GetDMAPos reports, which is `dma.channels` per frame.
static void snd_oracle_advance(int frames)
{
	s_framesConsumed += frames;
	snd_oracle_dma_pos += frames * dma.channels;
	snd_oracle_clock_ms = (unsigned int)((s_framesConsumed * 1000) / dma.speed);
}

int main(int argc, char **argv)
{
	if (argc != 3) {
		fprintf(stderr, "usage: %s <script.snd> <ring.bin>\n", argv[0]);
		return 2;
	}

	FILE *script = fopen(argv[1], "rb");
	if (!script) {
		fprintf(stderr, "snd-oracle: cannot open %s\n", argv[1]);
		return 2;
	}

	snd_oracle_host_init();

	printf("== snd-oracle %s ==\n", argv[1]);

	char line[512];
	while (fgets(line, sizeof(line), script)) {
		char *nl = strpbrk(line, "\r\n");
		if (nl) {
			*nl = 0;
		}
		char *cursor = line;
		const char *cmd = snd_oracle_next(&cursor);
		if (!cmd || cmd[0] == '#') {
			continue;
		}

		if (strcmp(cmd, "cvar") == 0) {
			const char *name = snd_oracle_word(&cursor, cmd);
			const char *value = snd_oracle_word(&cursor, cmd);
			snd_oracle_cvar_set(name, value);

		} else if (strcmp(cmd, "init") == 0) {
			S_Init();

		} else if (strcmp(cmd, "beginreg") == 0) {
			S_BeginRegistration();

		} else if (strcmp(cmd, "register") == 0) {
			int slot = snd_oracle_int(&cursor, cmd);
			const char *path = snd_oracle_word(&cursor, cmd);
			if (slot < 0 || slot >= SND_ORACLE_MAX_SLOTS) {
				fprintf(stderr, "snd-oracle: register names slot %d, which is out of range\n", slot);
				return 1;
			}
			s_slotHandles[slot] = S_RegisterSound(path);
			printf("REGISTER slot %d handle %d path %s\n", slot, s_slotHandles[slot], path);

		} else if (strcmp(cmd, "length") == 0) {
			sfxHandle_t h = snd_oracle_slot(&cursor, cmd);
			printf("LENGTH handle %d ms %.6f\n", h, S_GetSampleLengthInMilliSeconds(h));

		} else if (strcmp(cmd, "respatialize") == 0) {
			int entnum = snd_oracle_int(&cursor, cmd);
			vec3_t head;
			snd_oracle_vec(&cursor, cmd, head);
			int inwater = snd_oracle_int(&cursor, cmd);
			vec3_t axis[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
			S_Respatialize(entnum, head, axis, inwater);

		} else if (strcmp(cmd, "respatializeaxis") == 0) {
			int entnum = snd_oracle_int(&cursor, cmd);
			vec3_t head;
			vec3_t axis[3];
			snd_oracle_vec(&cursor, cmd, head);
			snd_oracle_vec(&cursor, cmd, axis[0]);
			snd_oracle_vec(&cursor, cmd, axis[1]);
			snd_oracle_vec(&cursor, cmd, axis[2]);
			int inwater = snd_oracle_int(&cursor, cmd);
			S_Respatialize(entnum, head, axis, inwater);

		} else if (strcmp(cmd, "entitypos") == 0) {
			int entnum = snd_oracle_int(&cursor, cmd);
			vec3_t org;
			snd_oracle_vec(&cursor, cmd, org);
			S_UpdateEntityPosition(entnum, org);

		} else if (strcmp(cmd, "startsound") == 0) {
			vec3_t org;
			snd_oracle_vec(&cursor, cmd, org);
			int entnum = snd_oracle_int(&cursor, cmd);
			int entchannel = snd_oracle_int(&cursor, cmd);
			S_StartSound(org, entnum, entchannel, snd_oracle_slot(&cursor, cmd));

		} else if (strcmp(cmd, "startsoundent") == 0) {
			// A null origin makes the channel follow the entity position.
			int entnum = snd_oracle_int(&cursor, cmd);
			int entchannel = snd_oracle_int(&cursor, cmd);
			S_StartSound(NULL, entnum, entchannel, snd_oracle_slot(&cursor, cmd));

		} else if (strcmp(cmd, "startlocal") == 0) {
			sfxHandle_t h = snd_oracle_slot(&cursor, cmd);
			S_StartLocalSound(h, snd_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "startlocalloop") == 0) {
			S_StartLocalLoopingSound(snd_oracle_slot(&cursor, cmd));

		} else if (strcmp(cmd, "startambient") == 0) {
			vec3_t org;
			snd_oracle_vec(&cursor, cmd, org);
			int entnum = snd_oracle_int(&cursor, cmd);
			int volume = snd_oracle_int(&cursor, cmd);
			S_StartAmbientSound(org, entnum, (unsigned char)volume, snd_oracle_slot(&cursor, cmd));

		} else if (strcmp(cmd, "ambientloop") == 0) {
			vec3_t org;
			snd_oracle_vec(&cursor, cmd, org);
			int volume = snd_oracle_int(&cursor, cmd);
			S_AddAmbientLoopingSound(org, (unsigned char)volume, snd_oracle_slot(&cursor, cmd));

		} else if (strcmp(cmd, "music") == 0) {
			const char *intro = snd_oracle_word(&cursor, cmd);
			const char *loop = snd_oracle_next(&cursor);
			const char *cgameStart = loop ? snd_oracle_next(&cursor) : NULL;
			S_StartBackgroundTrack(intro, loop ? loop : "", cgameStart ? atoi(cgameStart) : 0);

		} else if (strcmp(cmd, "restartmusic") == 0) {
			S_RestartMusic();

		} else if (strcmp(cmd, "stopmusic") == 0) {
			S_StopBackgroundTrack();

		} else if (strcmp(cmd, "musicdata") == 0) {
			const char *label = snd_oracle_word(&cursor, cmd);
			printf("MUSICDATA label %s available %d\n", label, (int)Music_DynamicDataAvailable(label));

		} else if (strcmp(cmd, "musicfile") == 0) {
			int state = snd_oracle_int(&cursor, cmd);
			const char *name = Music_GetFileNameForState((MusicState_e)state);
			printf("MUSICFILE state %d name %s\n", state, name ? name : "<none>");

		} else if (strcmp(cmd, "musicinterrupt") == 0) {
			int a = snd_oracle_int(&cursor, cmd);
			int b = snd_oracle_int(&cursor, cmd);
			printf("MUSICINTERRUPT from %d to %d allowed %d\n",
				a, b, (int)Music_StateCanBeInterrupted((MusicState_e)a, (MusicState_e)b));

		} else if (strcmp(cmd, "musictransition") == 0) {
			float elapsed = snd_oracle_float(&cursor, cmd);
			int state = snd_oracle_int(&cursor, cmd);
			MusicState_e transition = eBGRNDTRACK_EXPLORE;
			float entry = 0.0f;
			int allowed = (int)Music_AllowedToTransition(elapsed, (MusicState_e)state, &transition, &entry);
			printf("MUSICTRANSITION at %.6f state %d allowed %d to %d entry %.6f\n",
				elapsed, state, allowed, allowed ? (int)transition : -1, allowed ? entry : 0.0f);

		} else if (strcmp(cmd, "musicentrytime") == 0) {
			int state = snd_oracle_int(&cursor, cmd);
			printf("MUSICENTRYTIME state %d time %.6f\n", state, Music_GetRandomEntryTime((MusicState_e)state));

		} else if (strcmp(cmd, "musicsetname") == 0) {
			printf("MUSICSETNAME %s\n", Music_GetLevelSetName());

		} else if (strcmp(cmd, "asprecache") == 0) {
			AS_AddPrecacheEntry(snd_oracle_word(&cursor, cmd));

		} else if (strcmp(cmd, "asparse") == 0) {
			AS_ParseSets();

		} else if (strcmp(cmd, "asupdate") == 0) {
			const char *name = snd_oracle_word(&cursor, cmd);
			vec3_t org;
			snd_oracle_vec(&cursor, cmd, org);
			snd_oracle_realtime = snd_oracle_int(&cursor, cmd);
			S_UpdateAmbientSet(name, org);

		} else if (strcmp(cmd, "aslocal") == 0) {
			const char *name = snd_oracle_word(&cursor, cmd);
			vec3_t listener;
			vec3_t org;
			snd_oracle_vec(&cursor, cmd, listener);
			snd_oracle_vec(&cursor, cmd, org);
			int entID = snd_oracle_int(&cursor, cmd);
			int time = snd_oracle_int(&cursor, cmd);
			snd_oracle_realtime = snd_oracle_int(&cursor, cmd);
			printf("LOCALSET name %s time %d\n", name, S_AddLocalSet(name, listener, org, entID, time));

		} else if (strcmp(cmd, "asbmodel") == 0) {
			const char *name = snd_oracle_word(&cursor, cmd);
			int stage = snd_oracle_int(&cursor, cmd);
			printf("BMODELSOUND name %s stage %d handle %d\n", name, stage, AS_GetBModelSound(name, stage));

		} else if (strcmp(cmd, "dumpmusic") == 0) {
			snd_oracle_dump_music(snd_oracle_word(&cursor, cmd));

		} else if (strcmp(cmd, "clearloops") == 0) {
			S_ClearLoopingSounds();

		} else if (strcmp(cmd, "addloop") == 0) {
			int entnum = snd_oracle_int(&cursor, cmd);
			vec3_t org;
			vec3_t vel;
			snd_oracle_vec(&cursor, cmd, org);
			snd_oracle_vec(&cursor, cmd, vel);
			S_AddLoopingSound(entnum, org, vel, snd_oracle_slot(&cursor, cmd));

		} else if (strcmp(cmd, "stoploop") == 0) {
			S_StopLoopingSound(snd_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "mute") == 0) {
			int entnum = snd_oracle_int(&cursor, cmd);
			S_MuteSound(entnum, snd_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "stopsounds") == 0) {
			S_StopSounds();

		} else if (strcmp(cmd, "stopall") == 0) {
			S_StopAllSounds();

		} else if (strcmp(cmd, "disable") == 0) {
			S_DisableSounds();

		} else if (strcmp(cmd, "rawsamples") == 0) {
			// A scripted raw-stream block: `rawsamples <frames> <rate> <amplitude>`
			// fills a stereo ramp so the raw path has deterministic content.
			int frames = snd_oracle_int(&cursor, cmd);
			int rate = snd_oracle_int(&cursor, cmd);
			int amplitude = snd_oracle_int(&cursor, cmd);
			short *block = (short *)malloc((size_t)frames * 2 * sizeof(short));
			for (int i = 0; i < frames; i++) {
				block[i * 2 + 0] = (short)((i % 64) * amplitude / 64);
				block[i * 2 + 1] = (short)(-((i % 64) * amplitude / 64));
			}
			S_RawSamples(frames, rate, 2, 2, (const byte *)block, 1.0f, qtrue);
			free(block);

		} else if (strcmp(cmd, "advance") == 0) {
			snd_oracle_advance(snd_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "update") == 0) {
			S_Update();

		} else if (strcmp(cmd, "dumpstate") == 0) {
			snd_oracle_dump_state(snd_oracle_word(&cursor, cmd));

		} else if (strcmp(cmd, "dumpsfx") == 0) {
			snd_oracle_dump_sfx(snd_oracle_word(&cursor, cmd));

		} else if (strcmp(cmd, "dumpring") == 0) {
			snd_oracle_dump_ring(snd_oracle_word(&cursor, cmd));

		} else if (strcmp(cmd, "dumplipsync") == 0) {
			snd_oracle_dump_lipsync(snd_oracle_word(&cursor, cmd));

		} else if (strcmp(cmd, "shutdown") == 0) {
			if (dma.buffer && dma.samples > 0) {
				int bytes = dma.samples * (dma.samplebits / 8);
				if (bytes <= SND_ORACLE_RING_MAX) {
					memcpy(s_ringSnapshot, dma.buffer, (size_t)bytes);
					s_ringSnapshotBytes = bytes;
				}
			}
			S_Shutdown();

		} else {
			fprintf(stderr, "snd-oracle: unknown command '%s'\n", cmd);
			return 1;
		}
	}
	fclose(script);

	if (dma.buffer && dma.samples > 0) {
		int bytes = dma.samples * (dma.samplebits / 8);
		if (bytes <= SND_ORACLE_RING_MAX) {
			memcpy(s_ringSnapshot, dma.buffer, (size_t)bytes);
			s_ringSnapshotBytes = bytes;
		}
	}

	FILE *ring = fopen(argv[2], "wb");
	if (!ring) {
		fprintf(stderr, "snd-oracle: cannot write %s\n", argv[2]);
		return 2;
	}
	fwrite(s_ringSnapshot, 1, (size_t)s_ringSnapshotBytes, ring);
	fclose(ring);

	printf("== end ==\n");
	return 0;
}
