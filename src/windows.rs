// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use winapi;
use winreg;
use self::winapi::shared::windef::HWND;
use self::winapi::shared::ntdef::LPCWSTR;
use self::winreg::enums::HKEY_CURRENT_USER;
use self::winreg::RegKey;
use crate::app::App;

use crate::errors::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

fn to_wide_chars(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}

#[link(name = "shell32")]
extern "system" {
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: i32,
    ) -> i32;
}

// as described at https://msdn.microsoft.com/en-us/library/aa767914(v=vs.85).aspx
/// Register the given App for the given schemes.
///
/// `app` should contain all fields necessary for registering URIs on all systems. `schemes` should
/// provide a list of schemes (the initial part of a URI, like `https`).
pub fn install(app: &App, schemes: &[String]) -> Result<(), Error> {
    // but we can't write on root, we'll have to do it for the current user only
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for protocol in schemes {
        let base_path = Path::new("Software").join("Classes").join(protocol);
        let key = hkcu.create_subkey(&base_path)?;
        // set our app name as the for reference
        // key.("", &app.name)?;
        // key.set_value("URL Protocol", &"")?;
        key.0.set_value("URL Protocol", &app.name);
        let command_key =
            hkcu.create_subkey(&base_path.join("shell").join("open").join("command"))?;
        command_key.0.set_value("", &format!("{} \"%1\"", app.exec))?
    }
    Ok(())
}

/// Open a given URI.
#[allow(unsafe_code)]
pub fn open<S: Into<String>>(uri: S) -> Result<(), Error> {
    let err = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            to_wide_chars("open").as_ptr(),
            to_wide_chars(&(uri.into().replace("\n", "%0A"))).as_ptr(),
            ptr::null(),
            ptr::null(),
            1,
        )
    };
    if err < 32 {
        Err(Error::ShellOpenError(err))
    } else {
        Ok(())
    }
}
