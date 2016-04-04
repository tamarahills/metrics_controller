extern crate sys_info;

use self::sys_info::*;

pub trait OSInfo {
    fn get_os(&mut self) -> Result <String, Error>;
    fn get_os_version(&mut self) -> Result <String, Error>;
}

pub fn get_os<T: OSInfo>(helper: &mut T) -> String {
    match helper.get_os() {
        Result::Ok(os_type) => return os_type,
        Result::Err(_) => return "Unknown OS".to_string()
    }
}

pub fn get_os_version<T: OSInfo>(helper: &mut T) -> String {
    match helper.get_os_version() {
        Result::Ok(os_version) => return os_version,
        Result::Err(_) => return "Unknown OS Version".to_string()
    }
}

pub struct SysInfoHelper;

impl OSInfo for SysInfoHelper {
    fn  get_os(&mut self) -> Result <String, Error> {
        os_type()
    }
    fn get_os_version(&mut self) -> Result <String, Error> {
        os_release()
    }
}

#[cfg(test)]
pub struct MockSysInfoHelper {
    success: bool
}

#[cfg(test)]
impl OSInfo for MockSysInfoHelper {
    fn  get_os(&mut self) -> Result <String, Error> {
        if self.success {
            os_type()
        }
        else {
            Err(Error::UnsupportedSystem)
        }
    }
    fn  get_os_version(&mut self) -> Result <String, Error> {
        if self.success {
            os_release()
        }
        else {
            Err(Error::UnsupportedSystem)
        }
    }
}

#[cfg(test)]
impl MockSysInfoHelper {
    fn getResult(&mut self) -> bool { self.success }
    fn setResult(&mut self, success: bool) {
        self.success = success
    }
}

#[test]
#[cfg(target_os = "macos")]
fn test_get_os_success() {
    let mut helper = MockSysInfoHelper { success: true };
    let os_type = get_os(&mut helper);
    assert_eq!(os_type, "Darwin".to_string());
}

#[test]
#[cfg(target_os = "windows")]
fn test_get_os_success() {
    let mut helper = MockSysInfoHelper { success: true };
    let os_type = get_os(&mut helper);
    assert_eq!(os_type, "Windows".to_string());
}

#[test]
#[cfg(target_os = "linux")]
fn test_get_os_success() {
    let mut helper = MockSysInfoHelper { success: true };
    let os_type = get_os(&mut helper);
    assert_eq!(os_type, "Linux".to_string());
}

#[test]
fn test_get_os_not_supported() {
    let mut helper = MockSysInfoHelper { success: false };
    let os_type = get_os(&mut helper);
    assert_eq!(os_type, "Unknown OS".to_string());
}

#[test]
fn test_get_os_version() {
    let mut helper = MockSysInfoHelper { success: true };
    let os_version = get_os_version(&mut helper);
    println!("\n{}", os_version);
}

#[test]
fn test_get_os_version_unknown() {
    let mut helper = MockSysInfoHelper { success: false };
    let os_version = get_os_version(&mut helper);
    println!("\n{}", os_version);
}
