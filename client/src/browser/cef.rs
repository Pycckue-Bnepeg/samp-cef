use winapi::um::libloaderapi::GetModuleHandleA;

use cef::app::App;
use cef::client::Client;
use cef::handlers::render_process::RenderProcessHandler;
use cef::types::string::CefString;

use std::sync::Arc;

// App placeholder. Literally does nothing.
struct DefaultApp;

impl RenderProcessHandler for DefaultApp {}

impl App for DefaultApp {
    type RenderProcessHandler = Self;
}

pub fn initialize() {
    let instance = unsafe { GetModuleHandleA(std::ptr::null()) };

    let main_args = cef_sys::cef_main_args_t { instance };

    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_settings_t>() };

    let path = CefString::new("./cef/renderer.exe");

    settings.size = std::mem::size_of::<cef_sys::cef_settings_t>();
    settings.no_sandbox = 1;
    settings.browser_subprocess_path = path.to_cef_string();
    settings.windowless_rendering_enabled = 1;
    settings.multi_threaded_message_loop = 1;

    cef::initialize::<DefaultApp>(&main_args, &settings, None);
}

pub fn create_browser<T: Client>(client: Arc<T>, url: &str) {
    let mut window_info = unsafe { std::mem::zeroed::<cef_sys::cef_window_info_t>() };

    window_info.parent_window = client_api::gta::hwnd();
    window_info.windowless_rendering_enabled = 1;

    let url = CefString::new(url);

    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_browser_settings_t>() };

    settings.size = std::mem::size_of::<cef_sys::cef_browser_settings_t>();
    settings.windowless_frame_rate = 60;
    settings.javascript_access_clipboard = cef_sys::cef_state_t::STATE_DISABLED;
    settings.javascript_dom_paste = cef_sys::cef_state_t::STATE_DISABLED;
    settings.webgl = cef_sys::cef_state_t::STATE_ENABLED;
    settings.plugins = cef_sys::cef_state_t::STATE_DISABLED;
    settings.javascript = cef_sys::cef_state_t::STATE_ENABLED;

    cef::browser::BrowserHost::create_browser(&window_info, Some(client), &url, &settings);
}
