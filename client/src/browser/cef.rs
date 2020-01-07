use winapi::um::libloaderapi::GetModuleHandleA;

use cef::app::App;
use cef::client::Client;
use cef::command_line::CommandLine;
use cef::handlers::browser_process::BrowserProcessHandler;
use cef::handlers::render_process::RenderProcessHandler;
use cef::types::string::CefString;

use std::sync::Arc;

use crossbeam_channel::Sender;

use crate::app::Event;

struct DefaultApp {
    event_tx: Sender<Event>,
}

impl RenderProcessHandler for DefaultApp {}
impl BrowserProcessHandler for DefaultApp {
    fn on_context_initialized(self: &Arc<Self>) {
        self.event_tx.send(Event::CefInitialize);
    }
}

impl App for DefaultApp {
    type RenderProcessHandler = Self;
    type BrowserProcessHandler = Self;

    fn browser_process_handler(self: &Arc<Self>) -> Option<Arc<Self::BrowserProcessHandler>> {
        Some(self.clone())
    }

    fn on_before_command_line_processing(
        self: &Arc<Self>, process_type: CefString, command_line: CommandLine,
    ) {
        command_line.append_switch("disable-surfaces");
        command_line.append_switch("disable-gpu-compositing");
        command_line.append_switch("disable-gpu");
        command_line.append_switch("disable-d3d11");
        command_line.append_switch("disable-gpu-vsync");
        command_line.append_switch("enable-begin-frame-scheduling");
        command_line.append_switch_with_value("autoplay-policy", "no-user-gesture-required");
    }
}

pub fn initialize(event_tx: Sender<Event>) {
    let instance = unsafe { GetModuleHandleA(std::ptr::null()) };

    let main_args = cef_sys::cef_main_args_t { instance };

    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_settings_t>() };

    let path = CefString::new("./cef/renderer.exe");

    settings.size = std::mem::size_of::<cef_sys::cef_settings_t>();
    settings.no_sandbox = 1;
    settings.browser_subprocess_path = path.to_cef_string();
    settings.windowless_rendering_enabled = 1;
    settings.multi_threaded_message_loop = 1;

    let app = Arc::new(DefaultApp { event_tx });

    cef::initialize(&main_args, &settings, Some(app));
}

pub fn create_browser<T: Client>(client: Arc<T>, url: &str) {
    let mut window_info = unsafe { std::mem::zeroed::<cef_sys::cef_window_info_t>() };

    window_info.parent_window = client_api::gta::hwnd();
    window_info.windowless_rendering_enabled = 1;

    let url = CefString::new(url);

    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_browser_settings_t>() };

    settings.size = std::mem::size_of::<cef_sys::cef_browser_settings_t>();
    settings.windowless_frame_rate = 60;
    settings.javascript_access_clipboard = cef_sys::cef_state_t::STATE_ENABLED;
    settings.javascript_dom_paste = cef_sys::cef_state_t::STATE_ENABLED;
    settings.remote_fonts = cef_sys::cef_state_t::STATE_ENABLED;
    settings.webgl = cef_sys::cef_state_t::STATE_ENABLED;
    settings.javascript = cef_sys::cef_state_t::STATE_ENABLED;

    cef::browser::BrowserHost::create_browser(&window_info, Some(client), &url, &settings);
}
