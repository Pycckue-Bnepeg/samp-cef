use cef_sys::cef_base_ref_counted_t;

use std::marker::PhantomData;
use std::sync::atomic::{self, AtomicUsize, Ordering};
use std::sync::Arc;

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
    interface: Arc<I>,
    ref_count: AtomicUsize,
    marker: PhantomData<T>,
}

impl<T, I> Wrapper<T, I> {
    pub fn new(mut cef_object: T, interface: Arc<I>) -> Wrapper<T, I> {
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
extern "stdcall" fn add_ref<T, I>(this: *mut cef_base_ref_counted_t) {
    let obj: &mut Wrapper<T, I> = Wrapper::<T, I>::unwrap(this as *mut T);

    obj.ref_count.fetch_add(1, Ordering::Relaxed);
}

#[inline(never)]
extern "stdcall" fn has_one_ref<T, I>(this: *mut cef_base_ref_counted_t) -> i32 {
    let obj: &mut Wrapper<T, I> = Wrapper::unwrap(this as *mut T);

    if obj.ref_count.load(Ordering::Relaxed) == 1 {
        1
    } else {
        0
    }
}

#[inline(never)]
extern "stdcall" fn has_at_least_one_ref<T, I>(this: *mut cef_base_ref_counted_t) -> i32 {
    let obj: &mut Wrapper<T, I> = Wrapper::unwrap(this as *mut T);

    if obj.ref_count.load(Ordering::Relaxed) >= 1 {
        1
    } else {
        0
    }
}

#[inline(never)]
pub extern "stdcall" fn release<T, I>(this: *mut cef_base_ref_counted_t) -> i32 {
    let obj: &mut Wrapper<T, I> = Wrapper::unwrap(this as *mut T);

    if obj.ref_count.fetch_sub(1, Ordering::Release) != 1 {
        0
    } else {
        atomic::fence(Ordering::Acquire);

        let _ = unsafe { Box::from_raw(this as *mut Wrapper<T, I>) };

        1
    }
}
