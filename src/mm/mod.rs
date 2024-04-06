pub mod buddy;
mod address;
mod page_table;
pub mod frame_allocator;

pub fn init_heap() {
    buddy::init_heap();
}
