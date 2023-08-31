use crate::memory::{alloc_pages, PAGE_SIZE};

use crate::types::{PhysAddr, VirtAddr};

pub const SATP_SV39: u64 = 8 << 60;
pub const PAGE_V: u64 = 1 << 0;
pub const PAGE_R: u64 = 1 << 1;
pub const PAGE_W: u64 = 1 << 2;
pub const PAGE_X: u64 = 1 << 3;
pub const PAGE_U: u64 = 1 << 4;

pub unsafe fn map_page(table2: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, flags: u64) {
    assert!(vaddr % PAGE_SIZE == 0);
    assert!(paddr % PAGE_SIZE == 0);

    let table2 = table2 as *mut u64;
    let vpn2 = ((vaddr >> 30) & 0b0001_1111_1111) as isize;
    if (*table2.offset(vpn2) & PAGE_V) == 0 {
        // 2段目のページテーブルが存在しないので作成する
        let pt_addr = alloc_pages(1);
        *table2.offset(vpn2) = ((pt_addr / PAGE_SIZE) << 10) | PAGE_V;
    }

    let table1 = ((*table2.offset(vpn2) << 2) & !0xfff) as *mut u64;
    let vpn1 = ((vaddr >> 21) & 0b0001_1111_1111) as isize;
    if (*table1.offset(vpn1) & PAGE_V) == 0 {
        // 3段目のページテーブルが存在しないので作成する
        let pt_addr = alloc_pages(1);
        *table1.offset(vpn1) = ((pt_addr / PAGE_SIZE) << 10) | PAGE_V;
    }

    // 3段目のページテーブルにエントリを追加する
    let table0 = ((*table1.offset(vpn1) << 2) & !0xfff) as *mut u64;
    let vpn0 = ((vaddr >> 12) & 0b0001_1111_1111) as isize;
    *table0.offset(vpn0) = ((paddr / PAGE_SIZE) << 10) | flags | PAGE_V;
}
