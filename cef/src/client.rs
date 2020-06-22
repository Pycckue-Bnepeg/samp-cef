use crate::handlers::audio::AudioHandler;
use crate::handlers::context_menu::ContextMenuHandler;
use crate::handlers::lifespan::LifespanHandler;
use crate::handlers::load::LoadHandler;
use crate::handlers::render::RenderHandler;

use crate::browser::{Browser, Frame};
use crate::process_message::ProcessMessage;
use crate::ProcessId;

use std::sync::Arc;

pub trait Client {
    type LifespanHandler: LifespanHandler;
    type RenderHandler: RenderHandler;
    type ContextMenuHandler: ContextMenuHandler;
    type LoadHandler: LoadHandler;
    type AudioHandler: AudioHandler;

    fn lifespan_handler(self: &Arc<Self>) -> Option<Arc<Self::LifespanHandler>> {
        None
    }

    fn render_handler(self: &Arc<Self>) -> Option<Arc<Self::RenderHandler>> {
        None
    }

    fn context_menu_handler(self: &Arc<Self>) -> Option<Arc<Self::ContextMenuHandler>> {
        None
    }

    fn load_handler(self: &Arc<Self>) -> Option<Arc<Self::LoadHandler>> {
        None
    }

    fn audio_handler(self: &Arc<Self>) -> Option<Arc<Self::AudioHandler>> {
        None
    }

    fn on_process_message(
        self: &Arc<Self>, browser: Browser, frame: Frame, source: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        false
    }
}
