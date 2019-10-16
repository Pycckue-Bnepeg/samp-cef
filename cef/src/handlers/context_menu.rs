use crate::browser::{Browser, ContextMenuParams, Frame, MenuModel};
use std::sync::Arc;

pub trait ContextMenuHandler {
    fn on_before_context_menu(
        self: &Arc<Self>, browser: Browser, frame: Frame, params: ContextMenuParams,
        model: MenuModel,
    );
}
