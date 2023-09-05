#![no_std]
#![no_main]
#![feature(offset_of)]

mod elf;
mod handler;
mod memory;
mod paging;
mod print;
mod process;
mod sbi;
mod syscall;
mod tarfs;
mod types;
mod utils;
mod virtio_blk;

use crate::{
    elf::ElfHeader,
    process::{process_yield, CURRENT_PROC, IDLE_PROC},
};
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    ptr,
};
use process::Process;

extern "C" {
    pub static __kernel_base: u8;
    static mut __bss: u8;
    static __bss_end: u8;
    pub static __free_ram_end: u8;
    fn kernel_entry();
}

#[no_mangle]
fn kernel_main() -> ! {
    clear_bss();

    write_csr!("stvec", kernel_entry as u64);

    unsafe {
        virtio_blk::init();
        tarfs::init();

        IDLE_PROC = Process::create(ptr::null());
        (*IDLE_PROC).pid = -1;
        CURRENT_PROC = IDLE_PROC;

        let shell = &tarfs::lookup("shell.elf").unwrap().as_ref().unwrap().data as *const u8
            as *const ElfHeader;
        Process::create(shell);
        process_yield();
    }

    panic!("switched to idle process");
}

global_asm!(
    r#"
.section ".text.boot"
.global boot
boot:
    la sp, __stack_top
    j  kernel_main
    "#
);

fn clear_bss() {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

// fn proc_a_entry() {
//     println!("starting process A");
//     loop {
//         print!("A");
//         unsafe {
//             process_yield();

//             for _ in 0..3000000 {
//                 asm!("nop");
//             }
//         }
//     }
// }

// fn proc_b_entry() {
//     println!("starting process B");
//     loop {
//         print!("B");
//         unsafe {
//             process_yield();

//             for _ in 0..3000000 {
//                 asm!("nop");
//             }
//         }
//     }
// }
