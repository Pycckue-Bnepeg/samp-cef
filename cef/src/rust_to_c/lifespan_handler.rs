use super::Wrapper;
use crate::handlers::lifespan::LifespanHandler;

use crate::browser::Browser;
use cef_sys::{cef_browser_t, cef_life_span_handler_t};

extern "system" fn on_after_created<I: LifespanHandler>(
    this: *mut cef_life_span_handler_t, browser: *mut cef_browser_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let browser = Browser::from_raw_add_ref(browser);

    obj.interface.on_after_created(browser);
}

extern "system" fn on_before_close<I: LifespanHandler>(
    this: *mut cef_life_span_handler_t, browser: *mut cef_browser_t,
) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let browser = Browser::from_raw_borrowed(browser);

    obj.interface.on_before_close(browser);
}

pub fn wrap<T: LifespanHandler>(lifespan: T) -> *mut cef_life_span_handler_t {
    let mut object: cef_life_span_handler_t = unsafe { std::mem::zeroed() };
    object.on_before_close = Some(on_before_close::<T>);
    object.on_after_created = Some(on_after_created::<T>);

    let wrapper = Wrapper::new(object, lifespan);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
