use super::Wrapper;
use crate::browser::Browser;
use crate::handlers::render::{DirtyRects, PaintElement, RenderHandler};

use cef_sys::{
    cef_accessibility_handler_t, cef_browser_t, cef_cursor_info_t, cef_cursor_type_t,
    cef_drag_data_t, cef_drag_operations_mask_t, cef_paint_element_type_t, cef_range_t, cef_rect_t,
    cef_render_handler_t, cef_screen_info_t, cef_string_t, cef_text_input_mode_t, HCURSOR,
};

use std::sync::Arc;

unsafe extern "stdcall" fn get_accessibility_handler<I: RenderHandler>(
    this: *mut cef_render_handler_t,
) -> *mut cef_accessibility_handler_t {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.accessibility_handler();
    return 0 as _;
}

unsafe extern "stdcall" fn get_root_screen_rect<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, rect: *mut cef_rect_t,
) -> ::std::os::raw::c_int {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    if obj.ref_count.load(std::sync::atomic::Ordering::Relaxed) > 0 {
        ()
    }

    (*browser).base.release.unwrap()(&mut (*browser).base);

    // let _ = obj.interface.root_screen_rect();
    return 0 as _;
}

unsafe extern "stdcall" fn get_view_rect<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, rect: *mut cef_rect_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let rect = &mut *rect;

    obj.interface.view_rect(browser, rect);
}

unsafe extern "stdcall" fn get_screen_point<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, viewX: ::std::os::raw::c_int,
    viewY: ::std::os::raw::c_int, screenX: *mut ::std::os::raw::c_int,
    screenY: *mut ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    (*browser).base.release.unwrap()(&mut (*browser).base);
    // let _ = obj.interface.screen_point();
    return 0 as _;
}

unsafe extern "stdcall" fn get_screen_info<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    screen_info: *mut cef_screen_info_t,
) -> ::std::os::raw::c_int {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    (*browser).base.release.unwrap()(&mut (*browser).base);
    // let _ = obj.interface.screen_info();
    return 0 as _;
}

unsafe extern "stdcall" fn on_popup_show<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, show: ::std::os::raw::c_int,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let show = show == 1;
    obj.interface.on_popup_show(browser, show);
}

unsafe extern "stdcall" fn on_popup_size<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, rect: *const cef_rect_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    obj.interface.on_popup_size(browser, &*rect);
}

unsafe extern "stdcall" fn on_paint<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    type_: cef_paint_element_type_t::Type, dirtyRectsCount: usize, dirtyRects: *const cef_rect_t,
    buffer: *const ::std::os::raw::c_void, width: ::std::os::raw::c_int,
    height: ::std::os::raw::c_int,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    let rects = std::slice::from_raw_parts(dirtyRects, dirtyRectsCount);

    let rects = DirtyRects {
        count: dirtyRectsCount,
        rects: Vec::from(rects),
    };

    let buffer = std::slice::from_raw_parts(buffer as *const u8, (width * height * 4) as usize);

    obj.interface.on_paint(
        browser,
        PaintElement::from(type_),
        rects,
        buffer,
        width as usize,
        height as usize,
    );
}

unsafe extern "stdcall" fn on_accelerated_paint<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    type_: cef_paint_element_type_t::Type, dirtyRectsCount: usize, dirtyRects: *const cef_rect_t,
    shared_handle: *mut ::std::os::raw::c_void,
) {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    (*browser).base.release.unwrap()(&mut (*browser).base);
    println!("accelerated");
    // let _ = obj.interface.on_accelerated_paint();
}

unsafe extern "stdcall" fn on_cursor_change<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, cursor: HCURSOR,
    type_: cef_cursor_type_t::Type, custom_cursor_info: *const cef_cursor_info_t,
) {
    (*browser).base.release.unwrap()(&mut (*browser).base);
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.on_cursor_change();
}

unsafe extern "stdcall" fn start_dragging<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, drag_data: *mut cef_drag_data_t,
    allowed_ops: cef_drag_operations_mask_t, x: ::std::os::raw::c_int, y: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    (*browser).base.release.unwrap()(&mut (*browser).base);
    (*drag_data).base.release.unwrap()(&mut (*drag_data).base);
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.start_dragging();
    return 0 as _;
}

unsafe extern "stdcall" fn update_drag_cursor<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    operation: cef_drag_operations_mask_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    (*browser).base.release.unwrap()(&mut (*browser).base);
    let _a = operation.0 * 4;
    // let _ = obj.interface.update_drag_cursor();
}

unsafe extern "stdcall" fn on_scroll_offset_changed<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, x: f64, y: f64,
) {
    (*browser).base.release.unwrap()(&mut (*browser).base);
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.on_scroll_offset_changed();
}

unsafe extern "stdcall" fn oicrc<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    selected_range: *const cef_range_t, character_boundsCount: usize,
    character_bounds: *const cef_rect_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    (*browser).base.release.unwrap()(&mut (*browser).base);
    // let _ = obj.interface.on_ime_composition_range_changed();
}

unsafe extern "stdcall" fn on_text_selection_changed<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    selected_text: *const cef_string_t, selected_range: *const cef_range_t,
) {
    (*browser).base.release.unwrap()(&mut (*browser).base);
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.on_text_selection_changed();
}

unsafe extern "stdcall" fn ovkr<I: RenderHandler>(
    _this: *mut cef_render_handler_t, _browser: *mut cef_browser_t,
    _input_mode: cef_text_input_mode_t::Type,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(_this);
    let _a = Browser::from_raw(_browser);
    //    let _b = _input_mode * 4;
}

pub fn wrap<T: RenderHandler>(object: Arc<T>) -> *mut cef_render_handler_t {
    let mut cef_object: cef_render_handler_t = unsafe { std::mem::zeroed() };

    cef_object.get_accessibility_handler = Some(get_accessibility_handler::<T>);
    cef_object.get_root_screen_rect = Some(get_root_screen_rect::<T>);
    cef_object.get_view_rect = Some(get_view_rect::<T>);
    cef_object.get_screen_point = Some(get_screen_point::<T>);
    cef_object.get_screen_info = Some(get_screen_info::<T>);
    cef_object.on_popup_show = Some(on_popup_show::<T>);
    cef_object.on_popup_size = Some(on_popup_size::<T>);
    cef_object.on_paint = Some(on_paint::<T>);
    cef_object.on_accelerated_paint = Some(on_accelerated_paint::<T>);
    // cef_object.on_cursor_change = Some(on_cursor_change::<T>);
    cef_object.start_dragging = Some(start_dragging::<T>);
    cef_object.update_drag_cursor = Some(update_drag_cursor::<T>);
    cef_object.on_scroll_offset_changed = Some(on_scroll_offset_changed::<T>);
    cef_object.on_ime_composition_range_changed = Some(oicrc::<T>);
    cef_object.on_text_selection_changed = Some(on_text_selection_changed::<T>);
    cef_object.on_virtual_keyboard_requested = Some(ovkr::<T>);

    let wrapper = Wrapper::new(cef_object, object);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
