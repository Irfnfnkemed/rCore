use alloc::vec::Vec;

use crate::mm::address::{PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::area::MapType::Framed;
use crate::mm::frame_allocator::{frame_alloc, FrameTracker};
use crate::mm::page_table::{PageTable, PTEFlags};

const PAGE_SIZE: usize = 0x1000;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MapArea {
    vpn_beg: VirtPageNum,
    vpn_end: VirtPageNum,
    data_frames: Vec<FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr,
               map_type: MapType, map_perm: MapPermission) -> Self {
        Self {
            vpn_beg: start_va.floor(),
            vpn_end: end_va.ceil(),
            data_frames: Vec::new(),
            map_type: map_type,
            map_perm: map_perm,
        }
    }

    pub fn new_from_exist(obj: &Self) -> Self {
        Self {
            vpn_beg: obj.vpn_beg,
            vpn_end: obj.vpn_end,
            data_frames: Vec::new(),
            map_type: obj.map_type,
            map_perm: obj.map_perm,
        }
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        let mut tmp = self.vpn_beg.0;
        while tmp < self.vpn_end.0 {
            let ppn: PhysPageNum;
            match self.map_type {
                MapType::Identical => {
                    ppn = PhysPageNum(tmp);
                }
                Framed => {
                    let frame = frame_alloc().unwrap();
                    ppn = frame.ppn;
                    self.data_frames.push(frame);
                }
            }
            let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
            page_table.map(VirtPageNum::from(tmp), ppn, pte_flags);
            tmp += 1;
        }
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        let mut tmp = self.vpn_beg.0;
        while tmp < self.vpn_end.0 {
            page_table.unmap(VirtPageNum::from(tmp));
            tmp += 1;
        }
    }

    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_beg;
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn).unwrap()
                .ppn().get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.next();
        }
    }

    pub fn get_beg_vpn(&self) -> VirtPageNum { self.vpn_beg }

    pub fn get_end_vpn(&self) -> VirtPageNum { self.vpn_end }
}
