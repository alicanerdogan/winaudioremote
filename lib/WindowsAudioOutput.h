#include <stdio.h>
#include <wchar.h>
#include <tchar.h>
#include "windows.h"
#include "Mmdeviceapi.h"
#include "PolicyConfig.h"
#include "Propidl.h"
#include "Functiondiscoverykeys_devpkey.h"
#define EXPORT __declspec(dllexport)

typedef unsigned long long u64;

struct AudioDevice
{
  const wchar_t *id;
  const wchar_t *name;
  boolean is_default;
};

struct AudioDevices
{
  u64 len;
  AudioDevice **device;
};

extern "C"
{
  EXPORT AudioDevices GetAudioDevices(void);
  EXPORT void ReleaseAudioDevices(AudioDevices);
  EXPORT void SetDefaultAudioDevice(const wchar_t *);
}
