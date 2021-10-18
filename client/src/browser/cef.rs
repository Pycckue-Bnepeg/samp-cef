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
        log::trace!("BrowserProcessHandler::on_context_initialized");
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
        self: &Arc<Self>, _process_type: CefString, command_line: CommandLine,
    ) {
        command_line.append_switch("disable-gpu-compositing");
        command_line.append_switch("disable-gpu");
        command_line.append_switch("enable-begin-frame-scheduling");
        command_line.append_switch_with_value("autoplay-policy", "no-user-gesture-required");

        // TODO: permissions
        command_line.append_switch("enable-media-stream");
    }
}

pub fn initialize(event_tx: Sender<Event>) {
    let instance = unsafe { GetModuleHandleA(std::ptr::null()) };
    let main_args = cef_sys::cef_main_args_t {
        instance: instance as *mut _,
    };

    log::trace!("browser::cef::initialize");

    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_settings_t>() };

    let cef_dir = crate::utils::cef_dir();
    let cache_path = cef_dir.join("cache");

    log::trace!("cef_dir: {:?}", cef_dir);

    let path = cef::types::string::into_cef_string(&cef_dir.join("renderer.exe").to_string_lossy());
    let cache_path = cef::types::string::into_cef_string(&cache_path.to_string_lossy());
    let locales_dir_path =
        cef::types::string::into_cef_string(&cef_dir.join("locales").to_string_lossy());
    let resources_dir_path = cef::types::string::into_cef_string(&cef_dir.to_string_lossy());
    let log_file = cef::types::string::into_cef_string(&cef_dir.join("cef.log").to_string_lossy());
    let user_data =
        cef::types::string::into_cef_string(&cef_dir.join("user_data").to_string_lossy());

    log::trace!("{:?}", cef_dir.join("cef.log"));

    settings.size = std::mem::size_of::<cef_sys::cef_settings_t>();
    settings.no_sandbox = 1;
    settings.browser_subprocess_path = path;
    settings.windowless_rendering_enabled = 1;
    settings.multi_threaded_message_loop = 1;
    settings.log_severity = cef_sys::cef_log_severity_t::LOGSEVERITY_ERROR;
    settings.cache_path = cache_path;
    settings.locales_dir_path = locales_dir_path;
    settings.resources_dir_path = resources_dir_path;
    settings.ignore_certificate_errors = 1;
    settings.log_file = log_file;
    settings.user_data_path = user_data;

    let app = Arc::new(DefaultApp { event_tx });

    log::trace!("PRE cef::initialize");

    cef::initialize(Some(&main_args), &settings, Some(app));

    log::trace!("POST cef::initialize");
}

pub fn create_browser<T: Client>(client: Arc<T>, url: &str) {
    let mut window_info = unsafe { std::mem::zeroed::<cef_sys::cef_window_info_t>() };

    window_info.parent_window = client_api::gta::hwnd() as *mut _;
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

    log::trace!("PRE BrowserHost::create_browser");

    let result =
        cef::browser::BrowserHost::create_browser(&window_info, Some(client), &url, &settings);

    log::trace!("POST BrowserHost::create_browser result {}", result);
}
