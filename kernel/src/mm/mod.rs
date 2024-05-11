use crate::mm::memory_set::{KERNEL_SPACE, remap_test};

pub mod buddy;
mod address;
mod page_table;
pub mod frame_allocator;
mod memory_set;
mod area;

pub fn init_mm() {
    buddy::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.borrow_exclusive().activate();
    remap_test();
}
