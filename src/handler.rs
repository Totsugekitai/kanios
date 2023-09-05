use crate::{println, read_csr, syscall::handle_syscall, write_csr};
use core::arch::global_asm;

const SCAUSE_ECALL: u64 = 8;

#[repr(C, packed)]
#[derive(Debug)]
pub struct TrapFrame {
    pub ra: u64,
    pub gp: u64,
    pub tp: u64,
    pub t0: u64,
    pub t1: u64,
    pub t2: u64,
    pub t3: u64,
    pub t4: u64,
    pub t5: u64,
    pub t6: u64,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub a4: u64,
    pub a5: u64,
    pub a6: u64,
    pub a7: u64,
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
    pub sp: u64,
}

global_asm!(
    r#"
.align 8
.global kernel_entry
kernel_entry:
    csrrw sp, sscratch, sp

    addi sp, sp, -8 * 31
    sd ra,  8 * 0(sp)
    sd gp,  8 * 1(sp)
    sd tp,  8 * 2(sp)
    sd t0,  8 * 3(sp)
    sd t1,  8 * 4(sp)
    sd t2,  8 * 5(sp)
    sd t3,  8 * 6(sp)
    sd t4,  8 * 7(sp)
    sd t5,  8 * 8(sp)
    sd t6,  8 * 9(sp)
    sd a0,  8 * 10(sp)
    sd a1,  8 * 11(sp)
    sd a2,  8 * 12(sp)
    sd a3,  8 * 13(sp)
    sd a4,  8 * 14(sp)
    sd a5,  8 * 15(sp)
    sd a6,  8 * 16(sp)
    sd a7,  8 * 17(sp)
    sd s0,  8 * 18(sp)
    sd s1,  8 * 19(sp)
    sd s2,  8 * 20(sp)
    sd s3,  8 * 21(sp)
    sd s4,  8 * 22(sp)
    sd s5,  8 * 23(sp)
    sd s6,  8 * 24(sp)
    sd s7,  8 * 25(sp)
    sd s8,  8 * 26(sp)
    sd s9,  8 * 27(sp)
    sd s10, 8 * 28(sp)
    sd s11, 8 * 29(sp)

    csrr a0, sscratch
    sd a0, 8 * 30(sp)

    addi a0, sp, 8 * 31
    csrw sscratch, a0

    mv a0, sp
    call handle_trap

    ld ra,  8 * 0(sp)
    ld gp,  8 * 1(sp)
    ld tp,  8 * 2(sp)
    ld t0,  8 * 3(sp)
    ld t1,  8 * 4(sp)
    ld t2,  8 * 5(sp)
    ld t3,  8 * 6(sp)
    ld t4,  8 * 7(sp)
    ld t5,  8 * 8(sp)
    ld t6,  8 * 9(sp)
    ld a0,  8 * 10(sp)
    ld a1,  8 * 11(sp)
    ld a2,  8 * 12(sp)
    ld a3,  8 * 13(sp)
    ld a4,  8 * 14(sp)
    ld a5,  8 * 15(sp)
    ld a6,  8 * 16(sp)
    ld a7,  8 * 17(sp)
    ld s0,  8 * 18(sp)
    ld s1,  8 * 19(sp)
    ld s2,  8 * 20(sp)
    ld s3,  8 * 21(sp)
    ld s4,  8 * 22(sp)
    ld s5,  8 * 23(sp)
    ld s6,  8 * 24(sp)
    ld s7,  8 * 25(sp)
    ld s8,  8 * 26(sp)
    ld s9,  8 * 27(sp)
    ld s10, 8 * 28(sp)
    ld s11, 8 * 29(sp)
    ld sp,  8 * 30(sp)
    sret
    "#
);

#[no_mangle]
fn handle_trap(f: *mut TrapFrame) {
    let scause = read_csr!("scause");
    let stval = read_csr!("stval");
    let mut user_pc = read_csr!("sepc");

    if scause == SCAUSE_ECALL {
        handle_syscall(f);
        user_pc += 4;
    } else {
        let f = unsafe { f.as_ref().unwrap() };
        println!("{f:#x?}");
        panic!("unexpected trap scause={scause:x}, stval={stval:x}, sepc={user_pc:x}");
    }

    write_csr!("sepc", user_pc);
}

global_asm!(
    r#"
.align 8
.global user_entry
user_entry:
    addi sp, sp, -8 * 2
    sd a0, 0 * 8(sp)
    sd a1, 1 * 8(sp)
    
    ld a0, (USER_BASE)
    csrw sepc, a0
    ld a0, (SSTATUS_SPIE)
    ld a1, (SSTATUS_SUM)
    or a0, a0 ,a1
    csrw sstatus, a0
    
    ld a0, 0 * 8(sp)
    ld a1, 1 * 8(sp)
    addi sp, sp, 8 * 2
    sret
    "#
);
