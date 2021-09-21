use std::collections::{HashSet, VecDeque};
use std::ffi::CString;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use parking_lot::{Condvar, Mutex};

use cef::browser::{Browser, ContextMenuParams, Frame, MenuModel};
use cef::client::Client;
use cef::handlers::audio::AudioHandler;
use cef::handlers::context_menu::ContextMenuHandler;
use cef::handlers::lifespan::LifespanHandler;
use cef::handlers::load::LoadHandler;
use cef::handlers::render::{DirtyRects, PaintElement, RenderHandler};
use cef::process_message::ProcessMessage;
use cef::types::list::ValueType;
use cef::ProcessId;

use cef_sys::{cef_audio_parameters_t, cef_rect_t};

use client_api::gta::rw::rwcore::{RwRaster, RwTexture};
use client_api::utils::handle_result;

use crate::app::Event;
use crate::audio::Audio;
use crate::browser::view::View;
use crate::external::{CallbackList, EXTERNAL_BREAK};
use crate::utils::RenderMode;

struct DrawData {
    buffer: *const u8,
    width: usize,
    height: usize,
    rects: DirtyRects,
    popup_buffer: Vec<u8>,
    popup_rect: cef_rect_t,
    popup_show: bool,
    popup_was_before: bool,
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
                rects: Vec::new(),
            },

            popup_buffer: Vec::new(),
            popup_rect: cef_rect_t {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },

            popup_show: false,
            popup_was_before: false,
            changed: false,
        }
    }
}

pub struct WebClient {
    id: u32, // static
    is_extern: bool,
    hidden: AtomicBool,
    closing: AtomicBool,
    listen_keys: AtomicBool,
    pub view: Mutex<View>,
    draw_data: Mutex<DrawData>,
    browser: Mutex<Option<Browser>>,
    audio: Option<Arc<Audio>>, // static
    event_tx: Sender<Event>,
    callbacks: CallbackList,
    object_list: Mutex<HashSet<i32>>,
    rendered: (Mutex<bool>, Condvar),
}

impl LifespanHandler for WebClient {
    fn on_after_created(self: &Arc<Self>, browser: Browser) {
        {
            let mut br = self.browser.lock();
            *br = Some(browser);
        }

        let hidden = self.hidden.load(Ordering::SeqCst);

        log::trace!("LifespanHandler::on_after_created. hidden: {}", hidden);

        self.hide(hidden);
    }

    fn on_before_close(self: &Arc<Self>, _: Browser) {
        log::trace!("LifespanHandler::on_before_close");

        let mut browser = self.browser.lock();

        if let Some(browser) = browser.take() {
            browser.host().close_dev_tools();
        }
    }
}

impl Client for WebClient {
    type LifespanHandler = Self;
    type RenderHandler = Self;
    type ContextMenuHandler = Self;
    type LoadHandler = Self;
    type AudioHandler = Self;

    fn lifespan_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn render_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn context_menu_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn load_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn audio_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        if self.is_extern {
            Some(self.clone())
        } else {
            None
        }
    }

    fn on_process_message(
        self: &Arc<Self>, _browser: Browser, _frame: Frame, _source: ProcessId, msg: ProcessMessage,
    ) -> bool {
        let name = msg.name().to_string();

        log::trace!(
            "WebClient::on_process_message. browser_id: {}, message: {:?}",
            self.id,
            name
        );

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
                let callbacks = self.callbacks.lock();

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

                    if idx != args.len() - 1 {
                        arguments.push(' ');
                    }
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
        let texture = self.view.lock();
        *rect = texture.rect();
    }

    fn on_popup_show(self: &Arc<Self>, _: Browser, show: bool) {
        let mut draw_data = self.draw_data.lock();
        draw_data.popup_show = show;

        if !show {
            draw_data.popup_buffer.clear();
            draw_data.popup_was_before = true; // REMOVE
        }
    }

    fn on_popup_size(self: &Arc<Self>, _: Browser, rect: &cef_rect_t) {
        let mut draw_data = self.draw_data.lock();

        draw_data.popup_rect = rect.clone();

        draw_data
            .popup_buffer
            .resize(rect.width as usize * rect.height as usize * 4, 0);
    }

    fn on_paint(
        self: &Arc<Self>, _: Browser, paint_type: PaintElement, mut dirty_rects: DirtyRects,
        buffer: &[u8], width: usize, height: usize,
    ) {
        let view = self.view.lock();

        if self.closing.load(Ordering::SeqCst) || view.is_empty() {
            return;
        }

        {
            let mut draw_data = self.draw_data.lock();

            match paint_type {
                PaintElement::Popup => {
                    if draw_data.popup_buffer.len() == buffer.len() {
                        draw_data.popup_buffer.copy_from_slice(buffer);
                    }

                    return;
                }

                PaintElement::View => {
                    if draw_data.popup_was_before && !draw_data.popup_show {
                        draw_data.popup_was_before = false;
                        dirty_rects.count += 1;
                        dirty_rects.rects.push(draw_data.popup_rect.clone());
                    }

                    draw_data.rects = dirty_rects;
                    draw_data.buffer = buffer.as_ptr();
                    draw_data.height = height;
                    draw_data.width = width;
                    draw_data.changed = true;
                }
            }
            // }
            //
            // {

            let timing = std::time::Instant::now();

            let (mutex, cv) = &self.rendered;
            let mut rendered = mutex.lock();

            *rendered = false;

            drop(view);
            drop(draw_data);

            // if cv
            //     .wait_for(&mut rendered, Duration::from_millis(10_000))
            //     .timed_out()
            // {
            //     log::trace!("timed_out ... fuckit");
            // }

            cv.wait(&mut rendered);

            let dur = timing.elapsed();

            if dur >= Duration::from_millis(100) {
                log::trace!("was waiting for ... {:?}", dur);
            }
        }

        self.draw_data.lock().changed = false;
    }
}

