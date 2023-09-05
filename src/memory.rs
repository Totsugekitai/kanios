use crate::types::PhysAddr;
use core::{arch::global_asm, ptr};

pub const PAGE_SIZE: u64 = 0x1000;

extern "C" {
    static __free_ram: u8;
    static __free_ram_end: u8;
    static mut next_paddr: PhysAddr;
}

global_asm!(
    r#"
.section ".data"
.global next_paddr
next_paddr:
.quad __free_ram
    "#
);

pub unsafe fn alloc_pages(n: u64) -> PhysAddr {
    let paddr = next_paddr;
    next_paddr += PhysAddr(n * PAGE_SIZE);

    if next_paddr > PhysAddr(ptr::addr_of!(__free_ram_end) as u64) {
        panic!("out of memory");
    }

    ptr::write_bytes(paddr.to_u64() as *mut u8, 0, (n * PAGE_SIZE) as usize);
    paddr
}
