use crate::cef_app::BrowserManager;
use crossbeam_channel::{Receiver, Sender};

static mut APP: Option<Application> = None;

pub enum Event {
    ShowCursor(bool),
}

pub struct Application {
    pub render_init: bool,
    pub cursor: bool,
    pub manager: BrowserManager,
    pub event_rx: Receiver<Event>,
    pub event_tx: Sender<Event>,
}

impl Application {
    pub fn create() {
        let manager = BrowserManager::new();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let app = Application {
            manager,
            event_tx,
            event_rx,
            render_init: false,
            cursor: false,
        };

        unsafe {
            APP = Some(app);
        }
    }

    pub fn get() -> Option<&'static mut Application> {
        unsafe { APP.as_mut() }
    }
}
