use core::cell::{RefCell, RefMut};

pub struct SafeCellSingle<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for SafeCellSingle<T> {}

impl<T> SafeCellSingle<T> {
    pub unsafe fn new(value: T) -> Self {
        Self { inner: RefCell::new(value) }
    }

    pub fn borrow_exclusive(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}


