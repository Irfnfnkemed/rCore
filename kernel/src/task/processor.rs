use alloc::sync::Arc;

use lazy_static::lazy_static;

use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::context::TaskContext;
use crate::task::manager::fetch_task;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::trap::context::TrapContext;

pub struct Processor {
    //The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,
    //The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            current: None,
            idle_task_cx: TaskContext::new_zero(),
        }
    }

    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
}

lazy_static! {
    pub static ref PROCESSOR: SafeCellSingle<Processor> = unsafe { SafeCellSingle::new(Processor::new()) };
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    PROCESSOR.borrow_exclusive().current.as_ref().map(Arc::clone).unwrap()
        .borrow_exclusive_inner().get_trap_cx_ref()
}

pub fn current_user_token() -> usize {
    PROCESSOR.borrow_exclusive().current.as_ref().map(Arc::clone).unwrap()
        .borrow_exclusive_inner().get_user_token()
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.borrow_exclusive().current.take()
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.borrow_exclusive();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            let mut task_inner = task.borrow_exclusive_inner();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner); // since sp will switch to other task after __switch, it's necessary to drop explicitly
            processor.current = Some(task);
            drop(processor);
            unsafe {
                __switch(
                    idle_task_cx_ptr,
                    next_task_cx_ptr,
                );
            }
        }
    }
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.borrow_exclusive();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

