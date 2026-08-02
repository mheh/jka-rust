// snd-oracle — the harness state the driver shares with the host seam.
#ifndef SND_ORACLE_HOST_H
#define SND_ORACLE_HOST_H

// The scripted clock, in milliseconds. Nothing in the harness reads a wall clock.
extern unsigned int snd_oracle_clock_ms;

// The scripted device read cursor, in the units SNDDMA_GetDMAPos returns.
extern int snd_oracle_dma_pos;

// Console call counts. The goldens carry the counts, never the text.
extern int snd_oracle_print_count;
extern int snd_oracle_dprint_count;

// The scripted `cls.realtime` the ambient system reads. build.sh points the
// snd_ambient copy at this instead of the client global.
extern int snd_oracle_realtime;

// The C runtime generator `Music_GetRandomEntryTime` draws from. build.sh
// routes the oracle's `rand()` here.
int snd_oracle_rand(void);

// Seeds a cvar before S_Init, the way a config file would.
cvar_t *snd_oracle_cvar_set(const char *name, const char *value);

// Registers the engine cvars the sound code reads but never creates. The driver
// calls it once before the script runs.
void snd_oracle_host_init(void);

#endif // SND_ORACLE_HOST_H
