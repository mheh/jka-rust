// snd-oracle stub for OpenAL `alc.h`. See openal_al.h for why the dropped arm
// still compiles.
//
// build.sh copies this file to the literal name `openal\alc.h`.
#ifndef SND_ORACLE_ALC_H
#define SND_ORACLE_ALC_H

#include "openal\al.h"

typedef void ALCdevice;
typedef void ALCcontext;
typedef int ALCenum;

#define ALC_NO_ERROR 0

static inline ALCdevice *alcOpenDevice(const ALubyte *) { snd_oracle_al_unreachable("alcOpenDevice"); return 0; }
static inline void alcCloseDevice(ALCdevice *) { snd_oracle_al_unreachable("alcCloseDevice"); }
static inline ALCcontext *alcCreateContext(ALCdevice *, ALint *) { snd_oracle_al_unreachable("alcCreateContext"); return 0; }
static inline void alcDestroyContext(ALCcontext *) { snd_oracle_al_unreachable("alcDestroyContext"); }
static inline void alcMakeContextCurrent(ALCcontext *) { snd_oracle_al_unreachable("alcMakeContextCurrent"); }
static inline ALCcontext *alcGetCurrentContext(void) { return 0; }
static inline ALCdevice *alcGetContextsDevice(ALCcontext *) { return 0; }
static inline ALCenum alcGetError(ALCdevice *) { return ALC_NO_ERROR; }

#endif // SND_ORACLE_ALC_H
