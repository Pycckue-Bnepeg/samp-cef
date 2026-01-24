use std::cell::UnsafeCell;

pub struct StaticCell<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Sync for StaticCell<T> {}

impl<T> StaticCell<T> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    /// # Safety
    /// Caller must ensure exclusive access to the stored value.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut(&self) -> Option<&mut T> {
        unsafe { (*self.0.get()).as_mut() }
    }

    /// # Safety
    /// Caller must ensure exclusive access to the stored value.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn set(&self, value: T) -> &mut T {
        unsafe {
            *self.0.get() = Some(value);
            (*self.0.get()).as_mut().unwrap()
        }
    }

    /// # Safety
    /// Caller must ensure exclusive access to the stored value.
    pub unsafe fn take(&self) -> Option<T> {
        unsafe { (*self.0.get()).take() }
    }
}

impl<T> Default for StaticCell<T> {
    fn default() -> Self {
        Self::new()
    }
}
