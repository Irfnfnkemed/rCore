use alloc::sync::{Arc, Weak};

use lazy_static::lazy_static;

pub use manager::{add_task, is_fixed};
pub use processor::{current_task, current_trap_cx, current_user_token, run_tasks};

use crate::loader::get_app_data_by_name;
use crate::task::context::TaskContext;
use crate::task::manager::add_server;
use crate::task::processor::{schedule, take_current_task};
use crate::task::task::{TaskControlBlock, TaskStatus};

mod stack;
mod task;
pub mod manager;
mod processor;
mod switch;
mod context;
mod rand;

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new_proc_special(get_app_data_by_name("initproc").unwrap(), 0)
    );
}

lazy_static! {
    pub static ref MANAGER: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new_proc_special(get_app_data_by_name("manager").unwrap(), 1)
    );
}

pub fn init_proc() {
    add_task(INITPROC.clone());
    add_server(MANAGER.clone());
}


pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap(); // move curr-task
    let mut task_inner = task.borrow_exclusive_inner();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;// Change status to Ready
    drop(task_inner);
    add_task(task);
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    current_task().unwrap().exit(exit_code);
    take_current_task();// move curr-task
    let mut _unused = TaskContext::new_zero();
    schedule(&mut _unused as *mut _);
}