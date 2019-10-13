use crate::browser::Browser;
use cef_sys::cef_rect_t;
use std::sync::Arc;

pub struct DirtyRects<'a> {
    pub count: usize,
    pub rects: &'a [cef_rect_t],
}

pub trait RenderHandler {
    fn view_rect(self: &Arc<Self>, browser: Browser, rect: &mut cef_rect_t);
    fn on_paint(
        self: &Arc<Self>, browser: Browser, paint_type: i32, dirty_rects: DirtyRects,
        buffer: &[u8], width: usize, height: usize,
    );
}

//pub struct DefaultRenderHandler;
//
//impl RenderHandler for DefaultRenderHandler {
//    fn view_rect(self: &Arc<Self>, browser: Browser, rect: &mut cef_rect_t) {
//
//    }
//}
