use std::sync::{Arc, Condvar, Mutex};

use crossbeam_channel::Sender;

use cef::browser::{Browser, Frame};
use cef::client::Client;
use cef::handlers::lifespan::LifespanHandler;
use cef::handlers::render::{DirtyRects, PaintElement, RenderHandler};
use cef::process_message::ProcessMessage;
use cef::types::list::ValueType;
use cef::ProcessId;

use cef_sys::cef_rect_t;

use crate::app::Event;
use crate::browser::view::View;

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
    view: Mutex<View>,
    draw_data: Mutex<DrawData>,
    browser: Mutex<Option<Browser>>,
    rendered: (Mutex<bool>, Condvar),
    last_texture: Mutex<Option<Vec<u8>>>,
    event_tx: Sender<Event>,
}

impl LifespanHandler for WebClient {
    fn on_after_created(self: &Arc<Self>, browser: Browser) {
        let mut br = self.browser.lock().unwrap();
        *br = Some(browser);
    }

    fn on_before_close(self: &Arc<Self>, _: Browser) {
        let mut browser = self.browser.lock().unwrap();
        browser.take();
    }
}

impl Client for WebClient {
    type LifespanHandler = Self;
    type RenderHandler = Self;

    fn lifespan_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn render_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }

    fn on_process_message(
        self: &Arc<Self>, _browser: Browser, _frame: Frame, _source: ProcessId, msg: ProcessMessage,
    ) -> bool {
        let name = msg.name().to_string();

        match name.as_str() {
            "block_input" => {
                let args = msg.argument_list();
                let value_type = args.get_type(0);

                let block = match value_type {
                    ValueType::Integer => args.integer(0) == 1,
                    ValueType::Bool => args.bool(0),
                    _ => false,
                };

                self.event_tx.send(Event::BlockInput(block));

                return true;
            }

            "emit_event" => {}

            _ => (),
        }

        false
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
    pub fn new(event_tx: Sender<Event>) -> Arc<WebClient> {
        let rect = crate::utils::client_rect();
        let view = View::new(client_api::gta::d3d9::device(), rect[0], rect[1]);

        let client = WebClient {
            view: Mutex::new(view),
            draw_data: Mutex::new(DrawData::new()),
            browser: Mutex::new(None),
            rendered: (Mutex::new(false), Condvar::new()),
            last_texture: Mutex::new(None),
            event_tx,
        };

        Arc::new(client)
    }

    pub fn draw(&self) {
        let mut texture = self.view.lock().unwrap();
        texture.draw();
    }

    pub fn on_lost_device(&self) {
        let mut texture = self.view.lock().unwrap();
        let mut buffer = self.last_texture.lock().unwrap();
        *buffer = texture.buffer();
        texture.on_lost_device();
    }

    pub fn on_reset_device(&self) {
        let mut texture = self.view.lock().unwrap();
        let mut buffer = self.last_texture.lock().unwrap();
        let rect = crate::utils::client_rect();
        texture.on_reset_device(client_api::gta::d3d9::device(), rect[0], rect[1]);

        if let Some(buffer) = buffer.take() {
            if rect[0] * rect[1] * 4 != buffer.len() {
                return;
            }

            let rect = cef_rect_t {
                x: 0,
                y: 0,
                width: rect[0] as i32,
                height: rect[1] as i32,
            };

            texture.update_texture(&buffer, &[rect]);
        }
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
}