impl LoadHandler for WebClient {
    fn on_load_end(self: &Arc<Self>, browser: Browser, frame: Frame, status_code: i32) {
        log::trace!(
            "LoadHandler::on_load_end. id: {} status: {}",
            self.id,
            status_code
        );

        if frame.is_main() {
            let event = Event::BrowserCreated(self.id, status_code);
            handle_result(self.event_tx.send(event));
        }
    }
}

impl AudioHandler for WebClient {
    fn get_audio_parameters(
        self: &Arc<Self>, browser: Browser, params: &mut cef_audio_parameters_t,
    ) -> bool {
        log::trace!(
            "get_audio_parameters: {} {} {}",
            params.sample_rate,
            params.channel_layout,
            params.frames_per_buffer
        );

        true
    }

    fn on_audio_stream_packet(
        self: &Arc<Self>, browser: Browser, stream_id: i32, data: *mut *const f32, frames: i32,
        pts: i64,
    ) {
        if let Some(audio) = self.audio.as_ref() {
            audio.append_pcm(self.id, stream_id, data, frames, pts as u64);
        }
    }

    fn on_audio_stream_started(
        self: &Arc<Self>, browser: Browser, stream_id: i32, channels: i32, channel_layout: i32,
        sample_rate: i32, frames_per_buffer: i32,
    ) {
        if let Some(audio) = self.audio.as_ref() {
            audio.create_stream(self.id, stream_id, channels, sample_rate, frames_per_buffer);
            let objects = self.object_list.lock();

            for &object_id in objects.iter() {
                audio.add_source(self.id, object_id);
            }
        }
    }

    fn on_audio_stream_stopped(self: &Arc<Self>, browser: Browser, stream_id: i32) {
        if let Some(audio) = self.audio.as_ref() {
            audio.remove_stream(self.id, stream_id);
        }
    }

    fn on_audio_stream_error(self: &Arc<Self>, browser: Browser, error: String) {
        log::trace!("on_audio_stream_error: {:?}", error);
    }
}

impl WebClient {
    pub fn new(id: u32, cbs: CallbackList, event_tx: Sender<Event>) -> Arc<WebClient> {
        let rect = crate::utils::client_rect();

        log::trace!("crate::utils::client_rect: {:?}", rect);

        let mut view = View::new(crate::utils::current_render_mode());
        view.make_directx(client_api::gta::d3d9::device(), rect[0], rect[1]);

        let client = WebClient {
            hidden: AtomicBool::new(false),
            closing: AtomicBool::new(false),
            listen_keys: AtomicBool::new(false),
            view: Mutex::new(view),
            draw_data: Mutex::new(DrawData::new()),
            browser: Mutex::new(None),
            callbacks: cbs,
            object_list: Mutex::new(HashSet::new()),
            is_extern: false,
            audio: None,
            event_tx,
            id,
            rendered: (Mutex::new(false), Condvar::new()),
        };

        Arc::new(client)
    }

    pub fn new_extern(
        id: u32, cbs: CallbackList, event_tx: Sender<Event>, audio: Arc<Audio>,
    ) -> Arc<WebClient> {
        let view = View::new(crate::utils::current_render_mode());

        let client = WebClient {
            hidden: AtomicBool::new(false),
            closing: AtomicBool::new(false),
            listen_keys: AtomicBool::new(false),
            view: Mutex::new(view),
            draw_data: Mutex::new(DrawData::new()),
            browser: Mutex::new(None),
            callbacks: cbs,
            object_list: Mutex::new(HashSet::new()),
            is_extern: true,
            audio: Some(audio),
            event_tx,
            id,
            rendered: (Mutex::new(false), Condvar::new()),
        };

        Arc::new(client)
    }

    #[inline]
    pub fn draw(&self) {
        let mut texture = self.view.lock();
        texture.draw();
    }

    #[inline]
    pub fn on_lost_device(&self) {
        self.internal_hide(true, false); // hide browser but do not save value

        {
            let mut texture = self.view.lock();
            texture.on_lost_device();
            texture.make_empty();
        }

        self.unlock();
    }

