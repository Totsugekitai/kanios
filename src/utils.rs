#[macro_export]
macro_rules! read_csr {
    ($csr:expr) => {
        unsafe {
            use core::arch::asm;
            let mut csrr: u64;
            asm!(concat!("csrr {r}, ", $csr), r = out(reg) csrr);
            csrr
        }
    };
}

#[macro_export]
macro_rules! write_csr {
    ($csr:expr, $value:expr) => {
        unsafe {
            use core::arch::asm;
            asm!(concat!("csrw ", $csr, ", {r}"), r = in(reg) $value);
        }
    };
}

pub const fn align_up(value: u64, align: u64) -> u64 {
    let r = value % align;
    if r == 0 {
        value
    } else {
        value + (align - r)
    }
}

pub fn oct2int(oct: *const u8, len: usize) -> u32 {
    let mut dec = 0;
    for i in 0..len {
        unsafe {
            if *oct.add(i) < b'0' || *oct.add(i) > b'7' {
                break;
            }
            dec = dec * 8 + (*oct.add(i) - b'0') as u32;
        }
    }
    dec
}

pub fn ascii_len(buf: *const u8) -> usize {
    let len;
    let mut i = 0;
    loop {
        if unsafe { *buf.add(i as usize) } == b'\0' {
            len = i + 1;
            break;
        }
        i += 1;
    }
    len
}
