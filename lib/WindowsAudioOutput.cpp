#pragma comment(lib, "ole32.lib")

#include "WindowsAudioOutput.h"

#ifdef __DEBUG
#define _CRTDBG_MAP_ALLOC
#include <stdlib.h>
#include <crtdbg.h>

void heapdump(void)
{
  _CrtSetReportMode(_CRT_WARN, _CRTDBG_MODE_FILE);
  _CrtSetReportFile(_CRT_WARN, _CRTDBG_FILE_STDOUT);
  _CrtSetReportMode(_CRT_ERROR, _CRTDBG_MODE_FILE);
  _CrtSetReportFile(_CRT_ERROR, _CRTDBG_FILE_STDOUT);
  _CrtSetReportMode(_CRT_ASSERT, _CRTDBG_MODE_FILE);
  _CrtSetReportFile(_CRT_ASSERT, _CRTDBG_FILE_STDOUT);
  _CrtDumpMemoryLeaks();
}
#endif

extern "C"
{
  EXPORT AudioDevices GetAudioDevices(void)
  {
    HRESULT hr = CoInitialize(NULL);
    if (!SUCCEEDED(hr))
    {
      return AudioDevices{0, NULL};
    }

    IMMDeviceEnumerator *pEnum = NULL;
    hr = CoCreateInstance(__uuidof(MMDeviceEnumerator), NULL, CLSCTX_ALL, __uuidof(IMMDeviceEnumerator), (void **)&pEnum);
    if (!SUCCEEDED(hr))
    {
      return AudioDevices{0, NULL};
    }

    IMMDevice *pDefaultDevice;
    hr = pEnum->GetDefaultAudioEndpoint(eRender, eMultimedia, &pDefaultDevice);
    if (!SUCCEEDED(hr))
    {
      pEnum->Release();
      return AudioDevices{0, NULL};
    }

    LPWSTR defaultDeviceId = NULL;
    hr = pDefaultDevice->GetId(&defaultDeviceId);
    if (!SUCCEEDED(hr))
    {
      defaultDeviceId = NULL;
    }
    pDefaultDevice->Release();

    IMMDeviceCollection *pDevices;
    hr = pEnum->EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE, &pDevices);
    if (!SUCCEEDED(hr))
    {
      pEnum->Release();
      return AudioDevices{0, NULL};
    }

    UINT count;
    pDevices->GetCount(&count);
    if (!SUCCEEDED(hr))
    {
      pDevices->Release();
      pEnum->Release();
      return AudioDevices{0, NULL};
    }

    AudioDevice **devices = (AudioDevice **)malloc(sizeof(AudioDevice *) * count);

    for (UINT i = 0; i < count; ++i)
    {
      IMMDevice *pDevice;
      HRESULT hr = pDevices->Item(i, &pDevice);
      if (!SUCCEEDED(hr))
      {
        continue;
      }

      LPWSTR wstrID = NULL;
      hr = pDevice->GetId(&wstrID);
      if (!SUCCEEDED(hr))
      {
        pDevice->Release();
        continue;
      }

      IPropertyStore *pStore;
      hr = pDevice->OpenPropertyStore(STGM_READ, &pStore);
      if (!SUCCEEDED(hr))
      {
        pDevice->Release();
        continue;
      }

      PROPVARIANT friendlyName;
      PropVariantInit(&friendlyName);
      hr = pStore->GetValue(PKEY_Device_FriendlyName, &friendlyName);
      if (!SUCCEEDED(hr))
      {
        pDevice->Release();
        continue;
      }

      AudioDevice *audio_device = (AudioDevice *)malloc(sizeof(AudioDevice));

      wchar_t *device_id_clone = new wchar_t[wcslen(wstrID) + 1];
      wcscpy(device_id_clone, wstrID);
      audio_device->id = device_id_clone;

      wchar_t *device_name_clone = new wchar_t[wcslen(friendlyName.pwszVal) + 1];
      wcscpy(device_name_clone, friendlyName.pwszVal);
      audio_device->name = device_name_clone;

      audio_device->is_default = wcscmp(wstrID, defaultDeviceId) == 0;
      devices[i] = audio_device;

      PropVariantClear(&friendlyName);
      pStore->Release();
      pDevice->Release();
    }

    pDevices->Release();
    return AudioDevices{count,
                        devices};
  }

  EXPORT void ReleaseAudioDevices(AudioDevices audio_devices)
  {
    for (u64 index = 0; index < audio_devices.len; ++index)
    {
      AudioDevice *audio_device = audio_devices.device[index];
      free((wchar_t *)audio_device->id);
      free((wchar_t *)audio_device->name);
      free(audio_device);
    }
    free(audio_devices.device);
#ifdef __DEBUG
    heapdump();
#endif
  }

  EXPORT void SetDefaultAudioDevice(const wchar_t *device_id)
  {
    IPolicyConfigVista *pPolicyConfig;
    ERole reserved = eConsole;

    HRESULT hr = CoCreateInstance(__uuidof(CPolicyConfigVistaClient), NULL, CLSCTX_ALL, __uuidof(IPolicyConfigVista), (LPVOID *)&pPolicyConfig);

    if (SUCCEEDED(hr))
    {
      hr = pPolicyConfig->SetDefaultEndpoint(device_id, reserved);
      pPolicyConfig->Release();
    }

    SUCCEEDED(hr);

#ifdef __DEBUG
    heapdump();
#endif
  }
}
