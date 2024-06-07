use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

use crate::mm::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::memory_set::{KERNEL_SPACE, MemorySet, TRAP_CONTEXT};
use crate::mm::page_table::translated_refmut;
use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::context;
use crate::task::context::TaskContext;
use crate::task::pid::{pid_alloc, PidHandle};
use crate::task::stack::{KernelStack, TRAMPOLINE};
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

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        let pa: PhysAddr = self.trap_cx_ppn.into();
        unsafe { (pa.0 as *mut TrapContext).as_mut().unwrap() }
    }
}


impl TaskControlBlock {
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set.translate(VirtPageNum::from(TRAP_CONTEXT)).unwrap().ppn();
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

    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        let mut parent_inner = self.borrow_exclusive_inner();
        let memory_set = MemorySet::new_from_exist(&parent_inner.memory_set); // TODO: COW
        let trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let (kernel_stack_top, _) = KernelStack::get_stack_pos(pid_handle.0);
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack: kernel_stack,
            inner: unsafe {
                SafeCellSingle::new(TaskControlBlockInner {
                    trap_cx_ppn: trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::new_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set: memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                })
            },
        });
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx, which means child will return to User-mod
        let trap_cx = task_control_block.borrow_exclusive_inner().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        task_control_block
    }

    pub fn exec(&self, elf_data: &[u8]) {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let mut inner = self.borrow_exclusive_inner();
        inner.trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();
        inner.memory_set = memory_set; // replace mem_set
        inner.base_size = user_sp;
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.borrow_exclusive().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
    }

    pub fn waitpid(self: &Arc<TaskControlBlock>, pid: isize, exit_code_ptr: *mut i32) -> isize {
        let mut inner = self.borrow_exclusive_inner();
        if !inner.children.iter().
            any(|son| { pid == -1 || son.pid.0 == pid as usize }) {
            -1;
        }
        let pair = inner.children.iter().enumerate().
            find(|(_, son)| {
                (pid == -1 || son.pid.0 == pid as usize) &&
                    son.borrow_exclusive_inner().task_status == TaskStatus::Zombie
            });
        if let Some((index, _)) = pair {
            let mut child = inner.children.remove(index);
            assert_eq!(Arc::strong_count(&child), 1); // confirm, child-proc should be only owned here
            let found_pid = child.pid.0 as isize;
            let exit_code = child.borrow_exclusive_inner().exit_code;
            *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code; // write to the current user-space
            found_pid;
        }
        -2
    }
}
