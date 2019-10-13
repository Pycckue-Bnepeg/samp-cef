use crate::ref_counted::RefGuard;
use crate::types::list::List;
use crate::types::string::CefString;
use cef_sys::cef_process_message_t;

pub struct ProcessMessage {
    inner: RefGuard<cef_process_message_t>,
}

impl ProcessMessage {
    pub(crate) fn from_raw(raw: *mut cef_process_message_t) -> ProcessMessage {
        ProcessMessage {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub(crate) fn into_cef(self) -> *mut cef_process_message_t {
        self.inner.into_cef()
    }

    pub fn create(name: &str) -> ProcessMessage {
        let name = CefString::new(name);
        let ptr = unsafe { cef_sys::cef_process_message_create(name.as_cef_string()) };

        Self::from_raw(ptr)
    }

    pub fn name(&self) -> CefString {
        self.inner
            .get_name
            .map(|name| unsafe { name(self.inner.get_mut()) })
            .map(|raw| CefString::from(raw))
            .unwrap_or_else(|| CefString::new_null())
    }

    pub fn is_valid(&self) -> bool {
        self.inner
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.inner.get_mut()) })
            .map(|int| int == 1)
            .unwrap_or(false)
    }

    pub fn argument_list(&self) -> List {
        let get_list = self.inner.get_argument_list.unwrap();
        let list = unsafe { get_list(self.inner.get_mut()) };

        List::from_raw(list)
    }
}
