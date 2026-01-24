use crate::ProcessId;
use crate::browser::{Browser, Frame};
use crate::process_message::ProcessMessage;
use crate::v8::V8Context;

pub trait RenderProcessHandler {
    fn on_context_created(&self, _browser: Browser, _frame: Frame, _context: V8Context) {}
    fn on_context_released(&self, _browser: Browser, _frame: Frame, _context: V8Context) {}
    fn on_webkit_initialized(&self) {}
    fn on_process_message(
        &self, _browser: Browser, _frame: Frame, _source: ProcessId, _message: ProcessMessage,
    ) -> bool {
        false
    }
}
