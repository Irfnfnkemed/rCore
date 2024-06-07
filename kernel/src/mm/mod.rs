use crate::mm::buddy::{Allocator, AllocatorWrap, KERNEL_HEAP_SIZE};
use crate::mm::memory_set::{KERNEL_SPACE, remap_test};

pub mod buddy;
pub mod address;
pub mod page_table;
pub mod frame_allocator;
pub mod memory_set;
pub mod area;


pub fn init_heap() {
    unsafe {
        INNER_ALLOCATOR.init(HEAP_SPACE.as_ptr() as usize);
        HEAP_ALLOCATOR.allocator = &mut INNER_ALLOCATOR as *mut Allocator as usize;
    }
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

static mut INNER_ALLOCATOR: Allocator = Allocator::empty();

#[global_allocator]
static mut HEAP_ALLOCATOR: AllocatorWrap = AllocatorWrap::empty();


pub fn init_mm() {
    init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.borrow_exclusive().activate();
    remap_test();
}
