use libloading::Library;

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

            let current = std::env::current_dir().unwrap();
            let temp_dir = current.join("./cef");

            std::env::set_current_dir(temp_dir).unwrap();

            if let Ok(lib) = Library::new("client.dll") {
                unsafe {
                    LIBRARY = Some(lib);
                }
            }

            std::env::set_current_dir(current);
        });
    }

    if reason == DLL_PROCESS_DETACH {
        unsafe {
            LIBRARY.take();
        }
    }

    return true;
}
