use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};

use crossbeam_channel::Sender;

use cef::browser::{Browser, ContextMenuParams, Frame, MenuModel};
use cef::client::Client;
use cef::handlers::context_menu::ContextMenuHandler;
use cef::handlers::lifespan::LifespanHandler;
use cef::handlers::render::{DirtyRects, PaintElement, RenderHandler};
use cef::process_message::ProcessMessage;
use cef::types::list::ValueType;
use cef::ProcessId;

use cef_sys::cef_rect_t;

use client_api::utils::handle_result;

use crate::app::Event;
use crate::browser::view::View;
use crate::external::{CallbackList, EXTERNAL_BREAK};
use std::ffi::CString;

struct DrawData {
    buffer: *const u8,
    width: usize,
    height: usize,
    rects: DirtyRects,
    popup_buffer: Vec<u8>,
    popup_rect: cef_rect_t,
    changed: bool,
}

impl DrawData {
    fn new() -> DrawData {
        DrawData {
            buffer: std::ptr::null(),
            width: 0,
            height: 0,
            rects: DirtyRects {
                count: 0,
                rects: std::ptr::null(),
            },

            popup_buffer: Vec::new(),
            popup_rect: cef_rect_t {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },

            changed: false,
        }
    }
}

pub struct WebClient {
    id: u32, // static
    hidden: AtomicBool,
    view: Mutex<View>,
    draw_data: Mutex<DrawData>,
    browser: Mutex<Option<Browser>>,
    rendered: (Mutex<bool>, Condvar),
    event_tx: Sender<Event>,
    callbacks: CallbackList,
}

impl LifespanHandler for WebClient {
    fn on_after_created(self: &Arc<Self>, browser: Browser) {
        use winapi::um::winuser::*; // debug

        {
            let mut br = self.browser.lock().unwrap();

            // debug
            let window_name = cef::types::string::CefString::new("dev tools");

            let mut window_info = unsafe { std::mem::zeroed::<cef_sys::cef_window_info_t>() };

            window_info.style =
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;
            window_info.parent_window = std::ptr::null_mut();
            window_info.x = CW_USEDEFAULT;
            window_info.y = CW_USEDEFAULT;
            window_info.width = CW_USEDEFAULT;
            window_info.height = CW_USEDEFAULT;
            window_info.window_name = window_name.to_cef_string();
            window_info.windowless_rendering_enabled = 0;

            let mut settings = unsafe { std::mem::zeroed::<cef_sys::cef_browser_settings_t>() };

            settings.size = std::mem::size_of::<cef_sys::cef_browser_settings_t>();

            // todo: enable in debug mode
            //            browser.host().open_dev_tools(&window_info, &settings);

            *br = Some(browser);
        }

        if self.hidden.load(Ordering::SeqCst) {
            self.hide(true);
        }

        let event = Event::BrowserCreated(self.id);
        handle_result(self.event_tx.send(event));
    }

    fn on_before_close(self: &Arc<Self>, _: Browser) {
        let mut browser = self.browser.lock().unwrap();
        browser.take();
    }
}

impl Client for WebClient {
    type LifespanHandler = Self;
    type RenderHandler = Self;
    type ContextMenuHandler = Self;

    fn lifespan_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn render_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn context_menu_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn on_process_message(
        self: &Arc<Self>, _browser: Browser, _frame: Frame, _source: ProcessId, msg: ProcessMessage,
    ) -> bool {
        let name = msg.name().to_string();

        match name.as_str() {
            "set_focus" => {
                let args = msg.argument_list();
                let value_type = args.get_type(0);

                let focus = match value_type {
                    ValueType::Integer => args.integer(0) == 1,
                    ValueType::Bool => args.bool(0),
                    _ => false,
                };

                handle_result(self.event_tx.send(Event::FocusBrowser(self.id, focus)));

                return true;
            }

            "hide" => {
                let args = msg.argument_list();
                let value_type = args.get_type(0);

                let hide = match value_type {
                    ValueType::Integer => args.integer(0) == 1,
                    ValueType::Bool => args.bool(0),
                    _ => false,
                };

                handle_result(self.event_tx.send(Event::HideBrowser(self.id, hide)));

                return true;
            }

            "emit_event" => {
                let args = msg.argument_list();

                if args.get_type(0) != ValueType::String {
                    return true;
                }

                let event_name = args.string(0).to_string();
                let callbacks = self.callbacks.lock().unwrap();

                if let Some(cb) = callbacks.get(&event_name) {
                    let name = CString::new(event_name.clone()).unwrap(); // 100% valid string
                    let result = cb(name.as_ptr(), args.clone().into_cef());

                    // событие обработано плагином, нет смысла отправлять дальше серверу
                    if result == EXTERNAL_BREAK {
                        return true;
                    }
                }

                let mut arguments = String::new();

                for idx in 1..args.len() {
                    let arg = match args.get_type(idx) {
                        ValueType::String => args.string(idx).to_string(),
                        ValueType::Bool => (if args.bool(idx) { 1 } else { 0 }).to_string(),
                        ValueType::Double => args.double(idx).to_string(),
                        ValueType::Integer => args.integer(idx).to_string(),
                        _ => "CEF_NULL".to_string(),
                    };

                    arguments.push_str(&arg);
                    arguments.push(' ');
                }

                let event = Event::EmitEventOnServer(event_name, arguments);
                handle_result(self.event_tx.send(event));
            }

            _ => (),
        }

        false
    }
}

