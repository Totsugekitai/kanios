use crate::{
    __free_ram_end, __kernel_base,
    elf::ElfHeader,
    memory::{alloc_pages, PAGE_SIZE},
    paging::{map_page, PAGE_R, PAGE_U, PAGE_W, PAGE_X, SATP_SV39},
    types::{PhysAddr, VirtAddr},
    virtio_blk::VIRTIO_BLK_PADDR,
    write_csr,
};
use core::{
    arch::{asm, global_asm},
    mem, ptr,
};

extern "C" {
    fn switch_context(prev_sp: *mut VirtAddr, next_sp: *const VirtAddr);
    fn user_entry();
}

const PROCS_MAX: usize = 8;
pub const PROC_UNUSED: i64 = 0;
pub const PROC_RUNNABLE: i64 = 1;
pub const PROC_EXITED: i64 = 2;

#[no_mangle]
pub static USER_BASE: u64 = 0x100_0000;
#[no_mangle]
pub static SSTATUS_SPIE: u64 = 1 << 5;
#[no_mangle]
pub static SSTATUS_SUM: u64 = 1 << 18;

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
            sp: VirtAddr::new(0),
            page_table: PhysAddr::new(0),
            stack: [0; 8192],
        }
    }

    pub fn create(image: *const ElfHeader) -> *mut Process {
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
            *sp.sub(13) = user_entry as u64; // ra

            let page_table = alloc_pages(1);

            // カーネルのページをマッピングする
            let mut paddr = PhysAddr::new(ptr::addr_of!(__kernel_base) as *const u8 as u64);
            while paddr < PhysAddr::new(ptr::addr_of!(__free_ram_end) as *const u8 as u64) {
                map_page(
                    page_table,
                    VirtAddr::new(paddr.as_u64()),
                    paddr,
                    PAGE_R | PAGE_W | PAGE_X,
                );
                paddr += PhysAddr::new(PAGE_SIZE);
            }
            map_page(
                page_table,
                VirtAddr::new(VIRTIO_BLK_PADDR.as_u64()),
                VIRTIO_BLK_PADDR,
                PAGE_R | PAGE_W,
            );

            // ユーザーのページをマッピングする
            if image != ptr::null() {
                let ehdr = image.as_ref().unwrap();
                let count = ehdr.count_pages();
                let page = alloc_pages(count as u64);
                ehdr.load(page);

                for i in 0..count {
                    map_page(
                        page_table,
                        VirtAddr::new(USER_BASE + (i * 0x1000) as u64),
                        page + PhysAddr::new((i * 0x1000) as u64),
                        PAGE_U | PAGE_R | PAGE_W | PAGE_X,
                    );
                }
            }

            (*proc).pid = idx + 1;
            (*proc).state = PROC_RUNNABLE;
            (*proc).sp = VirtAddr::new(sp.sub(13) as u64);
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
        satp = in(reg) (((*next).page_table.as_u64() / PAGE_SIZE) | SATP_SV39)
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
