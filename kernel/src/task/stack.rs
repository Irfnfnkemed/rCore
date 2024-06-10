use crate::mm::address::{VirtAddr, VirtPageNum};
use crate::mm::area::MapPermission;
use crate::mm::memory_set::KERNEL_SPACE;
use crate::task::pid::PidHandle;

pub const TRAMPOLINE: usize = usize::MAX - 0x1000 + 1;
pub const PAGE_SIZE: usize = 0x1000;
pub const KERNEL_STACK_SIZE: usize = 2 * PAGE_SIZE;

pub struct KernelStack {
    pid: usize,
}


impl KernelStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_top, kernel_stack_bottom) = KernelStack::get_stack_pos(pid);
        KERNEL_SPACE
            .borrow_exclusive()
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MapPermission::R | MapPermission::W,
            );
        KernelStack {
            pid: pid_handle.0,
        }
    }

    pub fn get_stack_pos(pid: usize) -> (usize, usize) {
        let top = TRAMPOLINE - (KERNEL_STACK_SIZE + PAGE_SIZE) * (pid + 1) - PAGE_SIZE;
        let bottom = top - KERNEL_STACK_SIZE;
        (top, bottom)
    }

    pub fn get_top(&self) -> usize {
        TRAMPOLINE - (KERNEL_STACK_SIZE + PAGE_SIZE) * (self.pid + 1) - PAGE_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (_, kernel_stack_bottom) = KernelStack::get_stack_pos(self.pid);
        KERNEL_SPACE
            .borrow_exclusive()
            .remove_framed_area(VirtPageNum::from(kernel_stack_bottom));
    }
}