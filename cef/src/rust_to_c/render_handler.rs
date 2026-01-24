use super::Wrapper;
use crate::browser::Browser;
use crate::handlers::render::{DirtyRects, PaintElement, RenderHandler};

use cef_sys::{
    HCURSOR, cef_accessibility_handler_t, cef_browser_t, cef_cursor_info_t, cef_cursor_type_t,
    cef_drag_data_t, cef_drag_operations_mask_t, cef_paint_element_type_t, cef_range_t, cef_rect_t,
    cef_render_handler_t, cef_screen_info_t, cef_string_t, cef_text_input_mode_t,
};

unsafe extern "system" fn get_accessibility_handler(
    _this: *mut cef_render_handler_t,
) -> *mut cef_accessibility_handler_t {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.accessibility_handler();
    std::ptr::null_mut()
}

unsafe extern "system" fn get_root_screen_rect<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, _rect: *mut cef_rect_t,
) -> ::std::os::raw::c_int {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let _ = obj.ref_count.load(std::sync::atomic::Ordering::Relaxed);

    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }

    // let _ = obj.interface.root_screen_rect();
    0
}

unsafe extern "system" fn get_view_rect<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, rect: *mut cef_rect_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let rect = unsafe { &mut *rect };

    obj.interface.view_rect(browser, rect);
}

unsafe extern "system" fn get_screen_point(
    _this: *mut cef_render_handler_t, browser: *mut cef_browser_t, _view_x: ::std::os::raw::c_int,
    _view_y: ::std::os::raw::c_int, _screen_x: *mut ::std::os::raw::c_int,
    _screen_y: *mut ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    // let _ = obj.interface.screen_point();
    0
}

unsafe extern "system" fn get_screen_info(
    _this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    _screen_info: *mut cef_screen_info_t,
) -> ::std::os::raw::c_int {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    // let _ = obj.interface.screen_info();
    0
}

unsafe extern "system" fn on_popup_show<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, show: ::std::os::raw::c_int,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    let show = show == 1;
    obj.interface.on_popup_show(browser, show);
}

unsafe extern "system" fn on_popup_size<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t, rect: *const cef_rect_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);
    obj.interface.on_popup_size(browser, unsafe { &*rect });
}

unsafe extern "system" fn on_paint<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    type_: cef_paint_element_type_t::Type, dirty_rects_count: usize,
    dirty_rects: *const cef_rect_t, buffer: *const ::std::os::raw::c_void,
    width: ::std::os::raw::c_int, height: ::std::os::raw::c_int,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    let browser = Browser::from_raw(browser);

    let rects = unsafe { std::slice::from_raw_parts(dirty_rects, dirty_rects_count) };

    let rects = DirtyRects {
        count: dirty_rects_count,
        rects: Vec::from(rects),
    };

    let buffer =
        unsafe { std::slice::from_raw_parts(buffer as *const u8, (width * height * 4) as usize) };

    obj.interface.on_paint(
        browser,
        PaintElement::from(type_),
        rects,
        buffer,
        width as usize,
        height as usize,
    );
}

unsafe extern "system" fn on_accelerated_paint(
    _this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    _type_: cef_paint_element_type_t::Type, _dirty_rects_count: usize,
    _dirty_rects: *const cef_rect_t, _shared_handle: *mut ::std::os::raw::c_void,
) {
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    println!("accelerated");
    // let _ = obj.interface.on_accelerated_paint();
}

#[allow(dead_code)]
unsafe extern "system" fn on_cursor_change(
    _this: *mut cef_render_handler_t, browser: *mut cef_browser_t, _cursor: HCURSOR,
    _type_: cef_cursor_type_t::Type, _custom_cursor_info: *const cef_cursor_info_t,
) {
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.on_cursor_change();
}

unsafe extern "system" fn start_dragging(
    _this: *mut cef_render_handler_t, browser: *mut cef_browser_t, drag_data: *mut cef_drag_data_t,
    _allowed_ops: cef_drag_operations_mask_t, _x: ::std::os::raw::c_int, _y: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
        (*drag_data).base.release.unwrap()(&mut (*drag_data).base);
    }
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.start_dragging();
    0
}

unsafe extern "system" fn update_drag_cursor<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    operation: cef_drag_operations_mask_t,
) {
    let _obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    let _ = operation;
    // let _ = obj.interface.update_drag_cursor();
}

unsafe extern "system" fn on_scroll_offset_changed(
    _this: *mut cef_render_handler_t, browser: *mut cef_browser_t, _x: f64, _y: f64,
) {
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.on_scroll_offset_changed();
}

unsafe extern "system" fn oicrc<I: RenderHandler>(
    this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    _selected_range: *const cef_range_t, _character_bounds_count: usize,
    _character_bounds: *const cef_rect_t,
) {
    let _obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    // let _ = obj.interface.on_ime_composition_range_changed();
}

unsafe extern "system" fn on_text_selection_changed(
    _this: *mut cef_render_handler_t, browser: *mut cef_browser_t,
    _selected_text: *const cef_string_t, _selected_range: *const cef_range_t,
) {
    unsafe {
        (*browser).base.release.unwrap()(&mut (*browser).base);
    }
    //    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    // let _ = obj.interface.on_text_selection_changed();
}

unsafe extern "system" fn ovkr<I: RenderHandler>(
    _this: *mut cef_render_handler_t, _browser: *mut cef_browser_t,
    _input_mode: cef_text_input_mode_t::Type,
) {
    let _obj: &mut Wrapper<_, I> = Wrapper::unwrap(_this);
    let _ = Browser::from_raw(_browser);
    //    let _b = _input_mode * 4;
}

pub fn wrap<T: RenderHandler>(object: T) -> *mut cef_render_handler_t {
    let mut cef_object: cef_render_handler_t = unsafe { std::mem::zeroed() };

    cef_object.get_accessibility_handler = Some(get_accessibility_handler);
    cef_object.get_root_screen_rect = Some(get_root_screen_rect::<T>);
    cef_object.get_view_rect = Some(get_view_rect::<T>);
    cef_object.get_screen_point = Some(get_screen_point);
    cef_object.get_screen_info = Some(get_screen_info);
    cef_object.on_popup_show = Some(on_popup_show::<T>);
    cef_object.on_popup_size = Some(on_popup_size::<T>);
    cef_object.on_paint = Some(on_paint::<T>);
    cef_object.on_accelerated_paint = Some(on_accelerated_paint);
    // cef_object.on_cursor_change = Some(on_cursor_change::<T>);
    cef_object.start_dragging = Some(start_dragging);
    cef_object.update_drag_cursor = Some(update_drag_cursor::<T>);
    cef_object.on_scroll_offset_changed = Some(on_scroll_offset_changed);
    cef_object.on_ime_composition_range_changed = Some(oicrc::<T>);
    cef_object.on_text_selection_changed = Some(on_text_selection_changed);
    cef_object.on_virtual_keyboard_requested = Some(ovkr::<T>);

    let wrapper = Wrapper::new(cef_object, object);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
