use std::collections::HashSet;
use std::ffi::CString;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use crossbeam_channel::Sender;
use parking_lot::{Condvar, Mutex};

use cef::ProcessId;
use cef::browser::{Browser, ContextMenuParams, Frame, MenuModel};
use cef::client::Client;
use cef::handlers::audio::AudioHandler;
use cef::handlers::context_menu::ContextMenuHandler;
use cef::handlers::lifespan::LifespanHandler;
use cef::handlers::load::LoadHandler;
use cef::handlers::render::{DirtyRects, PaintElement, RenderHandler};
use cef::process_message::ProcessMessage;
use cef::types::list::ValueType;

use cef_sys::{cef_audio_parameters_t, cef_rect_t};

use client_api::utils::handle_result;

use crate::app::Event;
use crate::audio::Audio;
use crate::browser::view::View;
use crate::external::{CallbackList, EXTERNAL_BREAK};

struct DrawData {
    view_buffer: Vec<u8>,
    width: usize,
    height: usize,
    rects: DirtyRects,
    popup_buffer: Vec<u8>,
    popup_rect: cef_rect_t,
    popup_show: bool,
    popup_was_before: bool,
    changed: bool,
    generation: u64,
}

impl DrawData {
    fn new() -> DrawData {
        DrawData {
            view_buffer: Vec::new(),
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
            generation: 0,
        }
    }
}

pub struct WebClient {
    id: u32, // static
    is_extern: bool,
    hidden: AtomicBool,
    prev_hidden_flag: AtomicBool,
    closing: AtomicBool,
    listen_keys: AtomicBool,
    pub view: Mutex<View>,
    draw_data: Mutex<DrawData>,
    browser: Mutex<Option<Browser>>,
    audio: Option<Arc<Audio>>, // static
    event_tx: Sender<Event>,
    callbacks: CallbackList,
    object_list: Mutex<HashSet<i32>>,
    rendered: (Mutex<u64>, Condvar),
}

#[derive(Clone)]
pub struct WebClientRef(Arc<WebClient>);

impl From<Arc<WebClient>> for WebClientRef {
    fn from(inner: Arc<WebClient>) -> Self {
        Self(inner)
    }
}

impl LifespanHandler for WebClientRef {
    fn on_after_created(&self, browser: Browser) {
        {
            let mut br = self.0.browser.lock();
            *br = Some(browser);
        }

        let hidden = self.0.hidden.load(Ordering::SeqCst);

        log::trace!("LifespanHandler::on_after_created. hidden: {}", hidden);

        self.0.hide(hidden);
    }

    fn on_before_close(&self, _: Browser) {
        log::trace!("LifespanHandler::on_before_close");

        let mut browser = self.0.browser.lock();

        if let Some(browser) = browser.take() {
            browser.host().close_dev_tools();
        }
    }
}

impl Client for WebClientRef {
    type LifespanHandler = Self;
    type RenderHandler = Self;
    type ContextMenuHandler = Self;
    type LoadHandler = Self;
    type AudioHandler = Self;

    fn lifespan_handler(&self) -> Option<Self> {
        Some(self.clone())
    }

    fn render_handler(&self) -> Option<Self> {
        Some(self.clone())
    }

    fn context_menu_handler(&self) -> Option<Self> {
        Some(self.clone())
    }

    fn load_handler(&self) -> Option<Self> {
        Some(self.clone())
    }

    fn audio_handler(&self) -> Option<Self> {
        if self.0.is_extern {
            Some(self.clone())
        } else {
            None
        }
    }

    fn on_process_message(
        &self, _browser: Browser, _frame: Frame, _source: ProcessId, msg: ProcessMessage,
    ) -> bool {
        let name = msg.name().to_string();

        log::trace!(
            "WebClient::on_process_message. browser_id: {}, message: {:?}",
            self.0.id,
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

                handle_result(self.0.event_tx.send(Event::FocusBrowser(self.0.id, focus)));

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

                handle_result(self.0.event_tx.send(Event::HideBrowser(self.0.id, hide)));

                return true;
            }

            "emit_event" => {
                let args = msg.argument_list();

                if args.get_type(0) != ValueType::String {
                    return true;
                }

                let event_name = args.string(0).to_string();
                let callbacks = self.0.callbacks.lock();

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
                handle_result(self.0.event_tx.send(event));
            }

            _ => (),
        }

        false
    }
}

impl ContextMenuHandler for WebClientRef {
    fn on_before_context_menu(&self, _: Browser, _: Frame, _: ContextMenuParams, model: MenuModel) {
        model.clear(); // remove context menu
    }
}

