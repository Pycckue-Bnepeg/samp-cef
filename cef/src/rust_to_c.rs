use cef_sys::cef_base_ref_counted_t;

use std::marker::PhantomData;
use std::sync::atomic::{self, AtomicUsize, Ordering};

pub mod app;
pub mod audio_handler;
pub mod browser_process_handler;
pub mod client;
pub mod context_menu_handler;
pub mod lifespan_handler;
pub mod load_handler;
pub mod render_handler;
pub mod render_process_handler;
pub mod task;
pub mod v8handler;

#[repr(C)]
pub(crate) struct Wrapper<T, I> {
    cef_object: T,
    interface: I,
    ref_count: AtomicUsize,
    marker: PhantomData<T>,
}

impl<T, I> Wrapper<T, I> {
    pub fn new(mut cef_object: T, interface: I) -> Wrapper<T, I> {
        let base = unsafe { &mut *(&mut cef_object as *mut T as *mut cef_base_ref_counted_t) };

        base.size = std::mem::size_of::<T>();

        base.add_ref = Some(add_ref::<T, I>);
        base.has_one_ref = Some(has_one_ref::<T, I>);
        base.has_at_least_one_ref = Some(has_at_least_one_ref::<T, I>);
        base.release = Some(release::<T, I>);

        Wrapper {
            cef_object,
            interface,
            ref_count: AtomicUsize::new(1),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn unwrap<'a>(ptr: *mut T) -> &'a mut Wrapper<T, I> {
        unsafe { &mut *(ptr as *mut Wrapper<T, I>) }
    }
}

#[inline(never)]
extern "system" fn add_ref<T, I>(this: *mut cef_base_ref_counted_t) {
    let obj: &mut Wrapper<T, I> = Wrapper::<T, I>::unwrap(this as *mut T);

    obj.ref_count.fetch_add(1, Ordering::Relaxed);
}

#[inline(never)]
extern "system" fn has_one_ref<T, I>(this: *mut cef_base_ref_counted_t) -> i32 {
    let obj: &mut Wrapper<T, I> = Wrapper::unwrap(this as *mut T);

    if obj.ref_count.load(Ordering::Relaxed) == 1 {
        1
    } else {
        0
    }
}

#[inline(never)]
extern "system" fn has_at_least_one_ref<T, I>(this: *mut cef_base_ref_counted_t) -> i32 {
    let obj: &mut Wrapper<T, I> = Wrapper::unwrap(this as *mut T);

    if obj.ref_count.load(Ordering::Relaxed) >= 1 {
        1
    } else {
        0
    }
}

#[inline(never)]
pub extern "system" fn release<T, I>(this: *mut cef_base_ref_counted_t) -> i32 {
    let obj: &mut Wrapper<T, I> = Wrapper::unwrap(this as *mut T);

    if obj.ref_count.fetch_sub(1, Ordering::Release) != 1 {
        0
    } else {
        atomic::fence(Ordering::Acquire);

        let _ = unsafe { Box::from_raw(this as *mut Wrapper<T, I>) };

        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use crate::app::App;
    use crate::client::Client;
    use crate::handlers::audio::AudioHandler;
    use crate::handlers::browser_process::BrowserProcessHandler;
    use crate::handlers::context_menu::ContextMenuHandler;
    use crate::handlers::lifespan::LifespanHandler;
    use crate::handlers::load::LoadHandler;
    use crate::handlers::render::{DirtyRects, PaintElement, RenderHandler};
    use crate::handlers::render_process::RenderProcessHandler;
    use crate::handlers::v8handler::V8Handler;
    use crate::task::Task;
    use crate::types::string::CefString;
    use crate::v8::V8Value;
    use cef_sys::{
        cef_app_t, cef_audio_handler_t, cef_browser_process_handler_t, cef_client_t,
        cef_context_menu_handler_t, cef_life_span_handler_t, cef_load_handler_t, cef_rect_t,
        cef_render_handler_t, cef_render_process_handler_t, cef_task_t, cef_v8handler_t,
    };

    struct DropTracker {
        drops: Cell<usize>,
    }

    struct Dummy {
        base: cef_base_ref_counted_t,
        state: *const DropTracker,
    }

    impl Drop for Dummy {
        fn drop(&mut self) {
            let state = unsafe { &*self.state };
            state.drops.set(state.drops.get() + 1);
        }
    }

    fn new_wrapper(state: &DropTracker) -> (*mut Wrapper<Dummy, ()>, *mut cef_base_ref_counted_t) {
        let cef_object = Dummy {
            base: unsafe { std::mem::zeroed() },
            state,
        };
        let wrapper = Box::new(Wrapper::new(cef_object, ()));
        let wrapper_ptr = Box::into_raw(wrapper);
        let base_ptr = unsafe {
            std::ptr::addr_of_mut!((*wrapper_ptr).cef_object) as *mut cef_base_ref_counted_t
        };
        (wrapper_ptr, base_ptr)
    }

    unsafe fn read_cef_object<T, I>(ptr: *mut T) -> T {
        let wrapper_ptr = ptr as *mut Wrapper<T, I>;
        std::ptr::read(std::ptr::addr_of!((*wrapper_ptr).cef_object))
    }

    unsafe fn drop_wrapper<T, I>(ptr: *mut T) {
        drop(Box::from_raw(ptr as *mut Wrapper<T, I>));
    }

    struct DummyRenderProcessHandler;
    impl RenderProcessHandler for DummyRenderProcessHandler {}

    struct DummyBrowserProcessHandler;
    impl BrowserProcessHandler for DummyBrowserProcessHandler {}

    struct DummyApp;
    impl App for DummyApp {
        type RenderProcessHandler = DummyRenderProcessHandler;
        type BrowserProcessHandler = DummyBrowserProcessHandler;
    }

    struct DummyV8Handler;
    impl V8Handler for DummyV8Handler {
        fn execute(&self, _name: CefString, _args: Vec<V8Value>) -> bool {
            false
        }
    }

    struct DummyRenderHandler;
    impl RenderHandler for DummyRenderHandler {
        fn view_rect(&self, _browser: crate::browser::Browser, _rect: &mut cef_rect_t) {}
        fn on_popup_show(&self, _browser: crate::browser::Browser, _show: bool) {}
        fn on_popup_size(&self, _browser: crate::browser::Browser, _rect: &cef_rect_t) {}
        fn on_paint(
            &self, _browser: crate::browser::Browser, _paint_type: PaintElement,
            _dirty_rects: DirtyRects, _buffer: &[u8], _width: usize, _height: usize,
        ) {
        }
    }

    struct DummyContextMenuHandler;
    impl ContextMenuHandler for DummyContextMenuHandler {
        fn on_before_context_menu(
            &self, _browser: crate::browser::Browser, _frame: crate::browser::Frame,
            _params: crate::browser::ContextMenuParams, _model: crate::browser::MenuModel,
        ) {
        }
    }

    struct DummyLifespanHandler;
    impl LifespanHandler for DummyLifespanHandler {
        fn on_after_created(&self, _browser: crate::browser::Browser) {}
        fn on_before_close(&self, _browser: crate::browser::Browser) {}
    }

    struct DummyLoadHandler;
    impl LoadHandler for DummyLoadHandler {}

    struct DummyAudioHandler;
    impl AudioHandler for DummyAudioHandler {}

    struct DummyClient;
    impl Client for DummyClient {
        type LifespanHandler = DummyLifespanHandler;
        type RenderHandler = DummyRenderHandler;
        type ContextMenuHandler = DummyContextMenuHandler;
        type LoadHandler = DummyLoadHandler;
        type AudioHandler = DummyAudioHandler;
    }

    struct DummyTask;
    impl Task for DummyTask {
        fn execute(&self) {}
    }

    #[test]
    fn wrapper_initializes_base() {
        let state = DropTracker {
            drops: Cell::new(0),
        };
        let (wrapper_ptr, base_ptr) = new_wrapper(&state);

        unsafe {
            let base = std::ptr::read(base_ptr);
            assert_eq!(base.size, std::mem::size_of::<Dummy>());
            assert!(base.add_ref.is_some());
            assert!(base.has_one_ref.is_some());
            assert!(base.has_at_least_one_ref.is_some());
            assert!(base.release.is_some());

            let _ = Box::from_raw(wrapper_ptr);
        }
    }

    #[test]
    fn ref_counting_via_base_callbacks() {
        let state = DropTracker {
            drops: Cell::new(0),
        };
        let (wrapper_ptr, base_ptr) = new_wrapper(&state);

        unsafe {
            let base = std::ptr::read(base_ptr);
            base.add_ref.unwrap()(base_ptr);
            assert_eq!((*wrapper_ptr).ref_count.load(Ordering::Relaxed), 2);
            assert_eq!(base.has_one_ref.unwrap()(base_ptr), 0);
            assert_eq!(base.has_at_least_one_ref.unwrap()(base_ptr), 1);

            assert_eq!(base.release.unwrap()(base_ptr), 0);
            assert_eq!((*wrapper_ptr).ref_count.load(Ordering::Relaxed), 1);
            assert_eq!(state.drops.get(), 0);

            assert_eq!(base.release.unwrap()(base_ptr), 1);
        }

        assert_eq!(state.drops.get(), 1);
    }

    #[test]
    fn wrap_app_sets_callbacks() {
        let ptr = super::app::wrap(DummyApp);
        unsafe {
            let cef: cef_app_t = read_cef_object::<cef_app_t, DummyApp>(ptr);
            assert!(cef.get_render_process_handler.is_some());
            assert!(cef.get_browser_process_handler.is_some());
            assert!(cef.on_before_command_line_processing.is_some());
            drop_wrapper::<cef_app_t, DummyApp>(ptr);
        }
    }

    #[test]
    fn wrap_browser_process_handler_sets_callbacks() {
        let ptr = super::browser_process_handler::wrap(DummyBrowserProcessHandler);
        unsafe {
            let cef: cef_browser_process_handler_t =
                read_cef_object::<cef_browser_process_handler_t, DummyBrowserProcessHandler>(ptr);
            assert!(cef.on_context_initialized.is_some());
            drop_wrapper::<cef_browser_process_handler_t, DummyBrowserProcessHandler>(ptr);
        }
    }

    #[test]
    fn wrap_v8handler_sets_callbacks() {
        let ptr = super::v8handler::wrap(DummyV8Handler);
        unsafe {
            let cef: cef_v8handler_t = read_cef_object::<cef_v8handler_t, DummyV8Handler>(ptr);
            assert!(cef.execute.is_some());
            drop_wrapper::<cef_v8handler_t, DummyV8Handler>(ptr);
        }
    }

    #[test]
    fn wrap_render_process_handler_sets_callbacks() {
        let ptr = super::render_process_handler::wrap(DummyRenderProcessHandler);
        unsafe {
            let cef: cef_render_process_handler_t =
                read_cef_object::<cef_render_process_handler_t, DummyRenderProcessHandler>(ptr);
            assert!(cef.on_context_created.is_some());
            assert!(cef.on_context_released.is_some());
            assert!(cef.on_web_kit_initialized.is_some());
            assert!(cef.on_process_message_received.is_some());
            drop_wrapper::<cef_render_process_handler_t, DummyRenderProcessHandler>(ptr);
        }
    }

    #[test]
    fn wrap_render_handler_sets_callbacks() {
        let ptr = super::render_handler::wrap(DummyRenderHandler);
        unsafe {
            let cef: cef_render_handler_t =
                read_cef_object::<cef_render_handler_t, DummyRenderHandler>(ptr);
            assert!(cef.get_accessibility_handler.is_some());
            assert!(cef.get_root_screen_rect.is_some());
            assert!(cef.get_view_rect.is_some());
            assert!(cef.get_screen_point.is_some());
            assert!(cef.get_screen_info.is_some());
            assert!(cef.on_popup_show.is_some());
            assert!(cef.on_popup_size.is_some());
            assert!(cef.on_paint.is_some());
            assert!(cef.on_accelerated_paint.is_some());
            assert!(cef.start_dragging.is_some());
            assert!(cef.update_drag_cursor.is_some());
            assert!(cef.on_scroll_offset_changed.is_some());
            assert!(cef.on_ime_composition_range_changed.is_some());
            assert!(cef.on_text_selection_changed.is_some());
            assert!(cef.on_virtual_keyboard_requested.is_some());
            drop_wrapper::<cef_render_handler_t, DummyRenderHandler>(ptr);
        }
    }

    #[test]
    fn wrap_context_menu_handler_sets_callbacks() {
        let ptr = super::context_menu_handler::wrap(DummyContextMenuHandler);
        unsafe {
            let cef: cef_context_menu_handler_t =
                read_cef_object::<cef_context_menu_handler_t, DummyContextMenuHandler>(ptr);
            assert!(cef.on_before_context_menu.is_some());
            drop_wrapper::<cef_context_menu_handler_t, DummyContextMenuHandler>(ptr);
        }
    }

    #[test]
    fn wrap_lifespan_handler_sets_callbacks() {
        let ptr = super::lifespan_handler::wrap(DummyLifespanHandler);
        unsafe {
            let cef: cef_life_span_handler_t =
                read_cef_object::<cef_life_span_handler_t, DummyLifespanHandler>(ptr);
            assert!(cef.on_before_close.is_some());
            assert!(cef.on_after_created.is_some());
            drop_wrapper::<cef_life_span_handler_t, DummyLifespanHandler>(ptr);
        }
    }

    #[test]
    fn wrap_load_handler_sets_callbacks() {
        let ptr = super::load_handler::wrap(DummyLoadHandler);
        unsafe {
            let cef: cef_load_handler_t =
                read_cef_object::<cef_load_handler_t, DummyLoadHandler>(ptr);
            assert!(cef.on_loading_state_change.is_some());
            assert!(cef.on_load_end.is_some());
            drop_wrapper::<cef_load_handler_t, DummyLoadHandler>(ptr);
        }
    }

    #[test]
    fn wrap_audio_handler_sets_callbacks() {
        let ptr = super::audio_handler::wrap(DummyAudioHandler);
        unsafe {
            let cef: cef_audio_handler_t =
                read_cef_object::<cef_audio_handler_t, DummyAudioHandler>(ptr);
            assert!(cef.get_audio_parameters.is_some());
            assert!(cef.on_audio_stream_packet.is_some());
            assert!(cef.on_audio_stream_started.is_some());
            assert!(cef.on_audio_stream_stopped.is_some());
            assert!(cef.on_audio_stream_error.is_some());
            drop_wrapper::<cef_audio_handler_t, DummyAudioHandler>(ptr);
        }
    }

    #[test]
    fn wrap_client_sets_callbacks() {
        let ptr = super::client::wrap(DummyClient);
        unsafe {
            let cef: cef_client_t = read_cef_object::<cef_client_t, DummyClient>(ptr);
            assert!(cef.get_life_span_handler.is_some());
            assert!(cef.on_process_message_received.is_some());
            assert!(cef.get_request_handler.is_some());
            assert!(cef.get_render_handler.is_some());
            assert!(cef.get_load_handler.is_some());
            assert!(cef.get_keyboard_handler.is_some());
            assert!(cef.get_jsdialog_handler.is_some());
            assert!(cef.get_focus_handler.is_some());
            assert!(cef.get_find_handler.is_some());
            assert!(cef.get_drag_handler.is_some());
            assert!(cef.get_download_handler.is_some());
            assert!(cef.get_display_handler.is_some());
            assert!(cef.get_dialog_handler.is_some());
            assert!(cef.get_context_menu_handler.is_some());
            assert!(cef.get_audio_handler.is_some());
            drop_wrapper::<cef_client_t, DummyClient>(ptr);
        }
    }

    #[test]
    fn wrap_task_sets_callbacks() {
        let ptr = super::task::wrap(DummyTask);
        unsafe {
            let cef: cef_task_t = read_cef_object::<cef_task_t, DummyTask>(ptr);
            assert!(cef.execute.is_some());
            drop_wrapper::<cef_task_t, DummyTask>(ptr);
        }
    }
}
