use crate::browser::Browser;
use crate::handlers::browser_process::BrowserProcessHandler;
use crate::handlers::render_process::RenderProcessHandler;
use crate::v8::V8Context;

use crate::command_line::CommandLine;
use crate::types::string::CefString;
use std::sync::Arc;

pub trait App {
    type RenderProcessHandler: RenderProcessHandler;
    type BrowserProcessHandler: BrowserProcessHandler;

    fn render_process_handler(self: &Arc<Self>) -> Option<Arc<Self::RenderProcessHandler>> {
        None
    }
    fn browser_process_handler(self: &Arc<Self>) -> Option<Arc<Self::BrowserProcessHandler>> {
        None
    }

    fn on_before_command_line_processing(
        self: &Arc<Self>, process_type: CefString, command_line: CommandLine,
    ) {
    }
}