impl RenderHandler for WebClientRef {
    fn view_rect(&self, _: Browser, rect: &mut cef_rect_t) {
        let texture = self.0.view.lock();
        *rect = texture.rect();
    }

    fn on_popup_show(&self, _: Browser, show: bool) {
        let mut draw_data = self.0.draw_data.lock();
        draw_data.popup_show = show;

        if !show {
            draw_data.popup_buffer.clear();
            draw_data.popup_was_before = true; // REMOVE
        }
    }

    fn on_popup_size(&self, _: Browser, rect: &cef_rect_t) {
        let mut draw_data = self.0.draw_data.lock();

        draw_data.popup_rect = *rect;

        draw_data
            .popup_buffer
            .resize(rect.width as usize * rect.height as usize * 4, 0);
    }

    fn on_paint(
        &self, _: Browser, paint_type: PaintElement, mut dirty_rects: DirtyRects, buffer: &[u8],
        width: usize, height: usize,
    ) {
        let view = self.0.view.lock();

        if self.0.closing.load(Ordering::SeqCst) || view.is_empty() {
            return;
        }

        let generation = {
            let mut draw_data = self.0.draw_data.lock();

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
                        dirty_rects.rects.push(draw_data.popup_rect);
                    }

                    draw_data.rects = dirty_rects;
                    draw_data.height = height;
                    draw_data.width = width;
                    if draw_data.view_buffer.len() != buffer.len() {
                        draw_data.view_buffer.resize(buffer.len(), 0);
                    }
                    draw_data.view_buffer.copy_from_slice(buffer);
                    draw_data.changed = true;
                    draw_data.generation = draw_data.generation.wrapping_add(1);
                }
            }

            draw_data.generation
        };

        drop(view);

        let (mutex, cv) = &self.0.rendered;
        let mut rendered = mutex.lock();

        while *rendered < generation {
            if cv
                .wait_for(&mut rendered, Duration::from_secs(2))
                .timed_out()
                && *rendered < generation
            {
                drop(rendered);
                let mut draw_data = self.0.draw_data.lock();
                if draw_data.generation == generation {
                    draw_data.changed = false;
                    draw_data.view_buffer.clear();
                }
                return;
            }
        }

        drop(rendered);
        let mut draw_data = self.0.draw_data.lock();
        if draw_data.generation == generation {
            draw_data.changed = false;
        }
    }
}

impl LoadHandler for WebClientRef {
    fn on_load_end(&self, _browser: Browser, frame: Frame, status_code: i32) {
        log::trace!(
            "LoadHandler::on_load_end. id: {} status: {}",
            self.0.id,
            status_code
        );

        if frame.is_main() {
            let event = Event::BrowserCreated(self.0.id, status_code);
            handle_result(self.0.event_tx.send(event));
        }
    }
}

impl AudioHandler for WebClientRef {
    fn get_audio_parameters(&self, _browser: Browser, params: &mut cef_audio_parameters_t) -> bool {
        log::trace!(
            "get_audio_parameters: {} {} {}",
            params.sample_rate,
            params.channel_layout,
            params.frames_per_buffer
        );

        true
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn on_audio_stream_packet(
        &self, _browser: Browser, stream_id: i32, data: *mut *const f32, frames: i32, pts: i64,
    ) {
        if let Some(audio) = self.0.audio.as_ref() {
            unsafe {
                audio.append_pcm(self.0.id, stream_id, data, frames, pts as u64);
            }
        }
    }

    fn on_audio_stream_started(
        &self, _browser: Browser, stream_id: i32, channels: i32, _channel_layout: i32,
        sample_rate: i32, frames_per_buffer: i32,
    ) {
        if let Some(audio) = self.0.audio.as_ref() {
            audio.create_stream(
                self.0.id,
                stream_id,
                channels,
                sample_rate,
                frames_per_buffer,
            );
            let objects = self.0.object_list.lock();

            for &object_id in objects.iter() {
                audio.add_source(self.0.id, object_id);
            }
        }
    }

    fn on_audio_stream_stopped(&self, _browser: Browser, stream_id: i32) {
        if let Some(audio) = self.0.audio.as_ref() {
            audio.remove_stream(self.0.id, stream_id);
        }
    }

    fn on_audio_stream_error(&self, _browser: Browser, error: String) {
        log::trace!("on_audio_stream_error: {:?}", error);
    }
}

impl WebClient {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(id: u32, cbs: CallbackList, event_tx: Sender<Event>) -> Arc<WebClient> {
        let rect = crate::utils::client_rect();

        log::trace!("crate::utils::client_rect: {:?}", rect);

        let mut view = View::new();
        view.make_display(rect[0], rect[1]);

