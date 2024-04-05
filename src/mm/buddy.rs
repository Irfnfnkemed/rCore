use alloc::alloc::alloc;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::arch::asm;
use core::borrow::BorrowMut;
use core::cell::{RefCell, RefMut};
use core::cmp::{max, min};
use core::mem::size_of;
use core::ptr::null_mut;

pub const KERNEL_HEAP_SIZE: usize = 0x40_0000;
pub const BLOCK_UNIT_SIZE: usize = 0x1000;
pub const BLOCK_LEVEL: usize = 11;
pub const TABLE_SIZE: usize = 1024;

#[derive(Copy, Clone)]
struct LinkNode {
    // prev index
    prev: i16,
    // next index
    next: i16,
    // block level
    level: i16,
    // is(not) free
    free: bool,
}


pub struct Allocator {
    // head/tail address of the link of different block size level
    free_head: [i16; BLOCK_LEVEL],
    free_tail: [i16; BLOCK_LEVEL],
    // store the nodes in the link
    link_table: [LinkNode; TABLE_SIZE],
    // begin address of the heap
    heap_beg_addr: usize,
}

pub struct AllocatorWrap {
    allocator: usize,
}

impl Allocator {
    pub const fn empty() -> Self {
        let free_head = [-2i16; BLOCK_LEVEL];
        let free_tail = [-1i16; BLOCK_LEVEL];
        let link_table = [LinkNode { prev: 0, next: 0, level: 0, free: false }; TABLE_SIZE];
        let mut allocator = Allocator {
            free_head: free_head,
            free_tail: free_tail,
            link_table: link_table,
            heap_beg_addr: 0,
        };
        allocator
    }

    pub fn init(&mut self, _heap_beg_addr: usize) {
        self.heap_beg_addr = _heap_beg_addr;
        self.push(_heap_beg_addr, BLOCK_LEVEL - 1);
    }


    fn merge(&mut self, addr: usize, level: usize) {
        if level >= BLOCK_LEVEL {
            panic!("[kernel]: Unknown error when alloca.");
        }
        if level == BLOCK_LEVEL - 1 {
            self.push(addr, level);
            return;
        }
        let current_size: usize = (1 << level) * BLOCK_UNIT_SIZE;
        let buddy_addr: usize = if (addr - self.heap_beg_addr) % (current_size << 1) == 0 {
            addr + current_size
        } else {
            addr - current_size
        };
        let buddy_index = self.get_link_index(buddy_addr);
        if self.link_table[buddy_index].free && self.link_table[buddy_index].level == level as i16 {
            self.pop(buddy_index);
            self.merge(min(addr, buddy_addr), level + 1);
        } else {
            self.push(addr, level);
            return;// buddy isn't free
        }
    }

    fn split(&mut self, level: usize) -> *mut u8 {
        let mut now_level: usize = level;
        let mut index: usize = 0;
        while now_level < BLOCK_LEVEL {
            if self.free_head[now_level] != -2 {
                index = self.free_head[now_level] as usize;
                self.pop(index);
                break;
            }
            now_level += 1;
        }
        if now_level == BLOCK_LEVEL {
            return null_mut();
        }
        while now_level > level {
            now_level -= 1;
            self.push(self.get_address(index as i16), now_level);
            index += (1 << now_level);
        }
        return self.get_address(index as i16) as *mut u8;
    }


    fn push(&mut self, addr: usize, level: usize) {
        let index = self.get_link_index(addr);
        self.link_table[index].prev = -1;
        self.link_table[index].next = self.free_head[level];
        self.link_table[index].free = true;
        self.link_table[index].level = level as i16;
        if self.free_head[level] == -2 {
            self.free_tail[level] = index as i16;
        } else {
            self.link_table[self.free_head[level] as usize].prev = index as i16;
        }
        self.free_head[level] = index as i16;
    }

    fn pop(&mut self, index: usize) {
        let level = self.link_table[index].level as usize;
        self.link_table[index].free = false;
        if self.link_table[index].next == -2 {
            self.free_tail[level] = self.link_table[index].prev;
        } else {
            self.link_table[self.link_table[index].next as usize].prev = self.link_table[index].prev;
        }
        if self.link_table[index].prev == -1 {
            self.free_head[level] = self.link_table[index].next;
        } else {
            self.link_table[self.link_table[index].prev as usize].next = self.link_table[index].next;
        }
    }

    fn get_address(&self, index: i16) -> usize {
        self.heap_beg_addr + index as usize * BLOCK_UNIT_SIZE
    }

    fn get_link_index(&self, addr: usize) -> usize {
        (addr - self.heap_beg_addr) / BLOCK_UNIT_SIZE
    }
}

impl AllocatorWrap {
    pub const fn empty() -> Self {
        AllocatorWrap { allocator: 0 }
    }

    pub unsafe fn init(&mut self, allocator: &mut Allocator) {
        self.allocator = allocator as *mut Allocator as usize;
    }
}

unsafe impl GlobalAlloc for AllocatorWrap {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        let size = max(max(_layout.size().next_power_of_two(), BLOCK_UNIT_SIZE), _layout.align());
        let alloctor = &mut *(self.allocator as *mut Allocator);
        alloctor.split((size / BLOCK_UNIT_SIZE).trailing_zeros() as usize)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let alloctor = &mut *(self.allocator as *mut Allocator);
        if alloctor.heap_beg_addr > _ptr as usize ||
            alloctor.heap_beg_addr + KERNEL_HEAP_SIZE <= _ptr as usize {
            panic!("[kernel]: Invalid address to dealloc.");
        }
        let size = max(max(_layout.size().next_power_of_two(), BLOCK_UNIT_SIZE), _layout.align());
        alloctor.merge(_ptr as usize, (size / BLOCK_UNIT_SIZE).trailing_zeros() as usize);
    }
}

pub fn init_heap() {
    unsafe {
        INNER_ALLOCATOR.init(HEAP_SPACE.as_ptr() as usize);
        HEAP_ALLOCATOR.allocator = &mut INNER_ALLOCATOR as *mut Allocator as usize;
    }
}

pub(crate) static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

static mut INNER_ALLOCATOR: Allocator = Allocator::empty();

#[global_allocator]
static mut HEAP_ALLOCATOR: AllocatorWrap = AllocatorWrap::empty();
