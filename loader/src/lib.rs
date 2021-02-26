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
            // unsafe {
            //     winapi::um::consoleapi::AllocConsole();
            // }

            let path = {
                let try_launcher = std::env::args()
                    .skip_while(|arg| !arg.contains("--lp"))
                    .skip(1)
                    .next()
                    .map(|p| PathBuf::from(&p));

                let try_exe_path = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()));

                try_launcher
                    .or(try_exe_path)
                    .unwrap_or_else(|| PathBuf::from("./"))
            };

            let search_dir: Vec<u16> = PathBuf::from(&path)
                .join("cef")
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();

            unsafe {
                winapi::um::winbase::SetDllDirectoryW(search_dir.as_ptr());
            }

            if let Ok(lib) = Library::new(path.join("cef\\client.dll")) {
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
