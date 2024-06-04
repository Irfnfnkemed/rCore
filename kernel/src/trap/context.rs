use riscv::register::sstatus;
use riscv::register::sstatus::{SPP, Sstatus};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

impl TrapContext {
    pub fn init_context(entry: usize, sp: usize, kernel_satp: usize,
                            kernel_sp: usize, trap_handler: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus: sstatus,
            sepc: entry,
            kernel_satp: kernel_satp,
            kernel_sp: kernel_sp,
            trap_handler: trap_handler,
        };
        cx.x[2] = sp;
        cx
    }
}

