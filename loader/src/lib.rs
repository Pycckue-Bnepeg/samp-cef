use libloading::Library;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

static mut LIBRARY: Option<Library> = None;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn DllMain(_instance: u32, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(|| {
            while !std::env::current_dir()
                .map(|dir| {
                    std::env::current_exe()
                        .map(|exe| exe.parent().unwrap() == dir)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            unsafe {
                let cur: Vec<u16> = std::env::current_exe()
                    .unwrap_or_else(|_| PathBuf::from("./"))
                    .parent()
                    .unwrap_or_else(|| Path::new("./"))
                    .join("./cef")
                    .as_os_str()
                    .encode_wide()
                    .chain(Some(0))
                    .collect();

                winapi::um::winbase::SetDllDirectoryW(cur.as_ptr());
            }

            if let Ok(lib) = Library::new("./cef/client.dll") {
                unsafe {
                    LIBRARY = Some(lib);
                }
            }
        });
    }

    if reason == DLL_PROCESS_DETACH {
        unsafe {
            LIBRARY.take();
        }
    }

    return true;
}
