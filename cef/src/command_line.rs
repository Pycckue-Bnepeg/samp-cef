use cef_sys::cef_command_line_t;

use crate::ref_counted::RefGuard;
use crate::types::string::CefString;

pub struct CommandLine {
    inner: RefGuard<cef_command_line_t>,
}

impl CommandLine {
    pub(crate) fn from_raw(raw: *mut cef_command_line_t) -> CommandLine {
        CommandLine {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn append_switch_with_value(&self, name: &str, value: &str) {
        let name = CefString::new(name);
        let value = CefString::new(value);
        let func = self.inner.append_switch_with_value.unwrap();

        unsafe {
            func(
                self.inner.get_mut(),
                name.as_cef_string(),
                value.as_cef_string(),
            );
        }
    }

    pub fn append_switch(&self, switch: &str) {
        let switch = CefString::new(switch);
        let func = self.inner.append_switch.unwrap();

        unsafe {
            func(self.inner.get_mut(), switch.as_cef_string());
        }
    }

    pub fn append_argument(&self, arg: &str) {
        let arg = CefString::new(arg);
        let func = self.inner.append_argument.unwrap();

        unsafe {
            func(self.inner.get_mut(), arg.as_cef_string());
        }
    }
}
