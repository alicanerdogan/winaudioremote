use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

pub fn add_app_to_startup() {
    let path = std::env::current_exe().unwrap();
    let path_str = path.to_str().unwrap();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("SOFTWARE")
        .join("Microsoft")
        .join("Windows")
        .join("CurrentVersion")
        .join("Run");
    let (key, _disp) = hkcu.create_subkey(&path).unwrap();
    key.set_value("winaudioremote", &path_str).unwrap();
}
