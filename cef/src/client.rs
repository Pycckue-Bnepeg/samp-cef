use crate::handlers::audio::AudioHandler;
use crate::handlers::context_menu::ContextMenuHandler;
use crate::handlers::lifespan::LifespanHandler;
use crate::handlers::load::LoadHandler;
use crate::handlers::render::RenderHandler;

use crate::ProcessId;
use crate::browser::{Browser, Frame};
use crate::process_message::ProcessMessage;

pub trait Client {
    type LifespanHandler: LifespanHandler;
    type RenderHandler: RenderHandler;
    type ContextMenuHandler: ContextMenuHandler;
    type LoadHandler: LoadHandler;
    type AudioHandler: AudioHandler;

    fn lifespan_handler(&self) -> Option<Self::LifespanHandler> {
        None
    }

    fn render_handler(&self) -> Option<Self::RenderHandler> {
        None
    }

    fn context_menu_handler(&self) -> Option<Self::ContextMenuHandler> {
        None
    }

    fn load_handler(&self) -> Option<Self::LoadHandler> {
        None
    }

    fn audio_handler(&self) -> Option<Self::AudioHandler> {
        None
    }

    fn on_process_message(
        &self, _browser: Browser, _frame: Frame, _source: ProcessId, _message: ProcessMessage,
    ) -> bool {
        false
    }
}
