use crate::browser::{Browser, ContextMenuParams, Frame, MenuModel};

pub trait ContextMenuHandler {
    fn on_before_context_menu(
        &self, browser: Browser, frame: Frame, params: ContextMenuParams, model: MenuModel,
    );
}
