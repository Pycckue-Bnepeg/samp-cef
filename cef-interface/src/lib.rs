use cef_api::CefApi;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn DllMain(_instance: u32, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(|| {
            initialize();
        });
    }

    if reason == DLL_PROCESS_DETACH {}

    return true;
}

fn initialize() {
    while client_api::samp::gamestate() != client_api::samp::Gamestate::Connected {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let cef = CefApi::wait_loading().unwrap();
    let mut phone_open = false;

    cef.create_browser(0xFF00, "http://5.63.153.185/telephone.html");

    loop {
        if client_api::utils::is_key_pressed(0x72) {
            if !phone_open {
                if cef.try_focus_browser(0xFF00) {
                    cef.hide_browser(0xFF00, false);
                    phone_open = true;
                }
            } else {
                cef.focus_browser(0xFF00, false);
                cef.hide_browser(0xFF00, true);
                phone_open = false;
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
