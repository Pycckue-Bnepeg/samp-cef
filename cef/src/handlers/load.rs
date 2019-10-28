use crate::browser::{Browser, Frame};
use std::sync::Arc;

pub trait LoadHandler {
    fn on_load_end(self: &Arc<Self>, browser: Browser, frame: Frame, status_code: i32) {}

    fn on_loading_state_change(
        self: &Arc<Self>, browser: Browser, is_loading: bool, can_go_back: bool,
        can_go_forward: bool,
    ) {
    }
}
