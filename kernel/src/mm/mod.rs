use crate::mm::memory_set::{KERNEL_SPACE, remap_test};

pub mod buddy;
pub(crate) mod address;
pub mod page_table;
pub mod frame_allocator;
pub mod memory_set;
pub mod area;

pub fn init_mm() {
    buddy::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.borrow_exclusive().activate();
    remap_test();
}
