use crate::handlers::v8handler::V8Handler;
use crate::rust_to_c::Wrapper;
use crate::types::string::CefString;
use crate::v8::V8Value;

use std::sync::Arc;

use cef_sys::{cef_string_t, cef_v8handler_t, cef_v8value_t};

unsafe extern "stdcall" fn execute<I: V8Handler>(
    this: *mut cef_v8handler_t, name: *const cef_string_t, object: *mut cef_v8value_t,
    argumentsCount: usize, arguments: *const *mut cef_v8value_t, retval: *mut *mut cef_v8value_t,
    exception: *mut cef_string_t,
) -> std::os::raw::c_int {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);

    let name = CefString::from(name);
    let object = V8Value::from_raw(object);

    let args: Vec<V8Value> = std::slice::from_raw_parts(arguments, argumentsCount)
        .iter()
        .map(|val| V8Value::from_raw(*val))
        .collect();

    let ret = obj.interface.execute(name, args);

    if ret {
        1
    } else {
        0
    }
}

pub fn wrap<T: V8Handler>(app: Arc<T>) -> *mut cef_v8handler_t {
    let mut object: cef_v8handler_t = unsafe { std::mem::zeroed() };

    object.execute = Some(execute::<T>);

    let wrapper = Wrapper::new(object, app);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
