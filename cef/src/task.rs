use crate::ref_counted::RefGuard;
use cef_sys::cef_task_runner_t;
use std::sync::Arc;

pub trait Task {
    fn execute(self: &Arc<Self>);
}

struct ClosureTask {
    closure: Box<dyn Fn() + 'static + Send + Sync>,
}

impl Task for ClosureTask {
    fn execute(self: &Arc<Self>) {
        (self.closure)();
    }
}

#[derive(Clone)]
pub struct TaskRunner {
    inner: RefGuard<cef_task_runner_t>,
}

impl TaskRunner {
    pub(crate) fn from_raw(raw: *mut cef_task_runner_t) -> TaskRunner {
        TaskRunner {
            inner: RefGuard::from_raw(raw),
        }
    }

    pub fn post_task<F>(&self, func: F)
    where
        F: Fn() + 'static + Send + Sync,
    {
        let func = Box::new(func);
        let task = Arc::new(ClosureTask { closure: func });
        let task = crate::rust_to_c::task::wrap(task);

        let post_task = self.inner.post_task.unwrap();
        unsafe {
            post_task(self.inner.get_mut(), task);
        }
    }
}
