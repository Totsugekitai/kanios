use core::arch::asm;

const EID_CONSOLE_PUTCHAR: i64 = 0x01;

pub struct SbiRet {
    pub error: i64,
    pub value: i64,
}

unsafe fn sbi_call(
    arg0: i64,
    arg1: i64,
    arg2: i64,
    arg3: i64,
    arg4: i64,
    arg5: i64,
    fid: i64,
    eid: i64,
) -> SbiRet {
    let mut error;
    let mut value;
    asm!(
        "ecall",
        inout("a0") arg0 => error, inout("a1") arg1 => value,
        in("a2") arg2, in("a3") arg3, in("a4") arg4, in("a5") arg5,
        in("a6") fid, in("a7") eid
    );
    SbiRet { error, value }
}

pub fn putchar(ch: u8) {
    unsafe {
        sbi_call(ch as i64, 0, 0, 0, 0, 0, 0, EID_CONSOLE_PUTCHAR);
    }
}

pub fn getchar() -> i64 {
    let ret = unsafe { sbi_call(0, 0, 0, 0, 0, 0, 0, 2) };
    ret.error
}
