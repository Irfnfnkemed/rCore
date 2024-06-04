use lazy_static::lazy_static;
pub use processor::{current_trap_cx, current_user_token};
use crate::task::context::TaskContext;

use crate::task::manager::add_task;
use crate::task::processor::{schedule, take_current_task};
use crate::task::task::TaskStatus;

mod pid;
mod stack;
mod task;
mod manager;
mod processor;
mod switch;
mod context;

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
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