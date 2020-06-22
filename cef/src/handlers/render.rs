use crate::browser::Browser;
use cef_sys::{cef_paint_element_type_t, cef_rect_t};

use std::sync::Arc;

#[derive(Debug)]
pub struct DirtyRects {
    pub count: usize,
    pub rects: Vec<cef_rect_t>,
}

impl DirtyRects {
    pub fn as_slice(&self) -> &[cef_rect_t] {
        &self.rects
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PaintElement {
    Popup,
    View,
}

impl From<cef_paint_element_type_t::Type> for PaintElement {
    fn from(paint: cef_paint_element_type_t::Type) -> PaintElement {
        match paint {
            cef_paint_element_type_t::PET_VIEW => PaintElement::View,
            _ => PaintElement::Popup,
        }
    }
}

impl Into<cef_paint_element_type_t::Type> for PaintElement {
    fn into(self) -> cef_paint_element_type_t::Type {
        match self {
            PaintElement::View => cef_paint_element_type_t::PET_VIEW,
            PaintElement::Popup => cef_paint_element_type_t::PET_POPUP,
        }
    }
}

pub trait RenderHandler {
    fn view_rect(self: &Arc<Self>, browser: Browser, rect: &mut cef_rect_t);
    fn on_popup_show(self: &Arc<Self>, browser: Browser, show: bool);
    fn on_popup_size(self: &Arc<Self>, browser: Browser, rect: &cef_rect_t);
    fn on_paint(
        self: &Arc<Self>, browser: Browser, paint_type: PaintElement, dirty_rects: DirtyRects,
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
