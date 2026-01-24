use crate::handlers::browser_process::BrowserProcessHandler;
use crate::handlers::render_process::RenderProcessHandler;

use crate::command_line::CommandLine;
use crate::types::string::CefString;
pub trait App {
    type RenderProcessHandler: RenderProcessHandler;
    type BrowserProcessHandler: BrowserProcessHandler;

    fn render_process_handler(&self) -> Option<Self::RenderProcessHandler> {
        None
    }
    fn browser_process_handler(&self) -> Option<Self::BrowserProcessHandler> {
        None
    }

    fn on_before_command_line_processing(
        &self, _process_type: CefString, _command_line: CommandLine,
    ) {
    }
}
