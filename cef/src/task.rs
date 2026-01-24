use crate::ref_counted::RefGuard;
use cef_sys::cef_task_runner_t;
pub trait Task {
    fn execute(&self);
}

struct ClosureTask {
    closure: Box<dyn Fn() + 'static + Send + Sync>,
}

impl Task for ClosureTask {
    fn execute(&self) {
        (self.closure)();
    }
}

#[derive(Clone)]
pub struct TaskRunner {
    inner: RefGuard<cef_task_runner_t>,
}

impl TaskRunner {
    #[allow(dead_code)]
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
        let task = ClosureTask { closure: func };
        let task = crate::rust_to_c::task::wrap(task);

        let post_task = self.inner.post_task.unwrap();
        unsafe {
            post_task(self.inner.get_mut(), task);
        }
    }
}
