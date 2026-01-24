use libloading::Library;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::Mutex;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

static LIBRARY: Mutex<Option<Library>> = Mutex::new(None);

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "stdcall" fn DllMain(_instance: u32, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(|| {
            // unsafe {
            //     winapi::um::consoleapi::AllocConsole();
            // }

            let path = {
                let try_launcher = std::env::args()
                    .skip_while(|arg| !arg.contains("--lp"))
                    .nth(1)
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

            if let Ok(lib) = unsafe { Library::new(path.join("cef\\client.dll")) } {
                unsafe {
                    if let Ok(initialize) =
                        lib.get::<unsafe extern "C" fn()>(b"cef_client_initialize")
                    {
                        initialize();
                    }
                }

                let mut library = LIBRARY.lock().unwrap();
                *library = Some(lib);
            }
        });
    }

    if reason == DLL_PROCESS_DETACH {
        let mut library = LIBRARY.lock().unwrap();
        library.take();
    }

    true
}
