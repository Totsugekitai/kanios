use crate::{
    handler::TrapFrame,
    println,
    process::{process_yield, CURRENT_PROC, PROC_EXITED},
    sbi::{getchar, putchar},
    tarfs,
    utils::ascii_len,
};
use core::{mem, slice};

const SYS_PUTCHAR: u64 = 1;
const SYS_GETCHAR: u64 = 2;
const SYS_EXIT: u64 = 3;
const SYS_READFILE: u64 = 4;
const SYS_WRITEFILE: u64 = 5;

pub fn handle_syscall(f: *mut TrapFrame) {
    let f = unsafe { f.as_mut().unwrap() };
    let sysno = f.a3;
    match sysno {
        SYS_PUTCHAR => {
            putchar(f.a0 as u8);
        }
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
        SYS_READFILE | SYS_WRITEFILE => {
            let filename = f.a0 as *const u8;
            let filename_len = ascii_len(filename);
            let filename = unsafe {
                core::str::from_utf8(slice::from_raw_parts(filename, filename_len)).unwrap()
            };
            let buf = f.a1 as *mut u8;
            let mut len = f.a2 as usize;
            let file = if let Ok(f) = tarfs::lookup(filename) {
                unsafe { f.as_mut().unwrap() }
            } else {
                println!("file not found: {}", filename);
                f.a0 = 0xffff_ffff_ffff_fffe as u64;
                return;
            };

            if len > mem::size_of_val(&file.data) {
                len = file.size;
            }
            let buf = unsafe { slice::from_raw_parts_mut(buf, len) };

            if sysno == SYS_WRITEFILE {
                file.data[0..buf.len()].copy_from_slice(buf);
                file.size = len;
                unsafe { tarfs::flush() };
            } else {
                buf.copy_from_slice(&file.data[0..len]);
            }

            f.a0 = len as u64;
        }
        _ => panic!("unexpected syscall a3={:x}", sysno),
    }
}
