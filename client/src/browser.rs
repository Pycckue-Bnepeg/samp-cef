use std::sync::{Arc, Mutex};

use cef::browser::{Browser, Frame};
use cef::client::Client;
use cef::handlers::render::DirtyRects;
use cef::handlers::{lifespan::LifespanHandler, render::RenderHandler};
use cef::process_message::ProcessMessage;
use cef::types::list::ValueType;
use cef::ProcessId;

use cef_sys::cef_rect_t;

use crate::application::Event;
use crate::view::View;
use cef::v8::V8Value;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use winapi::shared::windef::RECT;

pub struct WebClient {
    view: Mutex<Option<View>>,
    draw_data: Mutex<Vec<u8>>,
    browser: Mutex<Option<Browser>>,
    callbacks: Mutex<HashMap<String, V8Value>>,
    event_tx: Sender<Event>,
}

impl LifespanHandler for WebClient {
    fn on_after_created(self: &Arc<Self>, browser: Browser) {
        let view = crate::utils::client_rect();
        let mut draw_data = self.draw_data.lock().unwrap();
        *draw_data = vec![0u8; view[0] * view[1] * 4];
        self.create_view();
        let mut br = self.browser.lock().unwrap();
        *br = Some(browser);
    }

    fn on_before_close(self: &Arc<Self>, _: Browser) {}
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

        if name == "show_cursor" {
            let args = msg.argument_list();
            let value_type = args.get_type(0);

            let show = match value_type {
                ValueType::Integer => args.integer(0) == 1,
                ValueType::Bool => args.bool(0),
                _ => false,
            };

            self.event_tx.send(Event::ShowCursor(show));

            return true;
        }

        false
    }
}

impl RenderHandler for WebClient {
    fn view_rect(self: &Arc<Self>, _: Browser, rect: &mut cef_rect_t) {
        let texture = self.view.lock().unwrap();

        if let Some(rect_b) = texture.as_ref().map(|txt| txt.rect()) {
            *rect = rect_b;
        } else {
            rect.x = 0;
            rect.y = 0;
            rect.height = 0 as _;
            rect.width = 0 as _;
        }
    }

    fn on_paint(
        self: &Arc<Self>, _: Browser, paint_type: i32, dirty_rects: DirtyRects, buffer: &[u8],
        width: usize, height: usize,
    ) {
        if paint_type == cef_sys::cef_paint_element_type_t::PET_VIEW {
            let mut draw_data = self.draw_data.lock().unwrap();

            if draw_data.len() == buffer.len() {
                draw_data.copy_from_slice(buffer);
            }
        }
    }
}

impl WebClient {
    pub fn new(tx: Sender<Event>) -> Arc<WebClient> {
        let client = WebClient {
            view: Mutex::new(None),
            draw_data: Mutex::new(Vec::new()),
            browser: Mutex::new(None),
            callbacks: Mutex::new(HashMap::new()),
            event_tx: tx,
        };

        Arc::new(client)
    }

    pub fn create_view(&self) {
        let view = crate::utils::client_rect();
        let new = View::new(client_api::gta::d3d9::device(), view[0], view[1]);
        let mut texture = self.view.lock().unwrap();
        *texture = Some(new);
    }

    pub fn draw(&self) {
        let mut texture = self.view.lock().unwrap();
        if let Some(txt) = texture.as_mut() {
            txt.draw();
        }
    }

    pub fn on_device_lost(&self) {
        let mut texture = self.view.lock().unwrap();
        if let Some(txt) = texture.as_mut() {
            txt.on_device_lost();
        }
    }

    pub fn on_reset_device(&self) {
        let mut texture = self.view.lock().unwrap();
        if let Some(txt) = texture.as_mut() {
            let view = crate::utils::client_rect();
            txt.on_reset_device(client_api::gta::d3d9::device(), view[0], view[1]);
        }
    }

    pub fn update_view(&self) {
        let mut texture = self.view.lock().unwrap();
        if let Some(txt) = texture.as_mut() {
            let bytes = self.draw_data.lock().unwrap();
            txt.update_texture(&bytes);
        }
    }

    pub fn browser(&self) -> Option<Browser> {
        let browser = self.browser.lock().unwrap();

        if let Some(browser) = browser.as_ref() {
            Some(browser.clone())
        } else {
            None
        }
    }
}
