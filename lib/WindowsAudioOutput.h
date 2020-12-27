#include <wchar.h>
#define EXPORT __declspec(dllexport)

typedef unsigned long long u64;

struct AudioDevice
{
  const wchar_t *id;
  const wchar_t *name;
  bool is_default;
};

struct AudioDevices
{
  u64 len;
  AudioDevice **device;
};

extern "C"
{
  EXPORT AudioDevices GetAudioDevices();
  EXPORT void ReleaseAudioDevices(AudioDevices);
  EXPORT void SetDefaultAudioDevice(const wchar_t *);
  EXPORT u64 GetMasterVolume();
  EXPORT void SetMasterVolume(u64);
}
