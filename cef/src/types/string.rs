use cef_sys::cef_string_userfree_t;
pub use cef_sys::{
    cef_string_t, cef_string_utf16_clear, cef_string_utf16_to_utf8, cef_string_utf8_to_utf16,
};

#[repr(C)]
pub struct CefString {
    inner: *mut cef_string_t,
    owned: bool,
}

impl CefString {
    pub fn new(text: &str) -> CefString {
        let mut string = Self::new_null();

        unsafe {
            cef_string_utf8_to_utf16(text.as_ptr() as *const _, text.len(), string.inner);
        }

        string
    }

    pub fn new_null() -> CefString {
        let string = unsafe { Box::into_raw(Box::new(std::mem::zeroed())) };

        CefString {
            inner: string,
            owned: true,
        }
    }

    pub fn to_cef_string(&self) -> cef_string_t {
        let inner = unsafe { &*self.inner };
        cef_string_t {
            str: inner.str,
            length: inner.length,
            dtor: inner.dtor,
        }
    }

    pub fn as_cef_string(&self) -> &cef_string_t {
        unsafe { &*self.inner }
    }

    pub fn to_string(&self) -> String {
        unsafe {
            let utf16 = &*self.inner;
            let bytes = std::slice::from_raw_parts(utf16.str, utf16.length);
            String::from_utf16_lossy(bytes)
        }
    }
}

impl Drop for CefString {
    fn drop(&mut self) {
        unsafe {
            if (*self.inner).str.is_null() || !self.owned {
                return;
            }
        }

        unsafe {
            cef_string_utf16_clear(self.inner);
        }
    }
}

impl From<*const cef_string_t> for CefString {
    fn from(string: *const cef_string_t) -> CefString {
        CefString {
            inner: string as *mut _,
            owned: false,
        }
    }
}

impl From<cef_string_userfree_t> for CefString {
    fn from(string: cef_string_userfree_t) -> CefString {
        let mut cefstr = Self::new_null();

        unsafe {
            let src = &mut *string;
            let dst = &mut *cefstr.inner;

            dst.length = src.length;
            dst.str = src.str;
            dst.dtor = src.dtor;

            *src = std::mem::zeroed();

            cef_sys::cef_string_userfree_utf16_free(string);
        }

        cefstr
    }
}
