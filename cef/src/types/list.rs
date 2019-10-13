use crate::ref_counted::RefGuard;
use crate::types::string::CefString;
use cef_sys::cef_list_value_t;

pub struct List {
    inner: RefGuard<cef_list_value_t>,
}

impl List {
    pub(crate) fn from_raw(raw: *mut cef_list_value_t) -> List {
        List {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn len(&self) -> usize {
        let len = self.inner.get_size.unwrap();
        unsafe { len(self.inner.get_mut()) }
    }

    pub fn get_type(&self, index: usize) -> ValueType {
        let ty = self.inner.get_type.unwrap();
        let ty = unsafe { ty(self.inner.get_mut(), index) };

        ValueType::from(ty)
    }

    pub fn string(&self, index: usize) -> CefString {
        self.inner
            .get_string
            .map(|get| unsafe { get(self.inner.get_mut(), index) })
            .map(|raw| CefString::from(raw))
            .unwrap_or_else(|| CefString::new_null())
    }

    pub fn set_string(&self, index: usize, string: &CefString) {
        self.inner
            .set_string
            .map(|set| unsafe { set(self.inner.get_mut(), index, string.as_cef_string()) });
    }

    pub fn bool(&self, index: usize) -> bool {
        self.inner
            .get_bool
            .map(|get| unsafe { get(self.inner.get_mut(), index) })
            .map(|raw| raw == 1)
            .unwrap_or_else(|| false)
    }

    pub fn set_bool(&self, index: usize, value: bool) {
        self.inner
            .set_bool
            .map(|set| unsafe { set(self.inner.get_mut(), index, if value { 1 } else { 0 }) });
    }

    pub fn integer(&self, index: usize) -> i32 {
        self.inner
            .get_int
            .map(|get| unsafe { get(self.inner.get_mut(), index) })
            .unwrap_or(0)
    }

    pub fn set_integer(&self, index: usize, value: i32) {
        self.inner
            .set_int
            .map(|set| unsafe { set(self.inner.get_mut(), index, value) });
    }

    pub fn double(&self, index: usize) -> f64 {
        self.inner
            .get_double
            .map(|get| unsafe { get(self.inner.get_mut(), index) })
            .unwrap_or(0.0)
    }

    pub fn set_double(&self, index: usize, value: f64) {
        self.inner
            .set_double
            .map(|set| unsafe { set(self.inner.get_mut(), index, value) });
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Invalid,
    Null,
    Bool,
    Integer,
    Double,
    String,
    Binary,
    Dictionary,
    List,
}

use cef_sys::cef_value_type_t as cvtype;

impl From<cvtype::Type> for ValueType {
    fn from(value: cvtype::Type) -> ValueType {
        use cvtype::*;
        use ValueType::*;

        match value {
            VTYPE_NULL => Null,
            VTYPE_BOOL => Bool,
            VTYPE_INT => Integer,
            VTYPE_DOUBLE => Double,
            VTYPE_STRING => String,
            VTYPE_BINARY => Binary,
            VTYPE_DICTIONARY => Dictionary,
            VTYPE_LIST => List,
            _ => Invalid,
        }
    }
}
