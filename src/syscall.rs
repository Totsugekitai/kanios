use crate::{
    handler::TrapFrame,
    process::process_yield,
    sbi::{getchar, putchar},
};

const SYS_PUTCHAR: u64 = 1;
const SYS_GETCHAR: u64 = 2;

pub fn handle_syscall(f: *mut TrapFrame) {
    let f = unsafe { f.as_mut().unwrap() };
    let sysno = f.a3;
    match sysno {
        SYS_PUTCHAR => putchar(f.a0 as u8),
        SYS_GETCHAR => loop {
            let ch = getchar();
            if ch >= 0 {
                f.a0 = ch as u64;
                break;
            }

            unsafe {
                process_yield();
            }
        },
        _ => panic!("unexpected syscall a3={:x}", sysno),
    }
}
