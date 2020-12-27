extern crate libloading;
extern crate widestring;
extern crate winapi;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use winapi::ctypes::{__uint64, wchar_t};

#[repr(C)]
struct CAudioDevice {
    id: *const winapi::ctypes::wchar_t,
    name: *const winapi::ctypes::wchar_t,
    is_default: bool,
}

#[repr(C)]
struct CAudioDevices {
    len: __uint64,
    device: *const *const CAudioDevice,
}

pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub fn export_dll() {
    let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    path.push("WindowsAudioOutput");
    path.set_extension("dll");
    let bytes = include_bytes!("../lib/WindowsAudioOutput.dll");
    let mut file = File::create(path).unwrap();
    file.write_all(bytes).unwrap();
}

pub fn get_devices() -> Result<Vec<AudioDevice>, Box<dyn std::error::Error>> {
    // Note: this example does work on Windows
    let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    path.push("WindowsAudioOutput");
    path.set_extension("dll");

    let lib = libloading::Library::new(path.as_path())?;

    unsafe {
        let func: libloading::Symbol<unsafe extern "C" fn() -> CAudioDevices> =
            lib.get(b"GetAudioDevices")?;
        let release_audio_devices_fn: libloading::Symbol<
            unsafe extern "C" fn(CAudioDevices) -> (),
        > = lib.get(b"ReleaseAudioDevices")?;

        let devices = func();

        let ptr_size = mem::size_of::<usize>();
        let mut vec: Vec<AudioDevice> = vec![];
        for i in 0..(devices.len as usize) {
            let ptr_to_c_audio_device = (devices.device as usize) + (ptr_size * i);
            let ptr = ptr_to_c_audio_device as *const *const CAudioDevice;
            let audio_device = AudioDevice {
                id: widestring::U16CString::from_ptr_str((**(ptr)).id).to_string_lossy(),
                name: widestring::U16CString::from_ptr_str((**(ptr)).name).to_string_lossy(),
                is_default: (**(ptr)).is_default,
            };
            vec.push(audio_device);
        }

        release_audio_devices_fn(devices);
        return Ok(vec);
    }
}

pub fn set_default_audio_device(
    audio_device_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Note: this example does work on Windows
    let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    path.push("WindowsAudioOutput");
    path.set_extension("dll");

    let lib = libloading::Library::new(path.as_path())?;
    unsafe {
        let func: libloading::Symbol<unsafe extern "C" fn(*const wchar_t) -> ()> =
            lib.get(b"SetDefaultAudioDevice")?;
        func(
            widestring::U16CString::from_str(audio_device_id)
                .unwrap()
                .as_ptr(),
        );
        Ok(())
    }
}

pub fn get_master_volume() -> Result<u64, Box<dyn std::error::Error>> {
    // Note: this example does work on Windows
    let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    path.push("WindowsAudioOutput");
    path.set_extension("dll");

    let lib = libloading::Library::new(path.as_path())?;
    unsafe {
        let func: libloading::Symbol<unsafe extern "C" fn() -> u64> =
            lib.get(b"GetMasterVolume")?;
        Ok(func())
    }
}

pub fn set_master_volume(volume: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Note: this example does work on Windows
    let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    path.push("WindowsAudioOutput");
    path.set_extension("dll");

    let lib = libloading::Library::new(path.as_path())?;
    unsafe {
        let func: libloading::Symbol<unsafe extern "C" fn(u64) -> ()> =
            lib.get(b"SetMasterVolume")?;
        Ok(func(volume))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_exporting() -> Result<(), Box<dyn std::error::Error>> {
        export_dll();

        Ok(())
    }

    #[test]
    fn test_getting_devices() -> Result<(), Box<dyn std::error::Error>> {
        export_dll();
        let devices = get_devices()?;
        assert_ne!(devices.len(), 0);

        devices.iter().find(|&d| d.is_default).unwrap();

        for device in &devices {
            println!("{} {} {}", device.id, device.is_default, device.name);
        }

        Ok(())
    }

    #[test]
    fn test_setting_default_device() -> Result<(), Box<dyn std::error::Error>> {
        export_dll();
        let devices = get_devices()?;
        let default_device = devices.iter().find(|&d| d.is_default).unwrap();
        set_default_audio_device(&default_device.id)?;

        Ok(())
    }

    #[test]
    fn test_setting_and_getting_master_volume() -> Result<(), Box<dyn std::error::Error>> {
        export_dll();
        let volume = 41;
        set_master_volume(volume)?;
        let master_volume = get_master_volume()?;
        assert_eq!(volume, master_volume);

        Ok(())
    }
}