impl ContextMenuHandler for WebClient {
    fn on_before_context_menu(
        self: &Arc<Self>, _: Browser, _: Frame, _: ContextMenuParams, model: MenuModel,
    ) {
        model.clear(); // remove context menu
    }
}

impl RenderHandler for WebClient {
    fn view_rect(self: &Arc<Self>, _: Browser, rect: &mut cef_rect_t) {
        let texture = self.view.lock().unwrap();
        *rect = texture.rect();
    }

    fn on_popup_size(self: &Arc<Self>, _: Browser, rect: &cef_rect_t) {
        let mut draw_data = self.draw_data.lock().unwrap();

        draw_data.popup_rect = rect.clone();

        draw_data
            .popup_buffer
            .resize(rect.width as usize * rect.height as usize * 4, 0);
    }

    fn on_paint(
        self: &Arc<Self>, _: Browser, paint_type: PaintElement, dirty_rects: DirtyRects,
        buffer: &[u8], width: usize, height: usize,
    ) {
        {
            let mut draw_data = self.draw_data.lock().unwrap();

            match paint_type {
                PaintElement::Popup => {
                    if draw_data.popup_buffer.len() == buffer.len() {
                        draw_data.popup_buffer.copy_from_slice(buffer);
                    }

                    return;
                }

                PaintElement::View => {
                    draw_data.rects = dirty_rects;
                    draw_data.buffer = buffer.as_ptr();
                    draw_data.height = height;
                    draw_data.width = width;
                    draw_data.changed = true;
                }
            }
        }

        let (mutex, cv) = &self.rendered;
        let mut rendered = mutex.lock().unwrap();

        *rendered = false;

        while !*rendered {
            rendered = cv.wait(rendered).unwrap();
        }
    }
}

impl WebClient {
    pub fn new(id: u32, cbs: CallbackList, event_tx: Sender<Event>) -> Arc<WebClient> {
        let rect = crate::utils::client_rect();
        let view = View::new(client_api::gta::d3d9::device(), rect[0], rect[1]);

        let client = WebClient {
            hidden: AtomicBool::new(false),
            view: Mutex::new(view),
            draw_data: Mutex::new(DrawData::new()),
            browser: Mutex::new(None),
            rendered: (Mutex::new(false), Condvar::new()),
            callbacks: cbs,
            event_tx,
            id,
        };

        Arc::new(client)
    }

    pub fn draw(&self) {
        let mut texture = self.view.lock().unwrap();
        texture.draw();
    }

    pub fn on_lost_device(&self) {
        self.internal_hide(true, false); // hide browser but do not save value

        let mut texture = self.view.lock().unwrap();
        texture.on_lost_device();
    }

    pub fn on_reset_device(&self) {
        {
            let mut texture = self.view.lock().unwrap();
            let rect = crate::utils::client_rect();
            texture.on_reset_device(client_api::gta::d3d9::device(), rect[0], rect[1]);
        }

        self.restore_hide_status();
    }

    pub fn resize(&self, width: usize, height: usize) {
        let resized_view = View::new(client_api::gta::d3d9::device(), width, height);
        let mut view = self.view.lock().unwrap();
        let browser = self.browser.lock().unwrap();
        *view = resized_view;

        if let Some(host) = browser.as_ref().map(|brw| brw.host()) {
            host.was_resized();
        }
    }

    // TODO: показывать всплывающие окна
    pub fn update_view(&self) {
        if self.hidden.load(Ordering::SeqCst) {
            self.unlock();
            return;
        }

        {
            let mut texture = self.view.lock().unwrap();
            let mut draw_data = self.draw_data.lock().unwrap();

            if draw_data.buffer.is_null() {
                self.unlock();
                return;
            }

            if draw_data.height == 0 || draw_data.width == 0 {
                self.unlock();
                return;
            }

            if !draw_data.changed {
                return;
            }

            let bytes = unsafe {
                std::slice::from_raw_parts(draw_data.buffer, draw_data.width * draw_data.height * 4)
            };

            texture.update_texture(&bytes, draw_data.rects.as_slice());

            draw_data.changed = false;
        }

        self.unlock();
    }

    fn unlock(&self) {
        let (mutex, cv) = &self.rendered;
        let mut rendered = mutex.lock().unwrap();
        *rendered = true;
        cv.notify_all();
    }

    pub fn browser(&self) -> Option<Browser> {
        let browser = self.browser.lock().unwrap();

        browser.as_ref().map(|browser| browser.clone())
    }

    pub fn hide(&self, hide: bool) {
        self.internal_hide(hide, true);
    }

    pub fn internal_hide(&self, hide: bool, store_value: bool) {
        if store_value {
            self.hidden.store(hide, Ordering::SeqCst);
        }

        self.browser()
            .map(|browser| browser.host())
            .map(|host| host.was_hidden(hide));

        if hide {
            let mut view = self.view.lock().unwrap();
            view.clear_texture();
        }
    }

    pub fn restore_hide_status(&self) {
        self.internal_hide(self.hidden.load(Ordering::SeqCst), false);
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}
