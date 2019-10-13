use crate::browser::Browser;
use crate::handlers::render_process::RenderProcessHandler;
use crate::v8::V8Context;
use std::sync::Arc;

pub trait App {
    type RenderProcessHandler: RenderProcessHandler;

    fn render_process_handler(self: &Arc<Self>) -> Option<Arc<Self::RenderProcessHandler>> {
        None
    }
}