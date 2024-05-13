use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

use crate::mm::address::PhysPageNum;
use crate::mm::memory_set::MemorySet;
use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::context::TaskContext;
use crate::task::pid::PidHandle;
use crate::task::stack::KernelStack;

pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    inner: SafeCellSingle<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}


impl TaskControlBlock {
    pub fn borrow_exclusive_inner(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.borrow_exclusive()
    }
}
