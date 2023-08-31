#![no_std]
#![no_main]

mod console;
mod handler;
mod memory;
mod print;
mod process;
mod sbi;
mod types;
mod utils;

use crate::process::{process_yield, CURRENT_PROC, IDLE_PROC};
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    ptr,
};
use process::Process;

extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    fn kernel_entry();
}

#[no_mangle]
fn kernel_main() -> ! {
    clear_bss();

    write_csr!("stvec", kernel_entry as u64);

    let paddr0 = unsafe { memory::alloc_pages(2) };
    let paddr1 = unsafe { memory::alloc_pages(1) };
    println!("alloc_pages test: paddr0={paddr0:x}");
    println!("alloc_pages test: paddr1={paddr1:x}");

    unsafe {
        IDLE_PROC = Process::create(0);
        (*IDLE_PROC).pid = -1;
        CURRENT_PROC = IDLE_PROC;

        Process::create(proc_a_entry as u64);
        Process::create(proc_b_entry as u64);
        process_yield();
    }

    unsafe {
        asm!("unimp");
    }

    panic!("booted!");
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

fn proc_a_entry() {
    println!("starting process A");
    loop {
        print!("A");
        unsafe {
            process_yield();

            for _ in 0..3000000 {
                asm!("nop");
            }
        }
    }
}

fn proc_b_entry() {
    println!("starting process B");
    loop {
        print!("B");
        unsafe {
            process_yield();

            for _ in 0..3000000 {
                asm!("nop");
            }
        }
    }
}
