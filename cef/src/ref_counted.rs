use std::marker::PhantomData;
use std::ops::Deref;

use cef_sys::cef_base_ref_counted_t;

pub trait RefCounted {
    fn add_ref(&self);
    fn has_one_ref(&self) -> bool;
    fn has_at_least_one_ref(&self) -> bool;
    fn release(&self) -> bool;
    fn as_base(&self) -> &cef_base_ref_counted_t;
}

// types from library (cef_browser_t etc.)
// smart pointer
pub struct RefGuard<T: RefCounted> {
    object: *mut T,
    marker: PhantomData<T>,
}

impl<T: RefCounted> RefGuard<T> {
    #[inline]
    pub(crate) fn from_raw(ptr: *mut T) -> RefGuard<T> {
        RefGuard {
            object: ptr,
            marker: PhantomData,
        }
    }

    pub(crate) fn from_raw_add_ref(ptr: *mut T) -> RefGuard<T> {
        let guard = RefGuard {
            object: ptr,
            marker: PhantomData,
        };

        guard.add_ref();

        guard
    }

    /// clone value
    pub fn into_cef(self) -> *mut T {
        let ptr = unsafe { self.get_mut() };
        std::mem::forget(self);
        ptr
    }

    pub unsafe fn get_mut(&self) -> *mut T {
        self.object
    }
}

impl<T: RefCounted> Clone for RefGuard<T> {
    fn clone(&self) -> RefGuard<T> {
        self.add_ref();

        RefGuard {
            object: self.object,
            marker: PhantomData,
        }
    }
}

impl<T: RefCounted> Deref for RefGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.object }
    }
}

impl<T: RefCounted> Drop for RefGuard<T> {
    fn drop(&mut self) {
        self.release();
    }
}

impl RefCounted for cef_base_ref_counted_t {
    fn add_ref(&self) {
        if let Some(add_ref) = self.add_ref {
            unsafe {
                add_ref(self as *const _ as *mut _);
            }
        }
    }

    fn has_one_ref(&self) -> bool {
        if let Some(has_one_ref) = self.has_one_ref {
            let result = unsafe { has_one_ref(self as *const _ as *mut _) };

            return result == 1;
        }

        false
    }

    fn has_at_least_one_ref(&self) -> bool {
        if let Some(has_at_least_one_ref) = self.has_at_least_one_ref {
            let result = unsafe { has_at_least_one_ref(self as *const _ as *mut _) };

            return result == 1;
        }

        false
    }

    fn release(&self) -> bool {
        if let Some(release) = self.release {
            let result = unsafe { release(self as *const _ as *mut _) };

            return result == 1;
        }

        false
    }

    fn as_base(&self) -> &Self {
        self
    }
}

unsafe impl<T: RefCounted> Send for RefGuard<T> {}

macro_rules! impl_rc {
    ($name:ident) => {
        impl RefCounted for cef_sys::$name {
            fn add_ref(&self) {
                self.as_base().add_ref();
            }

            fn has_one_ref(&self) -> bool {
                self.as_base().has_one_ref()
            }

            fn has_at_least_one_ref(&self) -> bool {
                self.as_base().has_at_least_one_ref()
            }

            fn release(&self) -> bool {
                self.as_base().release()
            }

            fn as_base(&self) -> &cef_base_ref_counted_t {
                unsafe { &*(self as *const _ as *const cef_base_ref_counted_t) }
            }
        }
    };
}

impl_rc!(cef_browser_t);
impl_rc!(cef_frame_t);
impl_rc!(cef_browser_host_t);
impl_rc!(cef_v8context_t);
impl_rc!(cef_v8value_t);
impl_rc!(cef_process_message_t);
impl_rc!(cef_list_value_t);
impl_rc!(cef_task_t);
impl_rc!(cef_task_runner_t);
impl_rc!(cef_context_menu_params_t);
impl_rc!(cef_menu_model_t);
impl_rc!(cef_command_line_t);