    #[inline]
    pub fn on_reset_device(&self) {
        {
            let mut view = self.view.lock();

            if self.is_extern() {
            } else {
                let rect = crate::utils::client_rect();

                view.set_render_mode(crate::utils::current_render_mode());
                view.make_directx(client_api::gta::d3d9::device(), rect[0], rect[1]);

                self.notify_was_resized();
            }
        }

        self.restore_hide_status();
    }

    #[inline]
    pub fn resize(&self, width: usize, height: usize) {
        let device = if self.is_extern() {
            None
        } else {
            Some(client_api::gta::d3d9::device())
        };

        let mut view = self.view.lock();
        view.resize(device, width, height);
        self.notify_was_resized();

        self.unlock();
    }

    fn notify_was_resized(&self) {
        let browser = self.browser.lock();

        if let Some(host) = browser.as_ref().map(|brw| brw.host()) {
            host.was_resized();
        }
    }

    #[inline]
    pub fn update_view(&self) {
        if self.hidden.load(Ordering::SeqCst) || self.closing.load(Ordering::SeqCst) {
            self.unlock();
            return;
        }

        {
            let mut texture = self.view.lock();
            let mut draw_data = self.draw_data.lock();
            let size = texture.rect();

            if draw_data.changed
                && (size.height as usize != draw_data.height
                    || size.width as usize != draw_data.width)
            {
                draw_data.changed = false;
            }

            if draw_data.changed {
                if draw_data.buffer.is_null() {
                    self.unlock();
                    return;
                }

                if draw_data.height == 0 || draw_data.width == 0 {
                    self.unlock();
                    return;
                }

                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        draw_data.buffer,
                        draw_data.width * draw_data.height * 4,
                    )
                };

                if draw_data.rects.count > 0 {
                    let rect = &draw_data.rects.rects[0];
                    if rect.width > size.width || rect.height > size.height {
                        self.unlock();
                        return;
                    }
                }

                texture.update_texture(&bytes, draw_data.rects.as_slice());
            }

            if draw_data.popup_show {
                if draw_data.popup_rect.x + draw_data.popup_rect.width >= size.width
                    || draw_data.popup_rect.y + draw_data.popup_rect.height >= size.height
                {
                    self.unlock();
                    return;
                }

                texture.update_popup(&draw_data.popup_buffer, &draw_data.popup_rect);
            }
        }

        self.unlock();
    }

    #[inline(always)]
    fn unlock(&self) {
        let (mutex, cv) = &self.rendered;

        {
            let mut rendered = mutex.lock();
            *rendered = true;
        }

        cv.notify_all();
    }

    #[inline(always)]
    pub fn browser(&self) -> Option<Browser> {
        let browser = self.browser.lock();

        browser.as_ref().map(|browser| browser.clone())
    }

    pub fn hide(&self, hide: bool) {
        self.internal_hide(hide, true);
    }

    pub fn internal_hide(&self, hide: bool, store_value: bool) {
        if store_value {
            self.hidden.store(hide, Ordering::SeqCst);
        }

        if let Some(host) = self.browser().map(|browser| browser.host()) {
            if hide {
                let mut view = self.view.lock();
                view.clear();

                host.was_hidden(true);
            } else {
                host.was_hidden(false);
                host.invalidate(PaintElement::View);
            }
        }

        if !self.is_extern() {
            self.set_audio_muted(hide);
        }
    }

    pub fn restore_hide_status(&self) {
        self.internal_hide(self.hidden.load(Ordering::SeqCst), false);
    }

    pub fn set_audio_muted(&self, muted: bool) {
        if let Some(host) = self.browser().map(|br| br.host()) {
            host.set_audio_muted(muted);
        }
    }

    pub fn add_object(&self, object_id: i32) {
        let mut objects = self.object_list.lock();
        objects.insert(object_id);
    }

    pub fn remove_object(&self, object_id: i32) {
        let mut objects = self.object_list.lock();
        objects.remove(&object_id);
    }

    pub fn remove_view(&self) {
        self.view.lock().make_empty();
    }

    pub fn toggle_dev_tools(&self, enabled: bool) {
        use winapi::um::winuser::*;

        let id = self.id();

        self.browser().map(|br| br.host()).map(|host| {
            if enabled {
                let caption = format!("Dev Tools for {} browser", id);
                let window_name = cef::types::string::CefString::new(&caption);

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

                host.open_dev_tools(&window_info, &settings);
            } else {
                host.close_dev_tools();
            }
        });
    }

    pub fn load_url(&self, url: &str) {
        if let Some(browser) = self.browser() {
            browser.main_frame().load_url(url);
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_extern(&self) -> bool {
        self.is_extern
    }

    pub fn close(&self, force_close: bool) {
        self.closing.store(true, Ordering::SeqCst);
        self.unlock();

        self.browser()
            .map(|br| br.host())
            .map(|host| host.close_browser(force_close));
    }

    pub fn always_listen_keys(&self) -> bool {
        self.listen_keys.load(Ordering::SeqCst)
    }

    pub fn set_always_listen_keys(&self, listen: bool) {
        self.listen_keys.store(listen, Ordering::SeqCst);
    }
}
