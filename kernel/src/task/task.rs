use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

use crate::mm::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::frame_allocator::BUFFER_BEG;
use crate::mm::memory_set::{BUFFER, KERNEL_SPACE, MemorySet, PAGE_SIZE, TRAP_CONTEXT};
use crate::mm::page_table::translated_refmut;
use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::{context, suspend_current_and_run_next};
use crate::task::context::TaskContext;
use crate::task::manager::set_server;
use crate::task::stack::{KernelStack, TRAMPOLINE};
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

const NOP_REQUEST: usize = 0;
const FORK_REQUEST: usize = 1;
const EXIT_REQUEST: usize = 2;
const WAITPID_REQUEST: usize = 3;
const DONE_REQUEST: usize = 4;

pub struct TaskControlBlock {
    pub pid: usize,
    pub kernel_stack: KernelStack,
    inner: SafeCellSingle<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx_ref(&self) -> &'static mut TrapContext {
        let pa: PhysAddr = self.trap_cx_ppn.into();
        unsafe { (pa.0 as *mut TrapContext).as_mut().unwrap() }
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
    pub fn new_proc_special(elf_data: &[u8], pid: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();
        let kernel_stack = KernelStack::new(pid); //init_proc: pid=0
        let (kernel_stack_top, _) = KernelStack::get_stack_pos(pid);
        memory_set.map_buffer_user(pid);
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid,
            kernel_stack: kernel_stack,
            inner: unsafe {
                SafeCellSingle::new(TaskControlBlockInner {
                    trap_cx_ppn: trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::new_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set: memory_set,
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
        let mut memory_set = MemorySet::new_from_exist(&parent_inner.memory_set); // TODO: COW
        drop(parent_inner);
        let trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();
        let buffer_usize = unsafe {
            core::slice::from_raw_parts_mut((BUFFER_BEG + PAGE_SIZE) as *mut usize, PAGE_SIZE / 8)
        };
        buffer_usize[0] = FORK_REQUEST;
        buffer_usize[1] = self.pid;
        set_server(1);
        suspend_current_and_run_next();
        assert_eq!(buffer_usize[0], DONE_REQUEST); // confirm manager work correctly
        let pid = buffer_usize[1];
        let kernel_stack = KernelStack::new(pid);
        let (kernel_stack_top, _) = KernelStack::get_stack_pos(pid);
        memory_set.map_buffer_user(pid);
        let mut parent_inner = self.borrow_exclusive_inner();
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid,
            kernel_stack: kernel_stack,
            inner: unsafe {
                SafeCellSingle::new(TaskControlBlockInner {
                    trap_cx_ppn: trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::new_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set: memory_set,
                })
            },
        });
        // modify kernel_sp in trap_cx, which means child will return to User-mod
        let trap_cx = task_control_block.borrow_exclusive_inner().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        task_control_block
    }

    pub fn exec(&self, elf_data: &[u8]) {
        let (mut memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        memory_set.map_buffer_user(self.pid);
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
        let buffer_usize = unsafe {
            core::slice::from_raw_parts_mut((BUFFER_BEG + PAGE_SIZE) as *mut usize, PAGE_SIZE / 8)
        };
        buffer_usize[0] = WAITPID_REQUEST;
        buffer_usize[1] = self.pid;
        buffer_usize[2] = pid as usize;
        drop(inner);
        set_server(1);
        suspend_current_and_run_next();
        assert_eq!(buffer_usize[0], DONE_REQUEST); // confirm manager work correctly
        let mut inner = self.borrow_exclusive_inner();
        let ret = buffer_usize[1] as isize;
        if ret >= 0 {
            let exit_code = buffer_usize[2] as i32;
            *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code; // write to the current user-space
        }
        ret
    }

    pub fn exit(self: &Arc<TaskControlBlock>, exit_code: i32) {
        let mut task_inner = self.borrow_exclusive_inner();
        task_inner.task_status = TaskStatus::Zombie;
        drop(task_inner);
        let buffer_usize = unsafe {
            core::slice::from_raw_parts_mut((BUFFER_BEG + PAGE_SIZE) as *mut usize, PAGE_SIZE / 8)
        };
        buffer_usize[0] = EXIT_REQUEST;
        buffer_usize[1] = self.pid;
        buffer_usize[2] = exit_code as usize;
        set_server(1);
        suspend_current_and_run_next();
        let mut task_inner = self.borrow_exclusive_inner();
        task_inner.memory_set.recycle();
    }
}
