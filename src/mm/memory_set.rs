use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;

use lazy_static::lazy_static;
use riscv::register::satp;

use crate::mm::address::{PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::area::{MapArea, MapPermission, MapType};
use crate::mm::frame_allocator::{frame_alloc, FrameTracker, MEMORY_END};
use crate::mm::page_table::{PageTable, PTEFlags};
use crate::sync::safe_cell_single::SafeCellSingle;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}


pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}


impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            // map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    pub fn insert_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) {
        self.push(MapArea::new(start_va, end_va, MapType::Framed, permission), None);
    }

    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        // TODO:memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
        println!("mapping .text section");
        memory_set.push(MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::X,
        ), None);
        println!("mapping .rodata section");
        memory_set.push(MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,
        ), None);
        println!("mapping .data section");
        memory_set.push(MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        println!("mapping .bss section");
        memory_set.push(MapArea::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        println!("mapping physical memory");
        memory_set.push(MapArea::new(
            (ekernel as usize).into(),
            MEMORY_END.into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        memory_set
    }
    //pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize);

    pub fn activate(&self) {
        unsafe {
            satp::write(self.page_table.token());
            asm!("sfence.vma");
        }
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<SafeCellSingle<MemorySet>> = Arc::new(unsafe {
        SafeCellSingle::new(MemorySet::new_kernel()
    )});
}

pub fn remap_test() {
    // let mut kernel_space = KERNEL_SPACE.lock();
    // let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    // let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    // let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    // assert_eq!(
    //     kernel_space.page_table.translate(mid_text.floor()).unwrap().writable(),
    //     false
    // );
    // assert_eq!(
    //     kernel_space.page_table.translate(mid_rodata.floor()).unwrap().writable(),
    //     false,
    // );
    // assert_eq!(
    //     kernel_space.page_table.translate(mid_data.floor()).unwrap().executable(),
    //     false,
    // );
    println!("remap_test passed!");
}