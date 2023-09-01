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

pub fn align_up(value: u64, align: u64) -> u64 {
    let r = value % align;
    if r == 0 {
        value
    } else {
        value + (align - r)
    }
}
