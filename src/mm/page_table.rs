use alloc::vec;
use alloc::vec::Vec;

use bitflags::*;

use crate::mm::address::{PhysPageNum, VirtPageNum};
use crate::mm::frame_allocator::{frame_alloc, FrameTracker};

const VPN_PTE_BITS: usize = 9;

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V).bits != 0
    }

    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R).bits != 0
    }

    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W).bits != 0
    }

    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X).bits != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.readable() || self.writable() || self.executable()
    }
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn).unwrap();
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        *pte = PageTableEntry::empty();
    }


    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let mut index = [0usize; 3];
        for i in 0..3 {
            index[i] = (vpn.0 >> (VPN_PTE_BITS * (2 - i))) & ((1 << VPN_PTE_BITS) - 1);
        }
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            println!("!");
            let pte = &mut ppn.get_pte_array()[index[i]];
            if !pte.is_valid() {
                println!("?{:X}", pte.bits);
                return None;
            }
            if i == 2 {
                return if pte.is_leaf() { Some(pte) } else { None }; // leaf page is (in)valid
            } else {
                if pte.is_leaf() {
                    return None; // TODO: huge page
                } else {
                    ppn = pte.ppn();
                }
            }
        }
        return None;
    }

    fn create_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let mut index = [0usize; 3];
        for i in 0..3 {
            index[i] = (vpn.0 >> (VPN_PTE_BITS * (2 - i))) & ((1 << VPN_PTE_BITS) - 1);
        }
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[index[i]];
            if i == 2 {
                return if pte.is_valid() {
                    None // the vpn has been mapped before
                } else {
                    Some(pte)
                };
            } else {
                if !pte.is_valid() {
                    let frame = frame_alloc().unwrap();
                    *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                    self.frames.push(frame);
                } else if pte.is_leaf() {
                    println!("b");
                    return None; // TODO: huge page
                }
                ppn = pte.ppn();
            }
        }
        return None;
    }

    pub fn token(&self) -> usize {
        8usize << 60 | (self.root_ppn.0)
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
}
