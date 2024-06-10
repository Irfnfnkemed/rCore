use crate::trap::trap_return;

#[repr(C)]
pub struct TaskContext {
    ra: usize,
    pub sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn new_zero() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn new_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
