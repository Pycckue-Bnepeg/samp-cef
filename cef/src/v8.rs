use crate::handlers::v8handler::V8Handler;
use crate::ref_counted::RefGuard;
use crate::types::string::CefString;
use cef_sys::{cef_v8context_t, cef_v8value_t};
use std::sync::Arc;

#[derive(Clone)]
pub struct V8Context {
    inner: RefGuard<cef_v8context_t>,
}

impl V8Context {
    pub(crate) fn from_raw(raw: *mut cef_v8context_t) -> V8Context {
        if raw.is_null() {
            panic!("V8Context::from_raw null pointer");
        }

        V8Context {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn global(&self) -> V8Value {
        let ptr = self
            .inner
            .get_global
            .map(|get_global| unsafe { get_global(self.inner.get_mut()) })
            .unwrap_or(std::ptr::null_mut());

        V8Value::from_raw(ptr)
    }
}

#[derive(Clone)]
pub struct V8Value {
    inner: RefGuard<cef_v8value_t>,
}

impl V8Value {
    pub(crate) fn from_raw(raw: *mut cef_v8value_t) -> V8Value {
        if raw.is_null() {
            panic!("V8Value::from_raw null pointer");
        }

        V8Value {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn new_function<T: V8Handler>(name: &str, handler: Option<Arc<T>>) -> V8Value {
        let name = CefString::new(name);
        let handler = handler
            .map(|handler| crate::rust_to_c::v8handler::wrap(handler))
            .unwrap_or(std::ptr::null_mut());

        let func = unsafe { cef_sys::cef_v8value_create_function(name.as_cef_string(), handler) };

        V8Value::from_raw(func)
    }

    pub fn new_bool(value: bool) -> V8Value {
        let raw = unsafe { cef_sys::cef_v8value_create_bool(if value { 1 } else { 0 }) };

        V8Value::from_raw(raw)
    }

    pub fn new_integer(value: i32) -> V8Value {
        let raw = unsafe { cef_sys::cef_v8value_create_int(value) };

        V8Value::from_raw(raw)
    }

    pub fn new_double(value: f64) -> V8Value {
        let raw = unsafe { cef_sys::cef_v8value_create_double(value) };

        V8Value::from_raw(raw)
    }

    pub fn new_string(string: &str) -> V8Value {
        let cef_string = CefString::new(string);

        let raw = unsafe { cef_sys::cef_v8value_create_string(cef_string.as_cef_string()) };

        V8Value::from_raw(raw)
    }

    pub fn new_cefstring(string: &CefString) -> V8Value {
        let raw = unsafe { cef_sys::cef_v8value_create_string(string.as_cef_string()) };

        V8Value::from_raw(raw)
    }

    pub fn new_object() -> V8Value {
        let raw = unsafe {
            cef_sys::cef_v8value_create_object(std::ptr::null_mut(), std::ptr::null_mut())
        };

        V8Value::from_raw(raw)
    }

    pub fn is_string(&self) -> bool {
        self.inner
            .is_string
            .map(|is| unsafe { is(self.inner.get_mut()) })
            .map(|int| int == 1)
            .unwrap_or(false)
    }

    pub fn is_integer(&self) -> bool {
        self.inner
            .is_int
            .map(|is| unsafe { is(self.inner.get_mut()) })
            .map(|int| int == 1)
            .unwrap_or(false)
    }

    pub fn is_bool(&self) -> bool {
        self.inner
            .is_bool
            .map(|is| unsafe { is(self.inner.get_mut()) })
            .map(|int| int == 1)
            .unwrap_or(false)
    }

    pub fn bool(&self) -> bool {
        self.inner
            .get_bool_value
            .map(|get| unsafe { get(self.inner.get_mut()) })
            .map(|int| int == 1)
            .unwrap_or(false)
    }

    pub fn is_double(&self) -> bool {
        self.inner
            .is_double
            .map(|is| unsafe { is(self.inner.get_mut()) })
            .map(|int| int == 1)
            .unwrap_or(false)
    }

    pub fn string(&self) -> CefString {
        self.inner
            .get_string_value
            .map(|get_str| unsafe { get_str(self.inner.get_mut()) })
            .map(|string| CefString::from(string))
            .unwrap_or_else(|| CefString::new_null())
    }

    pub fn integer(&self) -> i32 {
        self.inner
            .get_int_value
            .map(|get| unsafe { get(self.inner.get_mut()) })
            .unwrap_or(0)
    }

    pub fn double(&self) -> f64 {
        self.inner
            .get_double_value
            .map(|get| unsafe { get(self.inner.get_mut()) })
            .unwrap_or(0.0)
    }

    pub fn is_function(&self) -> bool {
        self.inner
            .is_function
            .map(|is| unsafe { is(self.inner.get_mut()) })
            .map(|int| int == 1)
            .unwrap_or(false)
    }

    pub fn execute_function(&self, this: Option<V8Value>, arguments: Vec<V8Value>) {
        let exec = self.inner.execute_function.unwrap();
        let this = this
            .map(|this| this.inner.into_cef())
            .unwrap_or(std::ptr::null_mut());

        let args: Vec<*mut cef_v8value_t> =
            arguments.into_iter().map(|v| v.inner.into_cef()).collect();

        unsafe {
            exec(self.inner.get_mut(), this, args.len(), args.as_ptr());
        }
    }

    pub fn execute_function_with_context(
        &self, this: Option<V8Value>, context: &V8Context, arguments: &[V8Value],
    ) {
        let exec = self.inner.execute_function_with_context.unwrap();
        let this = this
            .map(|this| this.inner.into_cef())
            .unwrap_or(std::ptr::null_mut());

        let args: Vec<*mut cef_v8value_t> = arguments
            .iter()
            .map(|v| v.clone().inner.into_cef())
            .collect();

        unsafe {
            exec(
                self.inner.get_mut(),
                context.clone().inner.into_cef(),
                this,
                args.len(),
                args.as_ptr(),
            );
        }
    }

    pub fn set_value_by_key(&self, key: &CefString, value: &V8Value) {
        self.inner.set_value_bykey.map(|set_val| unsafe {
            set_val(
                self.inner.get_mut(),
                key.as_cef_string(),
                value.clone().inner.into_cef(),
                cef_sys::cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_NONE,
            );
        });
    }
}

pub fn register_extension<T: V8Handler>(
    extension_name: CefString, javascript_code: CefString, handler: Option<Arc<T>>,
) {
    let ptr = handler
        .map(|handler| crate::rust_to_c::v8handler::wrap(handler))
        .unwrap_or(std::ptr::null_mut());

    unsafe {
        cef_sys::cef_register_extension(
            extension_name.as_cef_string(),
            javascript_code.as_cef_string(),
            ptr,
        );
    }
}
