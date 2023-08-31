#![no_std]
#![no_main]

mod console;
mod handler;
mod print;
mod sbi;
mod types;
mod utils;

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    ptr,
};

extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    fn kernel_entry();
}

#[no_mangle]
fn kernel_main() -> ! {
    clear_bss();

    write_csr!("stvec", kernel_entry as u64);
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
