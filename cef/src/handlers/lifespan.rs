use crate::browser::Browser;
use std::sync::Arc;

pub trait LifespanHandler {
    fn on_after_created(self: &Arc<Self>, browser: Browser);
    fn on_before_close(self: &Arc<Self>, browser: Browser);
}