        let client = WebClient {
            hidden: AtomicBool::new(false),
            prev_hidden_flag: AtomicBool::new(false),
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
            rendered: (Mutex::new(0), Condvar::new()),
        };

        Arc::new(client)
    }

    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new_extern(
        id: u32, cbs: CallbackList, event_tx: Sender<Event>, audio: Arc<Audio>,
    ) -> Arc<WebClient> {
        let view = View::new();

        let client = WebClient {
            hidden: AtomicBool::new(false),
            prev_hidden_flag: AtomicBool::new(false),
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
            rendered: (Mutex::new(0), Condvar::new()),
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
        self.internal_hide(true, true); // hide browser but do not save value

        {
            let mut texture = self.view.lock();
            texture.on_lost_device();
            texture.make_inactive();
        }

        self.signal_rendered_current();
    }

    #[inline]
    pub fn on_reset_device(&self) {
        {
            let mut view = self.view.lock();

            if self.is_extern() {
            } else {
                let rect = crate::utils::client_rect();

                view.make_active();
                view.make_display(rect[0], rect[1]);

                self.notify_was_resized();
            }
        }

        self.restore_hide_status();
    }

    #[inline]
    pub fn resize(&self, width: usize, height: usize) {
        let mut view = self.view.lock();
        view.resize(self.is_extern(), width, height);
        self.notify_was_resized();

        self.signal_rendered_current();
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
            self.signal_rendered_current();
            return;
        }

        let generation = {
            let mut texture = self.view.lock();
            let mut draw_data = self.draw_data.lock();
            let size = texture.rect();
            let generation = draw_data.generation;

            if draw_data.changed
                && (size.height as usize != draw_data.height
                    || size.width as usize != draw_data.width)
            {
                draw_data.changed = false;
            }

            if draw_data.changed {
                if draw_data.height == 0 || draw_data.width == 0 {
                    draw_data.changed = false;
                }

                let expected_len = draw_data.width * draw_data.height * 4;

                if draw_data.view_buffer.len() < expected_len {
                    draw_data.changed = false;
                }

                if draw_data.rects.count > 0 {
                    let rect = &draw_data.rects.rects[0];
                    if rect.width > size.width || rect.height > size.height {
                        draw_data.changed = false;
                    }
                }

                if draw_data.changed {
                    let bytes = &draw_data.view_buffer[..expected_len];
                    texture.update_texture(bytes, draw_data.rects.as_slice());
                }
            }

            if draw_data.popup_show
                && draw_data.popup_rect.x + draw_data.popup_rect.width < size.width
                && draw_data.popup_rect.y + draw_data.popup_rect.height < size.height
            {
                texture.update_popup(&draw_data.popup_buffer, &draw_data.popup_rect);
            }

            generation
        };

        self.signal_rendered(generation);
    }

    #[inline(always)]
    fn signal_rendered(&self, generation: u64) {
        let (mutex, cv) = &self.rendered;

        {
            let mut rendered = mutex.lock();
            if generation > *rendered {
                *rendered = generation;
            }
        }

        cv.notify_all();
    }

    #[inline(always)]
    fn signal_rendered_current(&self) {
        let generation = self.draw_data.lock().generation;
        self.signal_rendered(generation);
    }

    #[inline(always)]
    pub fn browser(&self) -> Option<Browser> {
        let browser = self.browser.lock();

        browser.as_ref().cloned()
    }

    pub fn hide(&self, hide: bool) {
        self.internal_hide(hide, false);
    }

    pub fn internal_hide(&self, hide: bool, update_prev: bool) {
        if update_prev {
            let cur = self.hidden.load(Ordering::SeqCst);
            self.prev_hidden_flag.store(cur, Ordering::SeqCst);
        }

        self.hidden.store(hide, Ordering::SeqCst);
        self.signal_rendered_current();

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
        self.internal_hide(self.prev_hidden_flag.load(Ordering::SeqCst), false);
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
        self.view.lock().make_inactive();
    }

    pub fn toggle_dev_tools(&self, enabled: bool) {
        use winapi::um::winuser::*;

        let id = self.id();

        if let Some(host) = self.browser().map(|br| br.host()) {
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
        }
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
        self.signal_rendered_current();

        if let Some(host) = self.browser().map(|br| br.host()) {
            host.close_browser(force_close)
        }
    }

    pub fn always_listen_keys(&self) -> bool {
        self.listen_keys.load(Ordering::SeqCst)
    }

    pub fn set_always_listen_keys(&self, listen: bool) {
        self.listen_keys.store(listen, Ordering::SeqCst);
    }
}
