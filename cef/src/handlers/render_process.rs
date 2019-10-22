use crate::browser::{Browser, Frame};
use crate::process_message::ProcessMessage;
use crate::v8::V8Context;
use crate::ProcessId;

use std::sync::Arc;

pub trait RenderProcessHandler {
    fn on_context_created(self: &Arc<Self>, browser: Browser, frame: Frame, context: V8Context) {}
    fn on_context_released(self: &Arc<Self>, browser: Browser, frame: Frame, context: V8Context) {}
    fn on_webkit_initialized(self: &Arc<Self>) {}
    fn on_process_message(
        self: &Arc<Self>, browser: Browser, frame: Frame, source: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        false
    }
}
