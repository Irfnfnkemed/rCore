use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

use crate::mm::address::{PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::memory_set::{KERNEL_SPACE, MemorySet, TRAP_CONTEXT};
use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::context;
use crate::task::context::TaskContext;
use crate::task::pid::{pid_alloc, PidHandle};
use crate::task::stack::KernelStack;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

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

impl TaskControlBlockInner {
    pub fn get_trap_cx_ref(&self) -> &'static mut TrapContext {
        unsafe { (self.trap_cx_ppn.0 as *mut TrapContext).as_mut().unwrap() }
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}


impl TaskControlBlock {
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtPageNum::from(TRAP_CONTEXT)).unwrap().ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let (kernel_stack_top, _) = KernelStack::get_stack_pos(pid_handle.0);
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack: kernel_stack,
            inner: unsafe {
                SafeCellSingle::new(TaskControlBlockInner {
                    trap_cx_ppn: trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::new_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set: memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.borrow_exclusive_inner().get_trap_cx_ref();
        *trap_cx = TrapContext::init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.borrow_exclusive().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    pub fn borrow_exclusive_inner(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.borrow_exclusive()
    }
}
