use crate::mm::page_table::PageTableEntry;

const PAGE_SIZE_BITS: usize = 12;
const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;
const PAGE_SIZE: usize = 0x1000;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl PhysAddr {
    pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
    pub fn floor(&self) -> PhysPageNum { PhysPageNum(self.0 / PAGE_SIZE) }
    pub fn ceil(&self) -> PhysPageNum { PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}

impl VirtAddr {
    pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
    pub fn floor(&self) -> VirtPageNum { VirtPageNum(self.0 / PAGE_SIZE) }
    pub fn ceil(&self) -> VirtPageNum { VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
}

impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)
        }
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE)
        }
    }
}

impl VirtPageNum {
    pub fn next(&mut self) { self.0 += 1 }
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self { Self(v & ((1 << PA_WIDTH_SV39) - 1)) }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self { Self(v & ((1 << PPN_WIDTH_SV39) - 1)) }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self { Self(v & ((1 << VA_WIDTH_SV39) - 1)) }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self { Self(v & ((1 << VPN_WIDTH_SV39) - 1)) }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self { v.0 }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self { v.0 }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self { v.0 }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(phy_addr: PhysAddr) -> Self {
        assert_eq!(phy_addr.page_offset(), 0);
        phy_addr.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(phy_num: PhysPageNum) -> Self { Self(phy_num.0 << PAGE_SIZE_BITS) }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(vir_addr: VirtAddr) -> Self {
        assert_eq!(vir_addr.page_offset(), 0);
        vir_addr.floor()
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(vir_num: VirtPageNum) -> Self { Self(vir_num.0 << PAGE_SIZE_BITS) }
}




