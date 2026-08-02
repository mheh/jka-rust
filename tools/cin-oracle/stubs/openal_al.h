// snd-oracle stub for OpenAL `al.h`.
//
// DEC-57.4 drops the OpenAL/EAX arm. Raven gates that arm on the `s_UseOpenAL`
// cvar, not on a preprocessor symbol, so the harness must still compile and link
// the arm. Every entry point below aborts, and the harness keeps `s_UseOpenAL`
// at 0, which proves the goldens never enter the arm.
//
// build.sh copies this file to the literal name `openal\al.h`, because Raven
// writes the include with a Windows path separator (snd_local.h:12).
#ifndef SND_ORACLE_AL_H
#define SND_ORACLE_AL_H

typedef char ALbyte;
typedef unsigned char ALubyte;
typedef int ALint;
typedef unsigned int ALuint;
typedef int ALsizei;
typedef int ALenum;
typedef float ALfloat;
typedef double ALdouble;
typedef char ALboolean;
typedef void ALvoid;

#define AL_FALSE 0
#define AL_TRUE 1
#define AL_NO_ERROR 0
#define AL_INVALID_NAME 0xA001
#define AL_SOURCE_RELATIVE 0x202
#define AL_LOOPING 0x1007
#define AL_BUFFER 0x1009
#define AL_GAIN 0x100A
#define AL_POSITION 0x1004
#define AL_VELOCITY 0x1006
#define AL_ORIENTATION 0x100F
#define AL_REFERENCE_DISTANCE 0x1020
#define AL_SOURCE_STATE 0x1010
#define AL_PLAYING 0x1012
#define AL_STOPPED 0x1014
#define AL_BUFFERS_PROCESSED 0x1016
#define AL_SIZE 0x2004
#define AL_FORMAT_MONO16 0x1101
#define AL_FORMAT_STEREO16 0x1103

// host.cpp prints the name and aborts.
void snd_oracle_al_unreachable(const char *name);

static inline void alGenBuffers(ALsizei, ALuint *) { snd_oracle_al_unreachable("alGenBuffers"); }
static inline void alDeleteBuffers(ALsizei, const ALuint *) { snd_oracle_al_unreachable("alDeleteBuffers"); }
static inline void alBufferData(ALuint, ALenum, const ALvoid *, ALsizei, ALsizei) { snd_oracle_al_unreachable("alBufferData"); }
static inline void alGetBufferi(ALuint, ALenum, ALint *) { snd_oracle_al_unreachable("alGetBufferi"); }
static inline void alGenSources(ALsizei, ALuint *) { snd_oracle_al_unreachable("alGenSources"); }
static inline void alDeleteSources(ALsizei, const ALuint *) { snd_oracle_al_unreachable("alDeleteSources"); }
static inline void alSourcei(ALuint, ALenum, ALint) { snd_oracle_al_unreachable("alSourcei"); }
static inline void alSourcef(ALuint, ALenum, ALfloat) { snd_oracle_al_unreachable("alSourcef"); }
static inline void alSourcefv(ALuint, ALenum, const ALfloat *) { snd_oracle_al_unreachable("alSourcefv"); }
static inline void alGetSourcei(ALuint, ALenum, ALint *) { snd_oracle_al_unreachable("alGetSourcei"); }
static inline void alSourcePlay(ALuint) { snd_oracle_al_unreachable("alSourcePlay"); }
static inline void alSourceStop(ALuint) { snd_oracle_al_unreachable("alSourceStop"); }
static inline void alSourceQueueBuffers(ALuint, ALsizei, const ALuint *) { snd_oracle_al_unreachable("alSourceQueueBuffers"); }
static inline void alSourceUnqueueBuffers(ALuint, ALsizei, ALuint *) { snd_oracle_al_unreachable("alSourceUnqueueBuffers"); }
static inline void alListenerf(ALenum, ALfloat) { snd_oracle_al_unreachable("alListenerf"); }
static inline void alListenerfv(ALenum, const ALfloat *) { snd_oracle_al_unreachable("alListenerfv"); }
static inline ALenum alGetError(void) { return AL_NO_ERROR; }
static inline ALboolean alIsExtensionPresent(const ALubyte *) { return AL_FALSE; }
static inline void *alGetProcAddress(const ALubyte *) { return 0; }

#endif // SND_ORACLE_AL_H
