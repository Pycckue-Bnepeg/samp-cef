//#![feature(arbitrary_self_types)]
//
//use winapi::um::libloaderapi::GetModuleHandleA;
//use winapi::um::winuser::{
//    CW_USEDEFAULT, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
//};
//
//use cef::app::App;
//use cef::browser::Browser;
//use cef::client::Client;
//use cef::handlers::lifespan::LifespanHandler;
//use cef::handlers::render::DefaultRenderHandler;
//use cef::types::string::{cef_string_t, CefString};
//
//use std::ptr::{null, null_mut};
//use std::sync::Arc;
//
//struct SimpleHandler;
//
//impl LifespanHandler for SimpleHandler {
//    fn on_after_created(self: &Arc<Self>, _: Browser) {
//        println!("browser created!");
//    }
//
//    fn on_before_close(self: &Arc<Self>, _: Browser) {
//        println!("do close!");
//        cef::quit_message_loop();
//    }
//}
//
//impl Client for SimpleHandler {
//    type LifespanHandler = Self;
//    type RenderHandler = DefaultRenderHandler;
//
//    fn lifespan_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
//        println!("pass lifespan handler");
//        Some(self.clone())
//    }
//}
//
//struct SimpleApp;
//
//impl App for SimpleApp {}

fn main() {
    //    let instance = unsafe { GetModuleHandleA(null()) };
    //
    //    let main_args = cef_sys::cef_main_args_t { instance };
    //
    //    let app = Arc::new(SimpleApp);
    //
    //    let code = cef::execute_process(&main_args, Some(app.clone()));
    //
    //    if code >= 0 {
    //        std::process::exit(code);
    //    }
    //
    //    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_settings_t>() };
    //
    //    let path = CefString::new("");
    //
    //    settings.size = std::mem::size_of::<cef_sys::cef_settings_t>();
    //    settings.no_sandbox = 1;
    //    settings.browser_subprocess_path = path.to_cef_string();
    //    settings.windowless_rendering_enabled = 1;
    //
    //    cef::initialize(&main_args, &settings, Some(app));
    //
    //    let window_name = CefString::new("hello everyone!");
    //
    //    let mut window_info = unsafe { std::mem::zeroed::<cef_sys::cef_window_info_t>() };
    //
    //    window_info.style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;
    //    window_info.parent_window = null_mut();
    //    window_info.x = CW_USEDEFAULT;
    //    window_info.y = CW_USEDEFAULT;
    //    window_info.width = CW_USEDEFAULT;
    //    window_info.height = CW_USEDEFAULT;
    //    window_info.window_name = window_name.to_cef_string();
    //
    //    let url = CefString::new("https://google.com");
    //
    //    let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_browser_settings_t>() };
    //
    //    settings.size = std::mem::size_of::<cef_sys::cef_browser_settings_t>();
    //    settings.windowless_frame_rate = 60;
    //
    //    let client = Arc::new(SimpleHandler);
    //
    //    cef::browser::BrowserHost::create_browser(&window_info, Some(client), &url, &settings);
    //    cef::run_message_loop();
    //    cef::shutdown();
}
