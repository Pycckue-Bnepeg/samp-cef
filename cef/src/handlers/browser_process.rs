use std::sync::Arc;

pub trait BrowserProcessHandler {
    fn on_context_initialized(self: &Arc<Self>) {}
}
