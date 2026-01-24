use crate::browser::{Browser, Frame};

pub trait LoadHandler {
    fn on_load_end(&self, _browser: Browser, _frame: Frame, _status_code: i32) {}

    fn on_loading_state_change(
        &self, _browser: Browser, _is_loading: bool, _can_go_back: bool, _can_go_forward: bool,
    ) {
    }
}
