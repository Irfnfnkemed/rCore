use alloc::vec::Vec;

use lazy_static::lazy_static;

use crate::sync::safe_cell_single::SafeCellSingle;

pub struct PidHandle(pub usize);

pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.borrow_exclusive().dealloc(self.0);
    }
}

impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> PidHandle {
        let mut pid: usize = 0;
        if self.recycled.is_empty() {
            pid = self.current;
            self.current += 1;
        } else {
            pid = self.recycled.pop().unwrap();
        }
        PidHandle(pid)
    }

    pub fn dealloc(&mut self, pid: usize) {
        self.recycled.push(pid);
    }
}


lazy_static! {
    static ref PID_ALLOCATOR : SafeCellSingle<PidAllocator> = unsafe {
        SafeCellSingle::new(PidAllocator::new())
    };
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.borrow_exclusive().alloc()
}

pub fn pid_dealloc(pid: usize) {
    PID_ALLOCATOR.borrow_exclusive().dealloc(pid);
}