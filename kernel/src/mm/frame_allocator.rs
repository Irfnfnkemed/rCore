use alloc::vec::Vec;
use core::borrow::BorrowMut;

use lazy_static::lazy_static;

use crate::mm::address::{PhysAddr, PhysPageNum};
use crate::sync::safe_cell_single;
use crate::sync::safe_cell_single::SafeCellSingle;

pub const MEMORY_END: usize = 0x8800_0000;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

pub struct StackFrameAllocator {
    // free frames which have never been used
    beg: usize,
    end: usize,
    // recycle frames
    recycled: Vec<usize>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            beg: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.beg == self.end {
                None
            } else {
                self.beg += 1;
                Some((self.beg - 1).into())
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.into();
        if ppn >= self.beg ||
            self.recycled.iter().find(|&v| { *v == ppn }).is_some() {
            panic!("[Kernel]: Frame ppn={:#x} has not been allocated!", ppn);
        }
        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, _beg: PhysPageNum, _end: PhysPageNum) {
        self.beg = _beg.0;
        self.end = _end.0;
    }
}

type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: SafeCellSingle<FrameAllocatorImpl> = unsafe {
        SafeCellSingle::new(FrameAllocatorImpl::new())
    };
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array();// page cleaning
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR
        .borrow_exclusive()
        .init(PhysAddr::from(ekernel as usize).ceil(), PhysAddr::from(MEMORY_END).floor());
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .borrow_exclusive()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR
        .borrow_exclusive()
        .dealloc(ppn);
}
