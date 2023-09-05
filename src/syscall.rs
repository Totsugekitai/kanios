use crate::{
    handler::TrapFrame,
    println,
    process::{process_yield, CURRENT_PROC, PROC_EXITED},
    sbi::{getchar, putchar},
};

const SYS_PUTCHAR: u64 = 1;
const SYS_GETCHAR: u64 = 2;
const SYS_EXIT: u64 = 3;

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
        SYS_EXIT => {
            let current = unsafe { CURRENT_PROC.as_mut().unwrap() };
            println!("process {} exited", current.pid);
            current.state = PROC_EXITED;
            unsafe {
                process_yield();
            }
            unreachable!();
        }
        _ => panic!("unexpected syscall a3={:x}", sysno),
    }
}
