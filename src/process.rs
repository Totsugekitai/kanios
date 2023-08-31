use crate::{
    __free_ram_end, __kernel_base,
    memory::{alloc_pages, PAGE_SIZE},
    paging::SATP_SV39,
    types::{PhysAddr, VirtAddr},
    write_csr,
};
use core::{
    arch::{asm, global_asm},
    mem, ptr,
};

extern "C" {
    fn switch_context(prev_sp: *mut VirtAddr, next_sp: *const VirtAddr);
}

const PROCS_MAX: usize = 8;
const PROC_UNUSED: i64 = 0;
const PROC_RUNNABLE: i64 = 1;

#[repr(align(8))]
#[derive(Debug, Clone, Copy)]
pub struct Process {
    pub pid: i64,
    pub state: i64,
    pub sp: VirtAddr,
    pub page_table: PhysAddr,
    pub stack: [u8; 8192],
}

impl Process {
    const fn new() -> Self {
        Self {
            pid: 0,
            state: PROC_UNUSED,
            sp: 0,
            page_table: 0,
            stack: [0; 8192],
        }
    }

    pub fn create(pc: u64) -> *mut Process {
        let mut proc = ptr::null_mut();

        let mut idx = -1;
        for i in 0..PROCS_MAX {
            unsafe {
                if PROCS[i].state == PROC_UNUSED {
                    proc = &mut PROCS[i] as *mut Process;
                    idx = i as i64;
                    break;
                }
            }
        }

        if proc.is_null() {
            panic!("no free process slots");
        }

        unsafe {
            let sp = (&mut (*proc).stack as *mut [u8] as *mut u8)
                .offset(mem::size_of_val(&(*proc).stack) as isize) as *mut u64;

            *sp.sub(1) = 0; // s11
            *sp.sub(2) = 0; // s10
            *sp.sub(3) = 0; // s9
            *sp.sub(4) = 0; // s8
            *sp.sub(5) = 0; // s7
            *sp.sub(6) = 0; // s6
            *sp.sub(7) = 0; // s5
            *sp.sub(8) = 0; // s4
            *sp.sub(9) = 0; // s3
            *sp.sub(10) = 0; // s2
            *sp.sub(11) = 0; // s1
            *sp.sub(12) = 0; // s0
            *sp.sub(13) = pc; // ra

            let page_table = alloc_pages(1);
            let mut paddr = ptr::addr_of!(__kernel_base) as *const u8 as PhysAddr;
            while paddr < ptr::addr_of!(__free_ram_end) as *const u8 as PhysAddr {
                use crate::paging::*;
                map_page(page_table, paddr, paddr, PAGE_R | PAGE_W | PAGE_X);
                paddr += PAGE_SIZE;
            }

            (*proc).pid = idx + 1;
            (*proc).state = PROC_RUNNABLE;
            (*proc).sp = sp.sub(13) as VirtAddr;
            (*proc).page_table = page_table;
        }

        proc
    }
}

#[no_mangle]
static mut PROCS: [Process; PROCS_MAX] = [Process::new(); PROCS_MAX];

global_asm!(
    r#"
.align 8
.global switch_context
switch_context:
    addi sp, sp, -13 * 8
    sd ra,  0  * 8(sp)
    sd s0,  1  * 8(sp)
    sd s1,  2  * 8(sp)
    sd s2,  3  * 8(sp)
    sd s3,  4  * 8(sp)
    sd s4,  5  * 8(sp)
    sd s5,  6  * 8(sp)
    sd s6,  7  * 8(sp)
    sd s7,  8  * 8(sp)
    sd s8,  9  * 8(sp)
    sd s9,  10 * 8(sp)
    sd s10, 11 * 8(sp)
    sd s11, 12 * 8(sp)
    sd sp, (a0)
    ld sp, (a1)
    ld ra,  0  * 8(sp)
    ld s0,  1  * 8(sp)
    ld s1,  2  * 8(sp)
    ld s2,  3  * 8(sp)
    ld s3,  4  * 8(sp)
    ld s4,  5  * 8(sp)
    ld s5,  6  * 8(sp)
    ld s6,  7  * 8(sp)
    ld s7,  8  * 8(sp)
    ld s8,  9  * 8(sp)
    ld s9,  10 * 8(sp)
    ld s10, 11 * 8(sp)
    ld s11, 12 * 8(sp)
    addi sp, sp, 13 * 8
    ret
    "#
);

pub static mut CURRENT_PROC: *mut Process = ptr::null_mut();
pub static mut IDLE_PROC: *mut Process = ptr::null_mut();

pub unsafe fn process_yield() {
    let mut next = IDLE_PROC;
    for i in 0..PROCS_MAX {
        let proc = &mut PROCS
            [(CURRENT_PROC.as_ref().unwrap().pid as usize).wrapping_add(i) % PROCS_MAX as usize]
            as *mut Process;
        if (*proc).state == PROC_RUNNABLE && (*proc).pid > 0 {
            next = proc;
            break;
        }
    }

    if next == CURRENT_PROC {
        return;
    }

    asm!(
        "sfence.vma",
        "csrw satp, {satp}",
        "sfence.vma",
        satp = in(reg) (SATP_SV39 | ((*next).page_table / PAGE_SIZE))
    );
    write_csr!(
        "sscratch",
        (&mut (*next).stack as *mut [u8] as *mut u8)
            .offset(mem::size_of_val(&(*next).stack) as isize) as *mut u64
    );

    let prev = CURRENT_PROC;
    CURRENT_PROC = next;
    switch_context(&mut (*prev).sp, &(*next).sp)
}
