use crate::rust_to_c::Wrapper;
use crate::task::Task;
use cef_sys::cef_task_t;
use std::sync::Arc;

extern "stdcall" fn execute<I: Task>(this: *mut cef_task_t) {
    let obj: &mut Wrapper<_, I> = Wrapper::unwrap(this);
    obj.interface.execute();
}

pub fn wrap<T: Task>(app: Arc<T>) -> *mut cef_task_t {
    let mut object: cef_task_t = unsafe { std::mem::zeroed() };

    object.execute = Some(execute::<T>);

    let wrapper = Wrapper::new(object, app);

    Box::into_raw(Box::new(wrapper)) as *mut _
}
